use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cassh2rs")]
#[command(version, about = "Universal shell script to Rust converter", long_about = None)]
pub struct Args {
    /// Input shell script or directory
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,
    
    /// Build binaries after generating Rust source
    #[arg(short, long)]
    pub build: bool,
    
    /// Interactive wizard for dependency resolution
    #[arg(short, long)]
    pub wizard: bool,
    
    /// Output directory (default: ./rustsrc/)
    #[arg(short, long, default_value = "rustsrc")]
    pub output: PathBuf,
    
    /// Configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Quiet mode (suppress output)
    #[arg(short, long)]
    pub quiet: bool,
    
    /// Dry run (show what would be done)
    #[arg(short = 'n', long)]
    pub dry_run: bool,
    
    /// Enable security mode
    #[arg(long)]
    pub secure: bool,
    
    /// Watch mode for development
    #[arg(long)]
    pub watch: bool,
    
    /// Sandbox execution
    #[arg(long)]
    pub sandbox: bool,
    
    /// Join multiple scripts into single app with subcommands
    #[arg(long, value_name = "PRIMARY")]
    pub join: Option<Option<String>>,
    
    /// Release build mode
    #[arg(long)]
    pub release: bool,
    
    /// Enable update checking in release builds
    #[arg(long)]
    pub enable_updates: bool,
    
    /// Check for updates
    #[arg(short = 'U', long)]
    pub update: bool,
    
    /// Generate GUI launcher (for double-click execution)
    #[arg(long)]
    pub launcher: bool,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Initialize a new cassh2rs project
    Init {
        /// Project name
        name: String,
    },
    
    /// Validate shell scripts without converting
    Check {
        /// Scripts to check
        scripts: Vec<PathBuf>,
    },
    
    /// Show supported shell features
    Features {
        /// Filter by shell dialect
        #[arg(long)]
        shell: Option<String>,
    },
}

pub fn run(args: Args) -> Result<()> {
    // Set up logging
    if args.verbose && !args.quiet {
        std::env::set_var("RUST_LOG", "debug");
    } else if !args.quiet {
        std::env::set_var("RUST_LOG", "info");
    }
    
    match args.command {
        Some(Commands::Init { name }) => {
            init_project(&name)?;
        }
        Some(Commands::Check { scripts }) => {
            check_scripts(&scripts)?;
        }
        Some(Commands::Features { shell }) => {
            show_features(shell.as_deref())?;
        }
        None => {
            // Main conversion flow
            if args.update {
                check_for_updates()?;
            } else {
                convert_scripts(&args)?;
            }
        }
    }
    
    Ok(())
}

fn init_project(name: &str) -> Result<()> {
    println!("Initializing new cassh2rs project: {}", name);
    // TODO: Create project structure
    Ok(())
}

fn check_scripts(scripts: &[PathBuf]) -> Result<()> {
    use crate::parser::{ShellParser, shell_dialect::ShellDialect};
    use crate::resolver::TerminalDetector;
    use std::fs;
    
    for script in scripts {
        println!("Checking {}...", script.display());
        
        let content = fs::read_to_string(script)
            .context("Failed to read script file")?;
        
        let dialect = detect_shell_dialect(&content, script);
        let mut parser = ShellParser::new(content, dialect)?;
        
        match parser.parse() {
            Ok(ast) => {
                println!("✓ {} - Valid {} script", script.display(), format!("{:?}", dialect));
                
                // Analyze terminal requirements
                let terminal_analysis = TerminalDetector::analyze(&ast);
                println!("  Terminal: {}", match terminal_analysis.requirement {
                    crate::resolver::TerminalRequirement::None => "Can run headless",
                    crate::resolver::TerminalRequirement::Interactive => "Requires terminal (interactive)",
                    crate::resolver::TerminalRequirement::TerminalFeatures => "Uses terminal features",
                    crate::resolver::TerminalRequirement::FullTUI => "Full terminal UI application",
                });
                
                if !terminal_analysis.interactive_commands.is_empty() {
                    println!("  Interactive commands: {}", terminal_analysis.interactive_commands.join(", "));
                }
                
                if !terminal_analysis.tui_indicators.is_empty() {
                    println!("  TUI programs: {}", terminal_analysis.tui_indicators.join(", "));
                }
                
                let deps = ast.root.get_dependencies();
                if !deps.is_empty() {
                    println!("  Dependencies: {}", deps.join(", "));
                }
            }
            Err(e) => {
                println!("✗ {} - Parse error: {}", script.display(), e);
            }
        }
    }
    
    Ok(())
}

