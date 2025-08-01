use crate::parser::{AST, ASTNode};
use super::file_classifier::{FileClassifier, FileUsage, FileContext};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use regex::Regex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub path: PathBuf,
    pub dep_type: DependencyType,
    pub usage: FileUsage,
    pub line_numbers: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyType {
    SourceFile,      // Script sources this file
    DataFile,        // File read/written by script
    BinaryCommand,   // External command
    NetworkResource, // URL or remote resource
    Directory,       // Directory accessed
    ConfigFile,      // Configuration file
}

pub struct DependencyResolver {
    classifier: FileClassifier,
    script_dir: PathBuf,
    dependencies: HashMap<PathBuf, Dependency>,
    visited_sources: HashSet<PathBuf>,
    max_source_depth: usize,
}

// Regex patterns for detecting file paths and resources
static FILE_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[\s"'`(])(/[^/\s"'`()]+(?:/[^/\s"'`()]+)*/?|\./[^/\s"'`()]+(?:/[^/\s"'`()]+)*/?|\.\.(?:/[^/\s"'`()]+)+/?)"#).unwrap()
});

static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s<>\"{}|\\^`\[\]]+").unwrap()
});

static VARIABLE_FILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap()
});

impl DependencyResolver {
    pub fn new(script_path: &Path) -> Result<Self> {
        let script_dir = script_path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        
        Ok(Self {
            classifier: FileClassifier::new(),
            script_dir,
            dependencies: HashMap::new(),
            visited_sources: HashSet::new(),
            max_source_depth: 15,
        })
    }
    
    pub fn resolve(&mut self, ast: &AST) -> Result<Vec<Dependency>> {
        // Start with the main script
        self.analyze_ast_node(&ast.root, 0)?;
        
        // Extract dependencies from metadata
        for dep in &ast.metadata.dependencies {
            self.add_dependency(
                PathBuf::from(dep),
                DependencyType::BinaryCommand,
                FileUsage::default(),
                vec![],
            );
        }
        
        // Return all collected dependencies
        Ok(self.dependencies.values().cloned().collect())
    }
    
