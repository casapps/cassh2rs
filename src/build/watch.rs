use anyhow::{Result, Context};
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::cli::Args;

pub struct WatchMode {
    script_path: PathBuf,
    output_dir: PathBuf,
    args: Args,
}

impl WatchMode {
    pub fn new(script_path: PathBuf, output_dir: PathBuf, args: Args) -> Self {
        Self {
            script_path,
            output_dir,
            args,
        }
    }
    
    pub fn run(&self) -> Result<()> {
        println!("{}", "👁  Watch mode enabled".bold().blue());
        println!("Watching for changes in: {}", self.script_path.display());
        println!("Press Ctrl+C to stop\n");
        
        // Create a channel to receive events
        let (tx, rx) = channel();
        
        // Create a watcher with 1 second debounce
        let mut watcher = watcher(tx, Duration::from_secs(1))
            .context("Failed to create file watcher")?;
        
        // Watch the script file and its directory
        let watch_path = if self.script_path.is_file() {
            self.script_path.clone()
        } else {
            self.script_path.parent()
                .unwrap_or(&self.script_path)
                .to_path_buf()
        };
        
        watcher.watch(&watch_path, RecursiveMode::NonRecursive)
            .context("Failed to watch file")?;
        
        // Also watch for sourced files if detected
        let additional_paths = self.detect_sourced_files()?;
        for path in &additional_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::NonRecursive)
                    .context("Failed to watch sourced file")?;
                println!("Also watching: {}", path.display());
            }
        }
        
        // Initial build
        self.rebuild("Initial build")?;
        
        // Watch loop
        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::Write(path) |
                        DebouncedEvent::Create(path) |
                        DebouncedEvent::Rename(_, path) => {
                            if self.should_rebuild(&path) {
                                println!("\n{} {}", 
                                    "🔄".yellow(), 
                                    format!("File changed: {}", path.display()).yellow()
                                );
                                
                                if let Err(e) = self.rebuild("Rebuilding") {
                                    eprintln!("{} {}", "❌".red(), format!("Build failed: {}", e).red());
                                } else {
                                    println!("{} {}", "✅".green(), "Build successful!".green());
                                }
                            }
                        }
                        DebouncedEvent::Remove(path) => {
                            println!("\n{} {}", 
                                "⚠️ ".yellow(), 
                                format!("File removed: {}", path.display()).yellow()
                            );
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Watch error: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    fn should_rebuild(&self, changed_path: &Path) -> bool {
        // Check if the changed file is relevant
        if changed_path == self.script_path {
            return true;
        }
        
        // Check file extension
        if let Some(ext) = changed_path.extension() {
            let ext_str = ext.to_string_lossy();
            matches!(
                ext_str.as_ref(),
                "sh" | "bash" | "zsh" | "fish" | "ksh" | 
                "conf" | "config" | "toml" | "yaml" | "yml" | "json"
            )
        } else {
            // Files without extension might be scripts
            true
        }
    }
    
    fn rebuild(&self, message: &str) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        );
        spinner.set_message(message);
        spinner.enable_steady_tick(100);
        
        // Run the conversion
        let result = self.run_conversion();
        
        spinner.finish_and_clear();
        
        result
    }
    
    fn run_conversion(&self) -> Result<()> {
        use crate::parser::{ShellParser, shell_dialect::ShellDialect};
        use crate::generator::RustGenerator;
        use crate::resolver::DependencyResolver;
        use std::fs;
        
        // Read the script
        let content = fs::read_to_string(&self.script_path)
            .context("Failed to read script file")?;
        
        // Detect dialect
        let dialect = self.detect_dialect(&content);
        
        // Parse
        let mut parser = ShellParser::new(content, dialect)?;
        let ast = parser.parse()
            .context("Failed to parse script")?;
        
        // Generate Rust code
        let generator = RustGenerator::new(ast, &self.args);
        let rust_project = generator.generate()
            .context("Failed to generate Rust code")?;
        
        // Write to disk
        rust_project.write_to_disk(&self.output_dir)
            .context("Failed to write project files")?;
        
        // Build if requested
        if self.args.build {
            self.build_project()?;
        }
        
        Ok(())
    }
    
    fn build_project(&self) -> Result<()> {
        use std::process::Command;
        
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.output_dir);
        cmd.arg("build");
        
        if self.args.release {
            cmd.arg("--release");
        }
        
        let output = cmd.output()
            .context("Failed to run cargo build")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Cargo build failed:\n{}", stderr);
        }
        
        Ok(())
    }
    
    fn detect_dialect(&self, content: &str) -> crate::parser::shell_dialect::ShellDialect {
        use crate::parser::shell_dialect::ShellDialect;
        
        // Check shebang
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                return ShellDialect::from_shebang(first_line);
            }
        }
        
        // Check file extension
        if let Some(dialect) = ShellDialect::from_extension(&self.script_path) {
            return dialect;
        }
        
        ShellDialect::Bash
    }
    
    fn detect_sourced_files(&self) -> Result<Vec<PathBuf>> {
        use crate::parser::{ShellParser, ASTNode};
        use std::fs;
        
        let mut sourced_files = Vec::new();
        
        // Quick parse to find sourced files
        if let Ok(content) = fs::read_to_string(&self.script_path) {
            let dialect = self.detect_dialect(&content);
            if let Ok(mut parser) = ShellParser::new(content, dialect) {
                if let Ok(ast) = parser.parse() {
                    self.find_sourced_files_in_ast(&ast.root, &mut sourced_files);
                }
            }
        }
        
        Ok(sourced_files)
    }
    
    fn find_sourced_files_in_ast(&self, node: &crate::parser::ASTNode, files: &mut Vec<PathBuf>) {
        use crate::parser::ASTNode;
        
        match node {
            ASTNode::Script(statements) | ASTNode::Block(statements) => {
                for stmt in statements {
                    self.find_sourced_files_in_ast(stmt, files);
                }
            }
            ASTNode::Command { name, args, .. } if name == "source" || name == "." => {
                if let Some(first_arg) = args.first() {
                    if let ASTNode::String(path, _) = first_arg.as_ref() {
                        let source_path = if Path::new(path).is_relative() {
                            self.script_path.parent()
                                .unwrap_or(Path::new("."))
                                .join(path)
                        } else {
                            PathBuf::from(path)
                        };
                        
                        if !files.contains(&source_path) {
                            files.push(source_path);
                        }
                    }
                }
            }
            ASTNode::If { condition, then_block, elif_blocks, else_block } => {
                self.find_sourced_files_in_ast(condition, files);
                self.find_sourced_files_in_ast(then_block, files);
                for (cond, block) in elif_blocks {
                    self.find_sourced_files_in_ast(cond, files);
                    self.find_sourced_files_in_ast(block, files);
                }
                if let Some(block) = else_block {
                    self.find_sourced_files_in_ast(block, files);
                }
            }
            ASTNode::While { condition, body } | ASTNode::Until { condition, body } => {
                self.find_sourced_files_in_ast(condition, files);
                self.find_sourced_files_in_ast(body, files);
            }
            ASTNode::Function { body, .. } => {
                self.find_sourced_files_in_ast(body, files);
            }
            _ => {}
        }
    }
}