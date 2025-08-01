use std::path::{Path, PathBuf};
use regex::Regex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileClassification {
    /// Always embed at compile time
    Static,
    /// Never embed, always access at runtime
    Runtime,
    /// Depends on context and usage
    ContextDependent,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub classification: FileClassification,
    pub reason: String,
    pub size: Option<u64>,
}

pub struct FileClassifier {
    max_embed_size: u64,
}

impl Default for FileClassifier {
    fn default() -> Self {
        Self {
            max_embed_size: 50 * 1024 * 1024, // 50MB
        }
    }
}

// Lazy static regexes for performance
static SYSTEM_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(/proc/|/sys/|/dev/|/tmp/|/var/log/|/run/)").unwrap()
});

static CACHE_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\.cache/|/cache/|\.local/tmp/)").unwrap()
});

static SENSITIVE_FILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.(key|pem|password|secret|token|credentials)$").unwrap()
});

static PROCESS_FILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.(pid|lock|sock)$").unwrap()
});

static CONFIG_FILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.(conf|config|cfg|ini|toml|yaml|yml|json)$").unwrap()
});

static DOC_FILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(README|LICENSE|COPYING|AUTHORS|CHANGELOG|TODO|INSTALL)").unwrap()
});

static DATA_FILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.(json|yaml|yml|toml|xml|csv|tsv)$").unwrap()
});

impl FileClassifier {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_max_size(max_embed_size: u64) -> Self {
        Self { max_embed_size }
    }
    
    pub fn classify(&self, path: &Path, context: &FileContext) -> FileInfo {
        let path_str = path.to_string_lossy();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check for always-runtime patterns first
        if let Some(reason) = self.is_always_runtime(path, &path_str, filename, context) {
            return FileInfo {
                path: path.to_path_buf(),
                classification: FileClassification::Runtime,
                reason,
                size: context.size,
            };
        }
        
        // Check for always-static patterns
        if let Some(reason) = self.is_always_static(path, &path_str, filename, context) {
            return FileInfo {
                path: path.to_path_buf(),
                classification: FileClassification::Static,
                reason,
                size: context.size,
            };
        }
        
        // Everything else is context-dependent
        FileInfo {
            path: path.to_path_buf(),
            classification: FileClassification::ContextDependent,
            reason: "Requires context analysis".to_string(),
            size: context.size,
        }
    }
    
    fn is_always_runtime(&self, path: &Path, path_str: &str, filename: &str, context: &FileContext) -> Option<String> {
        // System paths
        if SYSTEM_PATH_REGEX.is_match(path_str) {
            return Some("System path - always runtime".to_string());
        }
        
        // Cache directories
        if CACHE_PATH_REGEX.is_match(path_str) {
            return Some("Cache directory - always runtime".to_string());
        }
        
        // Process files
        if PROCESS_FILE_REGEX.is_match(filename) {
            return Some("Process file (pid/lock/sock) - always runtime".to_string());
        }
        
        // Files modified by the script
        if context.is_modified {
            return Some("File is modified by script - must be runtime".to_string());
        }
        
        // Large files
        if let Some(size) = context.size {
            if size > self.max_embed_size {
                return Some(format!("File too large ({}MB > {}MB limit)", 
                    size / 1024 / 1024, 
                    self.max_embed_size / 1024 / 1024
                ));
            }
        }
        
        // Sensitive files
        if SENSITIVE_FILE_REGEX.is_match(filename) {
            return Some("Sensitive file (key/password) - never embed".to_string());
        }
        
        // Monitoring contexts
        if context.is_monitored {
            return Some("File is monitored (tail -f/watch) - runtime access required".to_string());
        }
        
        // Special directories
        if path_str.starts_with("/etc/") && !context.is_local_to_script {
            return Some("System configuration in /etc - runtime access".to_string());
        }
        
        None
    }
    
