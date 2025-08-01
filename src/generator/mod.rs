pub mod rust_project;
pub mod code_gen;

use crate::parser::AST;
use crate::cli::Args;
use crate::resolver::DependencyResolver;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub use rust_project::RustProject;
use code_gen::CodeGenerator;

pub struct RustGenerator {
    ast: AST,
    script_path: PathBuf,
    args: Args,
}

impl RustGenerator {
    pub fn new(ast: AST, args: &Args) -> Self {
        RustGenerator {
            ast,
            script_path: args.input.clone(),
            args: Args {
                input: args.input.clone(),
                build: args.build,
                wizard: args.wizard,
                output: args.output.clone(),
                config: args.config.clone(),
                verbose: args.verbose,
                quiet: args.quiet,
                dry_run: args.dry_run,
                secure: args.secure,
                watch: args.watch,
                sandbox: args.sandbox,
                join: args.join.clone(),
                release: args.release,
                enable_updates: args.enable_updates,
                update: args.update,
                command: args.command.clone(),
            },
        }
    }
    
    pub fn generate(self) -> Result<RustProject> {
        // Extract script name
        let script_name = self.script_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("script");
        
        // Create code generator
        let mut generator = CodeGenerator::new(self.ast.clone(), script_name);
        
        // Resolve dependencies
        let mut resolver = DependencyResolver::new(&self.script_path)?;
        let dependencies = resolver.resolve(&self.ast)?;
        
        if !self.args.quiet {
            println!("Found {} dependencies", dependencies.len());
            for dep in &dependencies {
                println!("  - {:?}: {}", dep.dep_type, dep.path.display());
            }
        }
        
        // Generate the Rust project
        let mut project = generator.generate()?;
        
        // Set update configuration based on environment or git
        if self.args.enable_updates {
            if let Ok(repo) = std::env::var("SCRIPT_REPO") {
                let api = std::env::var("RELEASE_API").ok();
                project.set_update_config(Some(repo), api);
            } else {
                // Try to detect from git
                if let Ok(repo_info) = detect_git_repo(&self.script_path) {
                    project.set_update_config(Some(repo_info), None);
                }
            }
        }
        
        Ok(project)
    }
}

fn detect_git_repo(script_path: &Path) -> Result<String> {
    use std::process::Command;
    
    let script_dir = script_path.parent().unwrap_or(Path::new("."));
    
    // Try to get remote origin URL
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(script_dir)
        .output()?;
    
    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Convert git URL to repo format
        // git@github.com:user/repo.git -> github.com/user/repo
        // https://github.com/user/repo.git -> github.com/user/repo
        if let Some(repo) = parse_git_url(&url) {
            return Ok(repo);
        }
    }
    
    anyhow::bail!("Not a git repository or no remote origin")
}

fn parse_git_url(url: &str) -> Option<String> {
    // Handle SSH format: git@github.com:user/repo.git
    if url.starts_with("git@") {
        let parts: Vec<&str> = url[4..].split(':').collect();
        if parts.len() == 2 {
            let domain = parts[0];
            let path = parts[1].trim_end_matches(".git");
            return Some(format!("{}/{}", domain, path));
        }
    }
    
    // Handle HTTPS format: https://github.com/user/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        let url = url.trim_end_matches(".git");
        let parts: Vec<&str> = url.splitn(3, '/').collect();
        if parts.len() == 3 {
            return Some(parts[2].to_string());
        }
    }
    
    None
}