use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BuildTarget {
    pub triple: &'static str,
    pub os: &'static str,
    pub arch: &'static str,
    pub binary_name: String,
}

impl BuildTarget {
    pub fn all() -> Vec<Self> {
        vec![
            // Linux targets
            BuildTarget {
                triple: "x86_64-unknown-linux-gnu",
                os: "linux",
                arch: "amd64",
                binary_name: String::new(),
            },
            BuildTarget {
                triple: "aarch64-unknown-linux-gnu",
                os: "linux",
                arch: "arm64",
                binary_name: String::new(),
            },
            BuildTarget {
                triple: "armv7-unknown-linux-gnueabihf",
                os: "linux",
                arch: "armv7",
                binary_name: String::new(),
            },
            
            // macOS targets
            BuildTarget {
                triple: "x86_64-apple-darwin",
                os: "darwin",
                arch: "amd64",
                binary_name: String::new(),
            },
            BuildTarget {
                triple: "aarch64-apple-darwin",
                os: "darwin",
                arch: "arm64",
                binary_name: String::new(),
            },
            
            // Windows targets
            BuildTarget {
                triple: "x86_64-pc-windows-gnu",
                os: "windows",
                arch: "amd64",
                binary_name: String::new(),
            },
            BuildTarget {
                triple: "aarch64-pc-windows-gnu",
                os: "windows",
                arch: "arm64",
                binary_name: String::new(),
            },
            
            // BSD targets
            BuildTarget {
                triple: "x86_64-unknown-freebsd",
                os: "freebsd",
                arch: "amd64",
                binary_name: String::new(),
            },
        ]
    }
    
    pub fn from_config(targets: &[String]) -> Vec<Self> {
        let all_targets = Self::all();
        let mut selected = Vec::new();
        
        for target_str in targets {
            // Match by os_arch pattern (e.g., "linux_amd64")
            if let Some(target) = all_targets.iter().find(|t| {
                format!("{}_{}", t.os, t.arch) == *target_str
            }) {
                selected.push(target.clone());
            }
        }
        
        selected
    }
    
    pub fn binary_name(&mut self, base_name: &str) {
        self.binary_name = if self.os == "windows" {
            format!("{}_{}_{}.exe", base_name, self.os, self.arch)
        } else {
            format!("{}_{}_{}", base_name, self.os, self.arch)
        };
    }
}

pub struct CrossCompiler {
    project_dir: PathBuf,
    output_dir: PathBuf,
    release: bool,
    verbose: bool,
}

impl CrossCompiler {
    pub fn new(project_dir: PathBuf, output_dir: PathBuf, release: bool, verbose: bool) -> Self {
        Self {
            project_dir,
            output_dir,
            release,
            verbose,
        }
    }
    