    fn is_always_static(&self, path: &Path, path_str: &str, filename: &str, context: &FileContext) -> Option<String> {
        // Local configs in script directory
        if context.is_local_to_script && CONFIG_FILE_REGEX.is_match(filename) {
            return Some("Local configuration file - embed".to_string());
        }
        
        // Documentation files
        if DOC_FILE_REGEX.is_match(filename) {
            return Some("Documentation file - embed".to_string());
        }
        
        // Small data files
        if DATA_FILE_REGEX.is_match(filename) {
            if let Some(size) = context.size {
                if size < 1024 * 1024 { // < 1MB
                    return Some("Small data file - embed".to_string());
                }
            }
        }
        
        // Markdown files
        if filename.ends_with(".md") {
            return Some("Markdown documentation - embed".to_string());
        }
        
        // Template files
        if filename.contains("template") || filename.ends_with(".tmpl") || filename.ends_with(".tpl") {
            return Some("Template file - embed".to_string());
        }
        
        // Source files for inclusion
        if context.is_sourced && context.is_local_to_script {
            return Some("Local sourced script - embed".to_string());
        }
        
        None
    }
    
    pub fn classify_with_usage(&self, path: &Path, usage: &FileUsage) -> FileInfo {
        let mut context = FileContext::from_usage(usage);
        
        // Get file size if possible
        if let Ok(metadata) = std::fs::metadata(path) {
            context.size = Some(metadata.len());
        }
        
        self.classify(path, &context)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileContext {
    pub is_local_to_script: bool,
    pub is_modified: bool,
    pub is_monitored: bool,
    pub is_sourced: bool,
    pub size: Option<u64>,
    pub usage_pattern: UsagePattern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsagePattern {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Append,
    Monitor,
    Source,
    Unknown,
}

impl Default for UsagePattern {
    fn default() -> Self {
        UsagePattern::Unknown
    }
}

#[derive(Debug, Clone)]
pub struct FileUsage {
    pub read_count: usize,
    pub write_count: usize,
    pub append_count: usize,
    pub in_loop: bool,
    pub in_condition: bool,
    pub is_sourced: bool,
    pub is_monitored: bool,
    pub commands: Vec<String>,
}

impl Default for FileUsage {
    fn default() -> Self {
        Self {
            read_count: 0,
            write_count: 0,
            append_count: 0,
            in_loop: false,
            in_condition: false,
            is_sourced: false,
            is_monitored: false,
            commands: Vec::new(),
        }
    }
}

impl FileContext {
    pub fn from_usage(usage: &FileUsage) -> Self {
        let is_modified = usage.write_count > 0 || usage.append_count > 0;
        
        let usage_pattern = if usage.is_monitored {
            UsagePattern::Monitor
        } else if usage.is_sourced {
            UsagePattern::Source
        } else if usage.write_count > 0 && usage.read_count == 0 {
            UsagePattern::WriteOnly
        } else if usage.append_count > 0 {
            UsagePattern::Append
        } else if usage.write_count > 0 && usage.read_count > 0 {
            UsagePattern::ReadWrite
        } else if usage.read_count > 0 {
            UsagePattern::ReadOnly
        } else {
            UsagePattern::Unknown
        };
        
        Self {
            is_local_to_script: false, // This needs to be determined externally
            is_modified,
            is_monitored: usage.is_monitored,
            is_sourced: usage.is_sourced,
            size: None,
            usage_pattern,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_paths() {
        let classifier = FileClassifier::new();
        let context = FileContext::default();
        
        let paths = vec![
            "/proc/cpuinfo",
            "/sys/class/net/eth0/address",
            "/dev/null",
            "/tmp/test.txt",
            "/var/log/syslog",
        ];
        
        for path in paths {
            let info = classifier.classify(Path::new(path), &context);
            assert_eq!(info.classification, FileClassification::Runtime);
            assert!(info.reason.contains("System path"));
        }
    }
    
    #[test]
    fn test_sensitive_files() {
        let classifier = FileClassifier::new();
        let context = FileContext::default();
        
        let paths = vec![
            "private.key",
            "server.pem",
            "db.password",
            "api.token",
        ];
        
        for path in paths {
            let info = classifier.classify(Path::new(path), &context);
            assert_eq!(info.classification, FileClassification::Runtime);
            assert!(info.reason.contains("Sensitive"));
        }
    }
    
    #[test]
    fn test_static_files() {
        let classifier = FileClassifier::new();
        let mut context = FileContext::default();
        context.is_local_to_script = true;
        
        let paths = vec![
            "config.toml",
            "settings.json",
            "README.md",
            "LICENSE",
        ];
        
        for path in paths {
            let info = classifier.classify(Path::new(path), &context);
            assert_eq!(info.classification, FileClassification::Static);
        }
    }
}