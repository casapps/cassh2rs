use anyhow::{Result, Context};
use dialoguer::{theme::ColorfulTheme, Select, MultiSelect, Input, Confirm};
use colored::*;
use crate::resolver::{Dependency, DependencyType, FileClassification};
use std::path::PathBuf;

pub struct DependencyWizard {
    theme: ColorfulTheme,
}

impl DependencyWizard {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }
    
    pub fn resolve_dependencies(&self, dependencies: Vec<Dependency>) -> Result<ResolvedDependencies> {
        println!("\n{}", "🔮 Dependency Resolution Wizard".bold().blue());
        println!("{}", "================================".blue());
        
        let mut resolved = ResolvedDependencies::default();
        
        // Group dependencies by type
        let mut file_deps = Vec::new();
        let mut binary_deps = Vec::new();
        let mut network_deps = Vec::new();
        let mut ambiguous_deps = Vec::new();
        
        for dep in dependencies {
            match dep.dep_type {
                DependencyType::DataFile | DependencyType::ConfigFile => {
                    if dep.path.exists() {
                        file_deps.push(dep);
                    } else {
                        ambiguous_deps.push(dep);
                    }
                }
                DependencyType::BinaryCommand => binary_deps.push(dep),
                DependencyType::NetworkResource => network_deps.push(dep),
                _ => ambiguous_deps.push(dep),
            }
        }
        
        // Resolve file dependencies
        if !file_deps.is_empty() {
            println!("\n{}", "📁 File Dependencies".bold());
            self.resolve_file_dependencies(&mut resolved, file_deps)?;
        }
        
        // Resolve binary dependencies
        if !binary_deps.is_empty() {
            println!("\n{}", "⚙️  Binary Dependencies".bold());
            self.resolve_binary_dependencies(&mut resolved, binary_deps)?;
        }
        
        // Resolve network resources
        if !network_deps.is_empty() {
            println!("\n{}", "🌐 Network Resources".bold());
            self.resolve_network_dependencies(&mut resolved, network_deps)?;
        }
        
        // Handle ambiguous dependencies
        if !ambiguous_deps.is_empty() {
            println!("\n{}", "❓ Ambiguous Dependencies".bold());
            self.resolve_ambiguous_dependencies(&mut resolved, ambiguous_deps)?;
        }
        
        // Security checks
        self.perform_security_checks(&mut resolved)?;
        
        println!("\n{}", "✅ Dependency resolution complete!".green());
        
        Ok(resolved)
    }
    
    fn resolve_file_dependencies(
        &self,
        resolved: &mut ResolvedDependencies,
        deps: Vec<Dependency>,
    ) -> Result<()> {
        println!("Found {} file dependencies:", deps.len());
        
        for dep in deps {
            let path_str = dep.path.display().to_string();
            let usage_info = format!(
                "Read: {}, Write: {}, Monitored: {}",
                dep.usage.read_count,
                dep.usage.write_count,
                if dep.usage.is_monitored { "Yes" } else { "No" }
            );
            
            println!("\n  {}", path_str.cyan());
            println!("  Usage: {}", usage_info.dimmed());
            
            let action = Select::with_theme(&self.theme)
                .with_prompt("How should this file be handled?")
                .items(&[
                    "Embed at compile time (static)",
                    "Access at runtime (dynamic)",
                    "Auto-detect based on usage",
                    "Skip this file",
                ])
                .default(2)
                .interact()?;
            
            match action {
                0 => resolved.embed_files.push(dep.path),
                1 => resolved.runtime_files.push(dep.path),
                2 => {
                    // Auto-detect based on usage
                    if dep.usage.is_monitored || dep.usage.write_count > 0 {
                        resolved.runtime_files.push(dep.path);
                    } else {
                        resolved.embed_files.push(dep.path);
                    }
                }
                _ => resolved.skip_files.push(dep.path),
            }
        }
        
        Ok(())
    }
    
    fn resolve_binary_dependencies(
        &self,
        resolved: &mut ResolvedDependencies,
        deps: Vec<Dependency>,
    ) -> Result<()> {
        let binary_names: Vec<String> = deps.iter()
            .map(|d| d.path.display().to_string())
            .collect();
        
        println!("Found {} external commands:", binary_names.len());
        for name in &binary_names {
            println!("  • {}", name.yellow());
        }
        
        let action = Select::with_theme(&self.theme)
            .with_prompt("How should these be handled?")
            .items(&[
                "Bundle binaries (increases size)",
                "Require system installation",
                "Select individually",
                "Use Rust alternatives where available",
            ])
            .default(3)
            .interact()?;
        
        match action {
            0 => {
                // Bundle all
                for dep in deps {
                    resolved.bundle_binaries.push(dep.path.display().to_string());
                }
            }
            1 => {
                // System dependencies
                for dep in deps {
                    resolved.system_deps.push(dep.path.display().to_string());
                }
            }
            2 => {
                // Select individually
                let selections = MultiSelect::with_theme(&self.theme)
                    .with_prompt("Select binaries to bundle (others will be system dependencies)")
                    .items(&binary_names)
                    .interact()?;
                
                for (i, dep) in deps.into_iter().enumerate() {
                    if selections.contains(&i) {
                        resolved.bundle_binaries.push(dep.path.display().to_string());
                    } else {
                        resolved.system_deps.push(dep.path.display().to_string());
                    }
                }
            }
            3 => {
                // Use Rust alternatives
                for dep in deps {
                    let name = dep.path.display().to_string();
                    if let Some(rust_alt) = get_rust_alternative(&name) {
                        resolved.rust_alternatives.insert(name, rust_alt);
                    } else {
                        resolved.system_deps.push(name);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn resolve_network_dependencies(
        &self,
        resolved: &mut ResolvedDependencies,
        deps: Vec<Dependency>,
    ) -> Result<()> {
        println!("Found {} network resources:", deps.len());
        
        for dep in deps {
            let url = dep.path.display().to_string();
            println!("\n  🔗 {}", url.blue());
            
            let action = Select::with_theme(&self.theme)
                .with_prompt("How should this URL be handled?")
                .items(&[
                    "Download and cache at build time",
                    "Download at runtime",
                    "Prompt user for local file",
                    "Skip",
                ])
                .default(1)
                .interact()?;
            
            match action {
                0 => resolved.cache_urls.push(url),
                1 => resolved.runtime_urls.push(url),
                2 => {
                    let local_path: String = Input::with_theme(&self.theme)
                        .with_prompt("Enter local file path")
                        .interact()?;
                    resolved.url_mappings.insert(url, PathBuf::from(local_path));
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn resolve_ambiguous_dependencies(
        &self,
        resolved: &mut ResolvedDependencies,
        deps: Vec<Dependency>,
    ) -> Result<()> {
        for dep in deps {
            println!("\n{} {}", "❓".yellow(), dep.path.display());
            println!("   Type: {:?}", dep.dep_type);
            
            if !dep.path.exists() {
                println!("   {} File not found", "⚠️ ".red());
                
                let action = Select::with_theme(&self.theme)
                    .with_prompt("How to handle missing file?")
                    .items(&[
                        "Create placeholder",
                        "Prompt for location",
                        "Mark as optional",
                        "Fail if missing",
                    ])
                    .default(2)
                    .interact()?;
                
                match action {
                    0 => resolved.create_placeholders.push(dep.path),
                    1 => {
                        let new_path: String = Input::with_theme(&self.theme)
                            .with_prompt("Enter correct path")
                            .interact()?;
                        resolved.path_mappings.insert(dep.path, PathBuf::from(new_path));
                    }
                    2 => resolved.optional_files.push(dep.path),
                    _ => resolved.required_files.push(dep.path),
                }
            }
        }
        
        Ok(())
    }
    
    fn perform_security_checks(&self, resolved: &mut ResolvedDependencies) -> Result<()> {
        println!("\n{}", "🔒 Security Checks".bold());
        
        // Check for curl|bash patterns
        if !resolved.runtime_urls.is_empty() {
            let has_curl_bash = resolved.runtime_urls.iter()
                .any(|url| url.contains("install.sh") || url.contains("get."));
            
            if has_curl_bash {
                println!("{} Detected potential remote code execution", "⚠️ ".red());
                
                let allow = Confirm::with_theme(&self.theme)
                    .with_prompt("Allow downloading and executing remote scripts?")
                    .default(false)
                    .interact()?;
                
                if !allow {
                    resolved.security_flags.block_remote_exec = true;
                }
            }
        }
        
        // Check for sensitive paths
        let sensitive_paths = ["/etc", "/root", "~/.ssh", "~/.gnupg"];
        for path in &resolved.runtime_files {
            for sensitive in &sensitive_paths {
                if path.starts_with(sensitive) {
                    println!("{} Access to sensitive path: {}", "⚠️ ".red(), path.display());
                    
                    let allow = Confirm::with_theme(&self.theme)
                        .with_prompt("Allow access to this sensitive location?")
                        .default(false)
                        .interact()?;
                    
                    if !allow {
                        resolved.blocked_paths.push(path.clone());
                    }
                    break;
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ResolvedDependencies {
    pub embed_files: Vec<PathBuf>,
    pub runtime_files: Vec<PathBuf>,
    pub skip_files: Vec<PathBuf>,
    pub bundle_binaries: Vec<String>,
    pub system_deps: Vec<String>,
    pub rust_alternatives: std::collections::HashMap<String, RustAlternative>,
    pub cache_urls: Vec<String>,
    pub runtime_urls: Vec<String>,
    pub url_mappings: std::collections::HashMap<String, PathBuf>,
    pub create_placeholders: Vec<PathBuf>,
    pub path_mappings: std::collections::HashMap<PathBuf, PathBuf>,
    pub optional_files: Vec<PathBuf>,
    pub required_files: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub security_flags: SecurityFlags,
}

#[derive(Debug, Default)]
pub struct SecurityFlags {
    pub block_remote_exec: bool,
    pub validate_paths: bool,
    pub sandbox_mode: bool,
}

#[derive(Debug, Clone)]
pub struct RustAlternative {
    pub crate_name: String,
    pub version: String,
    pub features: Vec<String>,
}

fn get_rust_alternative(binary: &str) -> Option<RustAlternative> {
    match binary {
        "git" => Some(RustAlternative {
            crate_name: "git2".to_string(),
            version: "0.18".to_string(),
            features: vec![],
        }),
        "curl" | "wget" => Some(RustAlternative {
            crate_name: "reqwest".to_string(),
            version: "0.11".to_string(),
            features: vec!["blocking".to_string()],
        }),
        "jq" => Some(RustAlternative {
            crate_name: "serde_json".to_string(),
            version: "1.0".to_string(),
            features: vec![],
        }),
        "sed" | "awk" => Some(RustAlternative {
            crate_name: "regex".to_string(),
            version: "1.10".to_string(),
            features: vec![],
        }),
        "tar" => Some(RustAlternative {
            crate_name: "tar".to_string(),
            version: "0.4".to_string(),
            features: vec![],
        }),
        "gzip" | "gunzip" => Some(RustAlternative {
            crate_name: "flate2".to_string(),
            version: "1.0".to_string(),
            features: vec![],
        }),
        _ => None,
    }
}