    fn analyze_ast_node(&mut self, node: &ASTNode, depth: usize) -> Result<()> {
        if depth > self.max_source_depth {
            anyhow::bail!("Maximum source depth exceeded");
        }
        
        match node {
            ASTNode::Script(statements) | ASTNode::Block(statements) => {
                for stmt in statements {
                    self.analyze_ast_node(stmt, depth)?;
                }
            }
            
            ASTNode::Command { name, args, .. } => {
                self.analyze_command(name, args, depth)?;
            }
            
            ASTNode::Pipeline(commands) => {
                for cmd in commands {
                    self.analyze_ast_node(cmd, depth)?;
                }
            }
            
            ASTNode::If { condition, then_block, elif_blocks, else_block } => {
                self.analyze_ast_node(condition, depth)?;
                self.analyze_ast_node(then_block, depth)?;
                for (cond, block) in elif_blocks {
                    self.analyze_ast_node(cond, depth)?;
                    self.analyze_ast_node(block, depth)?;
                }
                if let Some(block) = else_block {
                    self.analyze_ast_node(block, depth)?;
                }
            }
            
            ASTNode::While { condition, body } | ASTNode::Until { condition, body } => {
                self.analyze_ast_node(condition, depth)?;
                self.analyze_ast_node(body, depth)?;
            }
            
            ASTNode::For { items, body, .. } => {
                match items {
                    crate::parser::ast::ForItems::List(list) => {
                        for item in list {
                            self.analyze_ast_node(item, depth)?;
                        }
                    }
                    crate::parser::ast::ForItems::Command(cmd) => {
                        self.analyze_ast_node(cmd, depth)?;
                    }
                    crate::parser::ast::ForItems::CStyle { init, condition, update } => {
                        self.analyze_ast_node(init, depth)?;
                        self.analyze_ast_node(condition, depth)?;
                        self.analyze_ast_node(update, depth)?;
                    }
                }
                self.analyze_ast_node(body, depth)?;
            }
            
            ASTNode::Case { expr, cases } => {
                self.analyze_ast_node(expr, depth)?;
                for case in cases {
                    self.analyze_ast_node(&case.body, depth)?;
                }
            }
            
            ASTNode::Function { body, .. } => {
                self.analyze_ast_node(body, depth)?;
            }
            
            ASTNode::CommandSubstitution(cmd) | ASTNode::Subshell(cmd) => {
                self.analyze_ast_node(cmd, depth)?;
            }
            
            ASTNode::String(content, _) => {
                self.analyze_string_content(content)?;
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    fn analyze_command(&mut self, name: &str, args: &[Box<ASTNode>], depth: usize) -> Result<()> {
        // Check if it's an external command
        if !is_shell_builtin(name) {
            self.add_dependency(
                PathBuf::from(name),
                DependencyType::BinaryCommand,
                FileUsage::default(),
                vec![],
            );
        }
        
        // Special handling for specific commands
        match name {
            "source" | "." => {
                if let Some(first_arg) = args.first() {
                    if let ASTNode::String(path, _) = first_arg.as_ref() {
                        self.handle_source_command(path, depth)?;
                    }
                }
            }
            
            "cat" | "less" | "more" | "head" | "tail" => {
                for arg in args {
                    if let ASTNode::String(path, _) = arg.as_ref() {
                        let mut usage = FileUsage::default();
                        usage.read_count += 1;
                        if name == "tail" && args.iter().any(|a| {
                            matches!(a.as_ref(), ASTNode::String(s, _) if s == "-f")
                        }) {
                            usage.is_monitored = true;
                        }
                        self.add_file_dependency(path, usage)?;
                    }
                }
            }
            
            "echo" | "printf" if args.len() >= 2 => {
                // Check for output redirection (handled elsewhere)
            }
            
            "cp" | "mv" if args.len() >= 2 => {
                // Source file(s)
                for arg in &args[..args.len()-1] {
                    if let ASTNode::String(path, _) = arg.as_ref() {
                        let mut usage = FileUsage::default();
                        usage.read_count += 1;
                        self.add_file_dependency(path, usage)?;
                    }
                }
            }
            
            "rm" | "unlink" => {
                for arg in args {
                    if let ASTNode::String(path, _) = arg.as_ref() {
                        let mut usage = FileUsage::default();
                        usage.write_count += 1; // Deletion counts as write
                        self.add_file_dependency(path, usage)?;
                    }
                }
            }
            
            "mkdir" => {
                for arg in args {
                    if let ASTNode::String(path, _) = arg.as_ref() {
                        self.add_dependency(
                            PathBuf::from(path),
                            DependencyType::Directory,
                            FileUsage::default(),
                            vec![],
                        );
                    }
                }
            }
            
            "curl" | "wget" => {
                self.analyze_network_command(args)?;
            }
            
            "git" => {
                // Git is a common external dependency
                self.add_dependency(
                    PathBuf::from("git"),
                    DependencyType::BinaryCommand,
                    FileUsage::default(),
                    vec![],
                );
            }
            
            _ => {}
        }
        
        // Analyze all arguments for potential file paths
        for arg in args {
            self.analyze_ast_node(arg, depth)?;
        }
        
        Ok(())
    }
    
    fn handle_source_command(&mut self, path: &str, depth: usize) -> Result<()> {
        let source_path = self.resolve_path(path);
        
        // Avoid circular dependencies
        if self.visited_sources.contains(&source_path) {
            return Ok(());
        }
        
        self.visited_sources.insert(source_path.clone());
        
        // Add as source dependency
        let mut usage = FileUsage::default();
        usage.is_sourced = true;
        self.add_dependency(
            source_path.clone(),
            DependencyType::SourceFile,
            usage,
            vec![],
        );
        
        // Parse and analyze the sourced file
        if source_path.exists() {
            let content = std::fs::read_to_string(&source_path)
                .context("Failed to read sourced file")?;
            
            // Detect dialect from sourced file
            let dialect = crate::parser::shell_dialect::ShellDialect::from_shebang(
                content.lines().next().unwrap_or("")
            );
            
            // Parse the sourced file
            if let Ok(mut parser) = crate::parser::ShellParser::new(content, dialect) {
                if let Ok(ast) = parser.parse() {
                    self.analyze_ast_node(&ast.root, depth + 1)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn analyze_string_content(&mut self, content: &str) -> Result<()> {
        // Look for file paths in the string
        for cap in FILE_PATH_REGEX.captures_iter(content) {
            if let Some(path_match) = cap.get(1) {
                let path = path_match.as_str();
                // Basic heuristic: if it looks like a real path, track it
                if path.contains('/') && !path.contains('*') && !path.contains('?') {
                    self.add_file_dependency(path, FileUsage::default())?;
                }
            }
        }
        
        // Look for URLs
        for url_match in URL_REGEX.find_iter(content) {
            self.add_dependency(
                PathBuf::from(url_match.as_str()),
                DependencyType::NetworkResource,
                FileUsage::default(),
                vec![],
            );
        }
        
        Ok(())
    }
    
    fn analyze_network_command(&mut self, args: &[Box<ASTNode>]) -> Result<()> {
        for arg in args {
            if let ASTNode::String(content, _) = arg.as_ref() {
                if URL_REGEX.is_match(content) {
                    self.add_dependency(
                        PathBuf::from(content),
                        DependencyType::NetworkResource,
                        FileUsage::default(),
                        vec![],
                    );
                }
            }
        }
        Ok(())
    }
    
    fn add_file_dependency(&mut self, path: &str, usage: FileUsage) -> Result<()> {
        let resolved_path = self.resolve_path(path);
        
        // Determine dependency type based on path and usage
        let dep_type = if path.ends_with(".conf") || path.ends_with(".config") {
            DependencyType::ConfigFile
        } else {
            DependencyType::DataFile
        };
        
        self.add_dependency(resolved_path, dep_type, usage, vec![]);
        Ok(())
    }
    
    fn add_dependency(
        &mut self,
        path: PathBuf,
        dep_type: DependencyType,
        usage: FileUsage,
        line_numbers: Vec<usize>,
    ) {
        self.dependencies
            .entry(path.clone())
            .and_modify(|dep| {
                // Merge usage information
                dep.usage.read_count += usage.read_count;
                dep.usage.write_count += usage.write_count;
                dep.usage.append_count += usage.append_count;
                dep.usage.in_loop |= usage.in_loop;
                dep.usage.in_condition |= usage.in_condition;
                dep.usage.is_sourced |= usage.is_sourced;
                dep.usage.is_monitored |= usage.is_monitored;
                dep.line_numbers.extend(&line_numbers);
            })
            .or_insert(Dependency {
                path,
                dep_type,
                usage,
                line_numbers,
            });
    }
    
    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        
        if path.is_absolute() {
            path
        } else {
            self.script_dir.join(path)
        }
    }
}

fn is_shell_builtin(command: &str) -> bool {
    matches!(command,
        "echo" | "printf" | "read" | "cd" | "pwd" | "exit" |
        "source" | "." | "exec" | "eval" | "export" | "unset" |
        "set" | "shift" | "return" | "break" | "continue" |
        "trap" | "wait" | "jobs" | "fg" | "bg" | "kill" |
        "test" | "[" | "[[" | "true" | "false" | ":" |
        "alias" | "unalias" | "type" | "which" | "command" |
        "builtin" | "declare" | "typeset" | "local" | "readonly" |
        "let" | "history" | "getopts" | "hash" | "help" |
        "logout" | "times" | "ulimit" | "umask"
    )
}