fn show_features(shell: Option<&str>) -> Result<()> {
    use crate::parser::shell_dialect::{ShellDialect, ShellFeature};
    
    let dialects = if let Some(shell_name) = shell {
        vec![match shell_name.to_lowercase().as_str() {
            "bash" => ShellDialect::Bash,
            "zsh" => ShellDialect::Zsh,
            "fish" => ShellDialect::Fish,
            "dash" => ShellDialect::Dash,
            "ksh" => ShellDialect::Ksh,
            "tcsh" => ShellDialect::Tcsh,
            "csh" => ShellDialect::Csh,
            "powershell" | "pwsh" => ShellDialect::PowerShell,
            "posix" | "sh" => ShellDialect::Posix,
            _ => {
                eprintln!("Unknown shell: {}", shell_name);
                return Ok(());
            }
        }]
    } else {
        vec![
            ShellDialect::Bash,
            ShellDialect::Zsh,
            ShellDialect::Fish,
            ShellDialect::Dash,
            ShellDialect::Ksh,
            ShellDialect::Tcsh,
            ShellDialect::Csh,
            ShellDialect::PowerShell,
            ShellDialect::Posix,
        ]
    };
    
    let features = vec![
        ShellFeature::Arrays,
        ShellFeature::AssociativeArrays,
        ShellFeature::ProcessSubstitution,
        ShellFeature::ExtendedTest,
        ShellFeature::FunctionKeyword,
        ShellFeature::LocalKeyword,
        ShellFeature::SelectLoop,
    ];
    
    println!("Shell Feature Support Matrix:");
    println!("{:<20} {}", "Feature", dialects.iter().map(|d| format!("{:?}", d)).collect::<Vec<_>>().join(" "));
    println!("{}", "-".repeat(20 + dialects.len() * 10));
    
    for feature in features {
        print!("{:<20}", format!("{:?}", feature));
        for dialect in &dialects {
            print!(" {:^9}", if dialect.supports_feature(feature) { "✓" } else { "✗" });
        }
        println!();
    }
    
    Ok(())
}

fn check_for_updates() -> Result<()> {
    println!("Checking for updates...");
    // TODO: Implement update checking
    println!("cassh2rs is up to date!");
    Ok(())
}

fn convert_scripts(args: &Args) -> Result<()> {
    use std::fs;
    
    // Check if watch mode is enabled
    if args.watch {
        run_watch_mode(args)?;
    } else {
        if args.input.is_dir() {
            // Process directory
            if args.join.is_some() {
                convert_directory_joined(args)?;
            } else {
                convert_directory_separate(args)?;
            }
        } else {
            // Process single file
            convert_single_file(args)?;
        }
    }
    
    Ok(())
}

