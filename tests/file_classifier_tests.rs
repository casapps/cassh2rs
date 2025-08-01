use cassh2rs::resolver::{FileClassifier, FileClassification, FileContext, FileUsage, UsagePattern};
use std::path::Path;

#[test]
fn test_system_paths_classification() {
    let classifier = FileClassifier::new();
    let context = FileContext::default();
    
    let system_paths = vec![
        "/proc/cpuinfo",
        "/sys/class/net/eth0/address",
        "/dev/null",
        "/tmp/test.txt",
        "/var/log/syslog",
        "/run/systemd/system",
    ];
    
    for path in system_paths {
        let info = classifier.classify(Path::new(path), &context);
        assert_eq!(
            info.classification, 
            FileClassification::Runtime,
            "Path {} should be Runtime", 
            path
        );
        assert!(info.reason.contains("System path") || info.reason.contains("runtime"));
    }
}

#[test]
fn test_sensitive_files_classification() {
    let classifier = FileClassifier::new();
    let context = FileContext::default();
    
    let sensitive_files = vec![
        "id_rsa.key",
        "server.pem",
        "database.password",
        "api.token",
        "credentials.secret",
    ];
    
    for file in sensitive_files {
        let info = classifier.classify(Path::new(file), &context);
        assert_eq!(
            info.classification, 
            FileClassification::Runtime,
            "File {} should be Runtime", 
            file
        );
        assert!(info.reason.contains("Sensitive"));
    }
}

#[test]
fn test_static_files_classification() {
    let classifier = FileClassifier::new();
    let mut context = FileContext::default();
    context.is_local_to_script = true;
    
    let static_files = vec![
        "config.toml",
        "settings.json",
        "README.md",
        "LICENSE",
        "CHANGELOG.md",
    ];
    
    for file in static_files {
        let info = classifier.classify(Path::new(file), &context);
        assert_eq!(
            info.classification, 
            FileClassification::Static,
            "File {} should be Static", 
            file
        );
    }
}

#[test]
fn test_process_files_classification() {
    let classifier = FileClassifier::new();
    let context = FileContext::default();
    
    let process_files = vec![
        "app.pid",
        "service.lock",
        "daemon.sock",
    ];
    
    for file in process_files {
        let info = classifier.classify(Path::new(file), &context);
        assert_eq!(
            info.classification, 
            FileClassification::Runtime,
            "File {} should be Runtime", 
            file
        );
        assert!(info.reason.contains("Process file"));
    }
}

#[test]
fn test_cache_paths_classification() {
    let classifier = FileClassifier::new();
    let context = FileContext::default();
    
    let cache_paths = vec![
        "/home/user/.cache/app/data",
        "/home/user/.local/tmp/file",
        "/var/cache/apt/archives",
    ];
    
    for path in cache_paths {
        let info = classifier.classify(Path::new(path), &context);
        assert_eq!(
            info.classification, 
            FileClassification::Runtime,
            "Path {} should be Runtime", 
            path
        );
        assert!(info.reason.contains("Cache"));
    }
}

#[test]
fn test_file_size_classification() {
    let classifier = FileClassifier::with_max_size(10 * 1024 * 1024); // 10MB limit
    let mut context = FileContext::default();
    
    // Small file
    context.size = Some(1024 * 1024); // 1MB
    let info = classifier.classify(Path::new("data.bin"), &context);
    assert_eq!(info.classification, FileClassification::ContextDependent);
    
    // Large file
    context.size = Some(100 * 1024 * 1024); // 100MB
    let info = classifier.classify(Path::new("large.bin"), &context);
    assert_eq!(info.classification, FileClassification::Runtime);
    assert!(info.reason.contains("too large"));
}

#[test]
fn test_modified_file_classification() {
    let classifier = FileClassifier::new();
    let mut context = FileContext::default();
    context.is_modified = true;
    
    let info = classifier.classify(Path::new("output.txt"), &context);
    assert_eq!(info.classification, FileClassification::Runtime);
    assert!(info.reason.contains("modified by script"));
}

#[test]
fn test_monitored_file_classification() {
    let classifier = FileClassifier::new();
    let mut context = FileContext::default();
    context.is_monitored = true;
    
    let info = classifier.classify(Path::new("log.txt"), &context);
    assert_eq!(info.classification, FileClassification::Runtime);
    assert!(info.reason.contains("monitored"));
}

#[test]
fn test_template_files_classification() {
    let classifier = FileClassifier::new();
    let mut context = FileContext::default();
    context.is_local_to_script = true;
    
    let template_files = vec![
        "email.template",
        "config.tmpl",
        "report.tpl",
        "template_header.html",
    ];
    
    for file in template_files {
        let info = classifier.classify(Path::new(file), &context);
        assert_eq!(
            info.classification, 
            FileClassification::Static,
            "File {} should be Static", 
            file
        );
        assert!(info.reason.contains("Template"));
    }
}

#[test]
fn test_usage_pattern_context() {
    let classifier = FileClassifier::new();
    
    // Read-only usage
    let mut usage = FileUsage::default();
    usage.read_count = 5;
    let context = FileContext::from_usage(&usage);
    assert_eq!(context.usage_pattern, UsagePattern::ReadOnly);
    
    // Write-only usage
    let mut usage = FileUsage::default();
    usage.write_count = 3;
    let context = FileContext::from_usage(&usage);
    assert_eq!(context.usage_pattern, UsagePattern::WriteOnly);
    
    // Read-write usage
    let mut usage = FileUsage::default();
    usage.read_count = 2;
    usage.write_count = 1;
    let context = FileContext::from_usage(&usage);
    assert_eq!(context.usage_pattern, UsagePattern::ReadWrite);
    
    // Append usage
    let mut usage = FileUsage::default();
    usage.append_count = 1;
    let context = FileContext::from_usage(&usage);
    assert_eq!(context.usage_pattern, UsagePattern::Append);
    
    // Monitor usage
    let mut usage = FileUsage::default();
    usage.is_monitored = true;
    let context = FileContext::from_usage(&usage);
    assert_eq!(context.usage_pattern, UsagePattern::Monitor);
}

#[test]
fn test_etc_files_classification() {
    let classifier = FileClassifier::new();
    
    // System /etc file
    let mut context = FileContext::default();
    context.is_local_to_script = false;
    let info = classifier.classify(Path::new("/etc/passwd"), &context);
    assert_eq!(info.classification, FileClassification::Runtime);
    
    // Local etc file (relative to script)
    context.is_local_to_script = true;
    let info = classifier.classify(Path::new("etc/config.conf"), &context);
    assert_eq!(info.classification, FileClassification::ContextDependent);
}