    pub fn build_all(&self, targets: &mut [BuildTarget], base_name: &str) -> Result<()> {
        // Ensure output directory exists
        std::fs::create_dir_all(&self.output_dir)?;
        
        // Check if we have cargo and cross installed
        self.check_tools()?;
        
        println!("Building {} targets...", targets.len());
        
        for target in targets.iter_mut() {
            target.binary_name(base_name);
            
            println!("Building for {} ({})...", target.binary_name, target.triple);
            
            match self.build_target(target) {
                Ok(_) => {
                    println!("✓ Successfully built {}", target.binary_name);
                }
                Err(e) => {
                    eprintln!("✗ Failed to build {}: {}", target.binary_name, e);
                    if !self.should_continue_on_error() {
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn check_tools(&self) -> Result<()> {
        // Check for cargo
        let cargo_check = Command::new("cargo")
            .arg("--version")
            .output();
        
        if cargo_check.is_err() || !cargo_check.unwrap().status.success() {
            bail!("Cargo not found. Please install Rust toolchain.");
        }
        
        // Check for cross (optional but recommended)
        let cross_check = Command::new("cross")
            .arg("--version")
            .output();
        
        if cross_check.is_err() || !cross_check.unwrap().status.success() {
            eprintln!("Warning: 'cross' not found. Install it for better cross-compilation:");
            eprintln!("  cargo install cross");
            eprintln!("Falling back to cargo with manual target installation.");
        }
        
        Ok(())
    }
    
    fn build_target(&self, target: &BuildTarget) -> Result<()> {
        // Determine which tool to use
        let use_cross = self.should_use_cross(target);
        let tool = if use_cross { "cross" } else { "cargo" };
        
        // Set environment variables for the build
        let mut envs = HashMap::new();
        
        // Add SCRIPT_REPO and RELEASE_API if they're set
        if let Ok(repo) = std::env::var("SCRIPT_REPO") {
            envs.insert("SCRIPT_REPO", repo);
        }
        if let Ok(api) = std::env::var("RELEASE_API") {
            envs.insert("RELEASE_API", api);
        }
        
        // Build command
        let mut cmd = Command::new(tool);
        cmd.current_dir(&self.project_dir);
        
        for (key, value) in &envs {
            cmd.env(key, value);
        }
        
        cmd.arg("build");
        cmd.arg("--target").arg(target.triple);
        
        if self.release {
            cmd.arg("--release");
        }
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        // Execute build
        let output = cmd.output()
            .with_context(|| format!("Failed to execute {} build", tool))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Build failed:\n{}", stderr);
        }
        
        // Copy the built binary to output directory
        self.copy_binary(target)?;
        
        // Optionally compress the binary
        if self.should_compress() {
            self.compress_binary(target)?;
        }
        
        Ok(())
    }
    
    fn should_use_cross(&self, target: &BuildTarget) -> bool {
        // Use cross for non-native targets if available
        let cross_available = Command::new("cross")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        
        if !cross_available {
            return false;
        }
        
        // Check if this is a native target
        let current_target = std::env::var("TARGET").unwrap_or_else(|_| {
            // Try to detect current target
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            return "x86_64-unknown-linux-gnu".to_string();
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            return "aarch64-unknown-linux-gnu".to_string();
            #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
            return "x86_64-apple-darwin".to_string();
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            return "aarch64-apple-darwin".to_string();
            #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
            return "x86_64-pc-windows-msvc".to_string();
            
            String::new()
        });
        
        target.triple != current_target
    }
    
    fn copy_binary(&self, target: &BuildTarget) -> Result<()> {
        let profile = if self.release { "release" } else { "debug" };
        
        // Source binary path
        let src_binary = self.project_dir
            .join("target")
            .join(target.triple)
            .join(profile)
            .join(if target.os == "windows" {
                format!("{}.exe", self.project_dir.file_stem().unwrap().to_string_lossy())
            } else {
                self.project_dir.file_stem().unwrap().to_string_lossy().to_string()
            });
        
        // Destination path
        let dst_binary = self.output_dir.join(&target.binary_name);
        
        // Copy the binary
        std::fs::copy(&src_binary, &dst_binary)
            .with_context(|| format!(
                "Failed to copy binary from {} to {}",
                src_binary.display(),
                dst_binary.display()
            ))?;
        
        // Make it executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dst_binary)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dst_binary, perms)?;
        }
        
        Ok(())
    }
    
    fn compress_binary(&self, target: &BuildTarget) -> Result<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::fs::File;
        use std::io::Write;
        
        let binary_path = self.output_dir.join(&target.binary_name);
        let compressed_path = self.output_dir.join(format!("{}.gz", &target.binary_name));
        
        let input = std::fs::read(&binary_path)?;
        let output = File::create(&compressed_path)?;
        let mut encoder = GzEncoder::new(output, Compression::best());
        encoder.write_all(&input)?;
        encoder.finish()?;
        
        // Calculate compression ratio
        let original_size = input.len();
        let compressed_size = std::fs::metadata(&compressed_path)?.len() as usize;
        let ratio = 100.0 - (compressed_size as f64 / original_size as f64 * 100.0);
        
        println!("  Compressed {} ({:.1}% reduction)", 
            compressed_path.file_name().unwrap().to_string_lossy(),
            ratio
        );
        
        Ok(())
    }
    
    fn should_compress(&self) -> bool {
        // TODO: Read from config
        false
    }
    
    fn should_continue_on_error(&self) -> bool {
        // Continue building other targets even if one fails
        true
    }
}

pub fn get_host_target() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-unknown-linux-gnu";
    
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "aarch64-unknown-linux-gnu";
    
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "x86_64-apple-darwin";
    
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-apple-darwin";
    
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "x86_64-pc-windows-msvc";
    
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    return "unknown";
}