fn convert_single_file(args: &Args) -> Result<()> {
    use crate::parser::{ShellParser, shell_dialect::ShellDialect};
    use crate::generator::RustGenerator;
    use crate::resolver::DependencyResolver;
    use crate::ui::DependencyWizard;
    use std::fs;
    
    let content = fs::read_to_string(&args.input)
        .context("Failed to read script file")?;
    
    let dialect = detect_shell_dialect(&content, &args.input);
    
    if !args.quiet {
        println!("Converting {} ({:?} script)...", args.input.display(), dialect);
    }
    
    // Parse the script
    let mut parser = ShellParser::new(content, dialect)?;
    let ast = parser.parse()?;
    
    // Analyze terminal requirements
    let terminal_analysis = crate::resolver::TerminalDetector::analyze(&ast);
    if !args.quiet {
        println!("Terminal requirements: {}", match terminal_analysis.requirement {
            crate::resolver::TerminalRequirement::None => "None (can run headless)",
            crate::resolver::TerminalRequirement::Interactive => "Interactive terminal",
            crate::resolver::TerminalRequirement::TerminalFeatures => "Terminal features (colors, cursor)",
            crate::resolver::TerminalRequirement::FullTUI => "Full terminal UI",
        });
        
        if !terminal_analysis.get_required_crates().is_empty() {
            println!("Terminal crates needed: {}", 
                terminal_analysis.get_required_crates()
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    
    // Resolve dependencies if wizard mode is enabled
    if args.wizard {
        let mut resolver = DependencyResolver::new(&args.input)?;
        let dependencies = resolver.resolve(&ast)?;
        
        if !dependencies.is_empty() {
            let wizard = DependencyWizard::new();
            let resolved = wizard.resolve_dependencies(dependencies)?;
            
            // TODO: Apply resolved dependencies to the generator
            if !args.quiet {
                println!("\nResolved {} dependencies", 
                    resolved.embed_files.len() + 
                    resolved.runtime_files.len() + 
                    resolved.bundle_binaries.len()
                );
            }
        }
    }
    
    // Generate Rust code
    let generator = RustGenerator::new(ast, args);
    let rust_project = generator.generate()?;
    
    // Write output
    if !args.dry_run {
        rust_project.write_to_disk(&args.output)?;
        
        if !args.quiet {
            println!("✓ Generated Rust project in {}", args.output.display());
        }
        
        if args.build {
            build_project(&args.output, args)?;
        }
    }
    
    Ok(())
}

fn convert_directory_separate(args: &Args) -> Result<()> {
    // TODO: Implement directory conversion
    println!("Converting directory (separate projects)...");
    Ok(())
}

fn convert_directory_joined(args: &Args) -> Result<()> {
    // TODO: Implement joined directory conversion
    println!("Converting directory (joined with subcommands)...");
    Ok(())
}

fn build_project(project_dir: &PathBuf, args: &Args) -> Result<()> {
    use crate::build::{CrossCompiler, BuildTarget};
    use crate::generator::rust_project::RustProject;
    
    if !args.quiet {
        println!("Building project...");
    }
    
    // Load config to get build targets
    let config_path = project_dir.join("settings.toml");
    let config_content = std::fs::read_to_string(&config_path)
        .context("Failed to read settings.toml")?;
    let config: toml::Value = toml::from_str(&config_content)
        .context("Failed to parse settings.toml")?;
    
    // Get build targets from config
    let targets = config
        .get("build")
        .and_then(|b| b.get("targets"))
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["linux_amd64".to_string()]);
    
    // Get script name from Cargo.toml
    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml"))?;
    let cargo: toml::Value = toml::from_str(&cargo_toml)?;
    let script_name = cargo
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("script");
    
    // Create output directory
    let output_dir = if args.output.is_absolute() {
        args.output.join("dist")
    } else {
        project_dir.join("dist")
    };
    
    // Build for all targets
    let mut build_targets = BuildTarget::from_config(&targets);
    
    if build_targets.is_empty() {
        if !args.quiet {
            println!("No valid build targets found, building for host only");
        }
        // Build only for current platform
        use std::process::Command;
        
        let mut cmd = Command::new("cargo");
        cmd.current_dir(project_dir);
        cmd.arg("build");
        
        if args.release {
            cmd.arg("--release");
        }
        
        let status = cmd.status()
            .context("Failed to run cargo build")?;
        
        if !status.success() {
            anyhow::bail!("Build failed");
        }
    } else {
        let compiler = CrossCompiler::new(
            project_dir.clone(),
            output_dir,
            args.release,
            args.verbose,
        );
        
        compiler.build_all(&mut build_targets, script_name)?;
        
        if !args.quiet {
            println!("\n✓ Build complete! Binaries available in: {}", output_dir.display());
            for target in &build_targets {
                println!("  - {}", target.binary_name);
            }
        }
    }
    
    Ok(())
}

fn detect_shell_dialect(content: &str, path: &PathBuf) -> crate::parser::shell_dialect::ShellDialect {
    use crate::parser::shell_dialect::ShellDialect;
    
    // First check shebang
    if let Some(first_line) = content.lines().next() {
        if first_line.starts_with("#!") {
            return ShellDialect::from_shebang(first_line);
        }
    }
    
    // Then check file extension
    if let Some(dialect) = ShellDialect::from_extension(path) {
        return dialect;
    }
    
    // Default to bash
    ShellDialect::Bash
}

fn run_watch_mode(args: &Args) -> Result<()> {
    use crate::build::WatchMode;
    
    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {}", args.input.display());
    }
    
    if args.input.is_dir() {
        anyhow::bail!("Watch mode is not supported for directories. Please specify a single script file.");
    }
    
    let watch = WatchMode::new(
        args.input.clone(),
        args.output.clone(),
        args.clone(),
    );
    
    watch.run()
}