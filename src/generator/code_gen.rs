use crate::parser::{AST, ASTNode, ast::*};
use crate::resolver::{DependencyResolver, FileClassification, TerminalDetector, TerminalRequirement};
use super::rust_project::{RustProject, CrateDependency};
use anyhow::{Result, Context};
use std::collections::HashMap;

pub struct CodeGenerator {
    ast: AST,
    project: RustProject,
    indent_level: usize,
    variables: HashMap<String, String>,
    functions: HashMap<String, String>,
}

impl CodeGenerator {
    pub fn new(ast: AST, script_name: &str) -> Self {
        let mut project = RustProject::new(script_name);
        
        // Set metadata from AST
        if let Some(version) = &ast.metadata.version {
            project.version = version.clone();
        }
        if let Some(author) = &ast.metadata.author {
            project.author = author.clone();
        }
        if let Some(description) = &ast.metadata.description {
            project.description = description.clone();
        }
        
        Self {
            ast,
            project,
            indent_level: 0,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
    
    pub fn generate(mut self) -> Result<RustProject> {
        // Analyze terminal requirements
        let terminal_analysis = TerminalDetector::analyze(&self.ast);
        
        // Add required terminal crates based on analysis
        for (crate_name, version) in terminal_analysis.get_required_crates() {
            self.project.add_dependency(CrateDependency::new(crate_name, version));
        }
        
        // Generate main.rs with terminal support
        let main_content = self.generate_main_with_terminal(&terminal_analysis)?;
        self.project.add_file("src/main.rs".into(), main_content);
        
        // Generate config.rs
        let config_content = self.generate_config()?;
        self.project.add_file("src/config.rs".into(), config_content);
        
        // Generate shell_runtime.rs with terminal support
        let runtime_content = self.generate_shell_runtime_with_terminal(&terminal_analysis)?;
        self.project.add_file("src/shell_runtime.rs".into(), runtime_content);
        
        // Generate embedded_files.rs if needed
        if !self.project.embedded_files.is_empty() {
            let embedded_content = self.generate_embedded_files()?;
            self.project.add_file("src/embedded_files.rs".into(), embedded_content);
        }
        
        // Generate command implementations
        let commands_content = self.generate_commands()?;
        self.project.add_file("src/commands/mod.rs".into(), commands_content);
        
        // Generate UI module
        let ui_content = self.generate_ui()?;
        self.project.add_file("src/ui/mod.rs".into(), ui_content);
        
        // Generate terminal module if needed
        if terminal_analysis.needs_terminal() {
            let terminal_content = self.generate_terminal_module(&terminal_analysis)?;
            self.project.add_file("src/terminal/mod.rs".into(), terminal_content);
        }
        
        Ok(self.project)
    }
    
    fn generate_main(&mut self) -> Result<String> {
        let mut code = String::new();
        
        // Headers
        code.push_str("mod config;\n");
        code.push_str("mod shell_runtime;\n");
        code.push_str("mod commands;\n");
        code.push_str("mod ui;\n");
        
        if !self.project.embedded_files.is_empty() {
            code.push_str("mod embedded_files;\n");
        }
        
        code.push_str("\n");
        code.push_str("use anyhow::{Result, Context};\n");
        code.push_str("use clap::Parser;\n");
        code.push_str("use std::env;\n");
        code.push_str("use std::path::PathBuf;\n");
        code.push_str("\n");
        
        // CLI structure
        code.push_str("#[derive(Parser, Debug)]\n");
        code.push_str("#[command(version, about, long_about = None)]\n");
        code.push_str("struct Args {\n");
        code.push_str("    /// Arguments passed to the script\n");
        code.push_str("    #[arg(trailing_var_arg = true)]\n");
        code.push_str("    args: Vec<String>,\n");
        code.push_str("}\n\n");
        
        // Update check function
        if self.project.update_config.enabled {
            code.push_str(self.generate_update_check()?);
        }
        
        // Main function
        code.push_str("fn main() -> Result<()> {\n");
        code.push_str("    let args = Args::parse();\n");
        code.push_str("    let config = config::load_config()?;\n");
        code.push_str("    \n");
        
        if self.project.update_config.enabled {
            code.push_str("    // Check for updates if enabled\n");
            code.push_str("    if config.updates.enabled && config.updates.check_on_start {\n");
            code.push_str("        if let Ok(Some(new_version)) = check_updates() {\n");
            code.push_str("            println!(\"Update available: {}\", new_version);\n");
            code.push_str("        }\n");
            code.push_str("    }\n");
            code.push_str("    \n");
        }
        
        code.push_str("    // Initialize shell runtime\n");
        code.push_str("    let mut runtime = shell_runtime::ShellRuntime::new(args.args)?;\n");
        code.push_str("    \n");
        
        // Generate the main script logic
        code.push_str("    // Execute main script\n");
        code.push_str("    script_main(&mut runtime)?;\n");
        code.push_str("    \n");
        code.push_str("    Ok(())\n");
        code.push_str("}\n\n");
        
        // Generate script_main function
        code.push_str("fn script_main(runtime: &mut shell_runtime::ShellRuntime) -> Result<()> {\n");
        self.indent_level = 1;
        
        // Generate code for the AST
        let script_code = self.generate_node(&self.ast.root)?;
        code.push_str(&script_code);
        
        self.indent_level = 0;
        code.push_str("    Ok(())\n");
        code.push_str("}\n");
        
        // Add any generated functions
        for (name, func_code) in &self.functions {
            code.push_str("\n");
            code.push_str(func_code);
        }
        
        Ok(code)
    }
    
    fn generate_node(&mut self, node: &ASTNode) -> Result<String> {
        match node {
            ASTNode::Script(statements) | ASTNode::Block(statements) => {
                let mut code = String::new();
                for stmt in statements {
                    code.push_str(&self.indent());
                    code.push_str(&self.generate_node(stmt)?);
                    if !code.ends_with('\n') {
                        code.push('\n');
                    }
                }
                Ok(code)
            }
            
            ASTNode::Command { name, args, .. } => {
                self.generate_command(name, args)
            }
            
            ASTNode::Assignment { name, value, export, .. } => {
                self.generate_assignment(name, value, *export)
            }
            
            ASTNode::If { condition, then_block, elif_blocks, else_block } => {
                self.generate_if(condition, then_block, elif_blocks, else_block)
            }
            
            ASTNode::While { condition, body } => {
                self.generate_while(condition, body)
            }
            
            ASTNode::For { variable, items, body } => {
                self.generate_for(variable, items, body)
            }
            
            ASTNode::Function { name, body } => {
                self.generate_function(name, body)
            }
            
            ASTNode::String(s, _) => {
                Ok(format!("\"{}\"", escape_string(s)))
            }
            
            ASTNode::Variable(name) => {
                Ok(format!("runtime.get_var(\"{}\")?", name))
            }
            
            ASTNode::Exit(code) => {
                if let Some(code) = code {
                    let code_str = self.generate_node(code)?;
                    Ok(format!("std::process::exit({});", code_str))
                } else {
                    Ok("std::process::exit(0);".to_string())
                }
            }
            
            ASTNode::Return(value) => {
                if let Some(val) = value {
                    let val_str = self.generate_node(val)?;
                    Ok(format!("return Ok({});", val_str))
                } else {
                    Ok("return Ok(());".to_string())
                }
            }
            
            _ => Ok(format!("// TODO: Generate code for {:?}", node)),
        }
    }
    
    fn generate_command(&mut self, name: &str, args: &[Box<ASTNode>]) -> Result<String> {
        match name {
            "echo" => {
                // Check for -e flag (enable escape sequences)
                let has_e_flag = args.first()
                    .and_then(|a| match a.as_ref() {
                        ASTNode::String(s, _) => Some(s == "-e"),
                        _ => None
                    })
                    .unwrap_or(false);
                
                let start_idx = if has_e_flag { 1 } else { 0 };
                let mut arg_strs = Vec::new();
                for arg in &args[start_idx..] {
                    arg_strs.push(self.generate_node(arg)?);
                }
                
                if has_e_flag {
                    // Handle color codes automatically based on terminal
                    Ok(format!("runtime.echo_with_colors(&[{}]);", arg_strs.join(", ")))
                } else {
                    Ok(format!("println!(\"{{}}\", {});", arg_strs.join(", ")))
                }
            }
            
            "cd" => {
                if let Some(dir) = args.first() {
                    let dir_str = self.generate_node(dir)?;
                    Ok(format!("runtime.change_dir({})?;", dir_str))
                } else {
                    Ok("runtime.change_dir_home()?;".to_string())
                }
            }
            
            "export" => {
                if let Some(arg) = args.first() {
                    if let ASTNode::String(assignment, _) = arg.as_ref() {
                        if let Some((var, val)) = assignment.split_once('=') {
                            Ok(format!("runtime.export_var(\"{}\", \"{}\")?;", var, val))
                        } else {
                            Ok(format!("runtime.export_var(\"{}\", \"\")?;", assignment))
                        }
                    } else {
                        Ok("// TODO: Handle complex export".to_string())
                    }
                } else {
                    Ok("// Export without args".to_string())
                }
            }
            
            "read" => {
                // Handle read command with automatic terminal detection
                let mut var_name = "REPLY";
                let mut prompt = "";
                let mut silent = false;
                
                let mut i = 0;
                while i < args.len() {
                    if let ASTNode::String(s, _) = args[i].as_ref() {
                        match s.as_str() {
                            "-s" => silent = true,
                            "-p" => {
                                if i + 1 < args.len() {
                                    if let ASTNode::String(p, _) = args[i + 1].as_ref() {
                                        prompt = p;
                                        i += 1;
                                    }
                                }
                            }
                            s if !s.starts_with('-') => var_name = s,
                            _ => {}
                        }
                    }
                    i += 1;
                }
                
                if silent {
                    Ok(format!(
                        "runtime.set_var(\"{}\", runtime.read_password(\"{}\")?)?;",
                        var_name, prompt
                    ))
                } else {
                    Ok(format!(
                        "runtime.set_var(\"{}\", runtime.read_input(\"{}\")?)?;",
                        var_name, prompt
                    ))
                }
            }
            
            "select" => {
                // Handle select command with automatic terminal detection
                let var_name = args.first()
                    .and_then(|a| match a.as_ref() {
                        ASTNode::String(s, _) => Some(s.as_str()),
                        _ => None
                    })
                    .unwrap_or("REPLY");
                
                Ok(format!(
                    "// TODO: Implement select menu for {}\n    // runtime.select_option(...)?;",
                    var_name
                ))
            }
            
            _ => {
                // External command
                let mut arg_strs = Vec::new();
                for arg in args {
                    arg_strs.push(self.generate_node(arg)?);
                }
                
                if is_builtin(name) {
                    Ok(format!("commands::{}(&[{}])?;", name, arg_strs.join(", ")))
                } else {
                    Ok(format!(
                        "runtime.execute_command(\"{}\", &[{}])?;",
                        name,
                        arg_strs.join(", ")
                    ))
                }
            }
        }
    }
    
    fn generate_assignment(&mut self, name: &str, value: &ASTNode, export: bool) -> Result<String> {
        let value_str = self.generate_node(value)?;
        if export {
            Ok(format!("runtime.export_var(\"{}\", {})?;", name, value_str))
        } else {
            Ok(format!("runtime.set_var(\"{}\", {})?;", name, value_str))
        }
    }
    
    fn generate_if(
        &mut self,
        condition: &ASTNode,
        then_block: &ASTNode,
        elif_blocks: &[(Box<ASTNode>, Box<ASTNode>)],
        else_block: &Option<Box<ASTNode>>,
    ) -> Result<String> {
        let mut code = String::new();
        
        // Generate condition evaluation
        let cond_str = self.generate_condition(condition)?;
        code.push_str(&format!("if {} {{\n", cond_str));
        
        self.indent_level += 1;
        code.push_str(&self.generate_node(then_block)?);
        self.indent_level -= 1;
        
        // elif blocks
        for (elif_cond, elif_body) in elif_blocks {
            let elif_cond_str = self.generate_condition(elif_cond)?;
            code.push_str(&format!("{}}} else if {} {{\n", self.indent(), elif_cond_str));
            self.indent_level += 1;
            code.push_str(&self.generate_node(elif_body)?);
            self.indent_level -= 1;
        }
        
        // else block
        if let Some(else_body) = else_block {
            code.push_str(&format!("{}}} else {{\n", self.indent()));
            self.indent_level += 1;
            code.push_str(&self.generate_node(else_body)?);
            self.indent_level -= 1;
        }
        
        code.push_str(&format!("{}}}", self.indent()));
        Ok(code)
    }
    
    fn generate_while(&mut self, condition: &ASTNode, body: &ASTNode) -> Result<String> {
        let mut code = String::new();
        let cond_str = self.generate_condition(condition)?;
        
        code.push_str(&format!("while {} {{\n", cond_str));
        self.indent_level += 1;
        code.push_str(&self.generate_node(body)?);
        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));
        
        Ok(code)
    }
    
    fn generate_for(&mut self, variable: &str, items: &ForItems, body: &ASTNode) -> Result<String> {
        let mut code = String::new();
        
        match items {
            ForItems::List(list) => {
                code.push_str(&format!("for {} in &[", variable));
                let mut item_strs = Vec::new();
                for item in list {
                    item_strs.push(self.generate_node(item)?);
                }
                code.push_str(&item_strs.join(", "));
                code.push_str("] {\n");
                
                self.indent_level += 1;
                code.push_str(&format!("{}runtime.set_var(\"{}\", {})?;\n", self.indent(), variable, variable));
                code.push_str(&self.generate_node(body)?);
                self.indent_level -= 1;
                
                code.push_str(&format!("{}}}", self.indent()));
            }
            _ => {
                code.push_str("// TODO: Complex for loop");
            }
        }
        
        Ok(code)
    }
    
    fn generate_function(&mut self, name: &str, body: &ASTNode) -> Result<String> {
        let mut func_code = format!("fn shell_func_{}(runtime: &mut shell_runtime::ShellRuntime, args: &[String]) -> Result<()> {{\n", name);
        
        self.indent_level = 1;
        func_code.push_str(&self.generate_node(body)?);
        self.indent_level = 0;
        
        func_code.push_str("    Ok(())\n");
        func_code.push_str("}\n");
        
        self.functions.insert(name.to_string(), func_code);
        
        // Register the function in runtime
        Ok(format!("runtime.register_function(\"{}\", shell_func_{});", name, name))
    }
    
    fn generate_condition(&mut self, condition: &ASTNode) -> Result<String> {
        // For now, generate a simple command execution check
        match condition {
            ASTNode::Command { name, args, .. } if name == "test" || name == "[" => {
                // Handle test command
                if args.len() >= 3 {
                    // Simple file test
                    if let (Some(flag), Some(path)) = (args.get(0), args.get(1)) {
                        if let (ASTNode::String(flag_str, _), ASTNode::String(path_str, _)) = 
                            (flag.as_ref(), path.as_ref()) {
                            match flag_str.as_str() {
                                "-f" => return Ok(format!("std::path::Path::new(\"{}\").is_file()", path_str)),
                                "-d" => return Ok(format!("std::path::Path::new(\"{}\").is_dir()", path_str)),
                                "-e" => return Ok(format!("std::path::Path::new(\"{}\").exists()", path_str)),
                                _ => {}
                            }
                        }
                    }
                }
                Ok("true /* TODO: Complex test */".to_string())
            }
            _ => {
                // Execute command and check exit status
                Ok("runtime.last_exit_status() == 0".to_string())
            }
        }
    }
    
    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
    
    fn generate_update_check(&self) -> Result<&str> {
        Ok(r#"
const SCRIPT_REPO: &str = env!("SCRIPT_REPO", "");
const RELEASE_API: &str = env!("RELEASE_API", "");

fn check_updates() -> Result<Option<String>> {
    if SCRIPT_REPO.is_empty() || 
       matches!(SCRIPT_REPO, "null" | "nil" | "none") {
        return Ok(None);
    }
    
    // TODO: Implement update checking
    Ok(None)
}
"#)
    }
    
    fn generate_config(&self) -> Result<String> {
        Ok(r#"use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub ui: UiConfig,
    pub build: BuildConfig,
    pub notifications: NotificationConfig,
    pub updates: UpdateConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiConfig {
    pub theme: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildConfig {
    pub targets: Vec<String>,
    pub optimize: String,
    pub strip: bool,
    pub compress: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub desktop: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateConfig {
    pub enabled: bool,
    pub check_on_start: bool,
    pub auto_download: bool,
}

pub fn load_config() -> Result<Config> {
    let config_path = config_path()?;
    
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        toml::from_str(&content)
            .context("Failed to parse config file")
    } else {
        Ok(default_config())
    }
}

fn config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("CONFIG_PATH") {
        Ok(PathBuf::from(path))
    } else {
        Ok(PathBuf::from("settings.toml"))
    }
}

fn default_config() -> Config {
    Config {
        ui: UiConfig {
            theme: "dracula".to_string(),
        },
        build: BuildConfig {
            targets: vec!["linux_amd64".to_string()],
            optimize: "size".to_string(),
            strip: true,
            compress: true,
        },
        notifications: NotificationConfig {
            enabled: true,
            desktop: true,
        },
        updates: UpdateConfig {
            enabled: false,
            check_on_start: false,
            auto_download: false,
        },
    }
}
"#.to_string())
    }
    
    fn generate_shell_runtime(&self) -> Result<String> {
        Ok(r#"use anyhow::{Result, Context};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub struct ShellRuntime {
    variables: HashMap<String, String>,
    functions: HashMap<String, fn(&mut ShellRuntime, &[String]) -> Result<()>>,
    args: Vec<String>,
    last_exit_status: i32,
    current_dir: PathBuf,
}

impl ShellRuntime {
    pub fn new(args: Vec<String>) -> Result<Self> {
        let current_dir = env::current_dir()?;
        
        let mut runtime = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            args,
            last_exit_status: 0,
            current_dir,
        };
        
        // Initialize environment variables
        for (key, value) in env::vars() {
            runtime.variables.insert(key, value);
        }
        
        // Set positional parameters
        for (i, arg) in runtime.args.iter().enumerate() {
            runtime.variables.insert(i.to_string(), arg.clone());
        }
        runtime.variables.insert("#".to_string(), runtime.args.len().to_string());
        
        Ok(runtime)
    }
    
    pub fn get_var(&self, name: &str) -> Result<String> {
        Ok(self.variables.get(name).cloned().unwrap_or_default())
    }
    
    pub fn set_var(&mut self, name: &str, value: impl Into<String>) -> Result<()> {
        self.variables.insert(name.to_string(), value.into());
        Ok(())
    }
    
    pub fn export_var(&mut self, name: &str, value: impl Into<String>) -> Result<()> {
        let value = value.into();
        self.variables.insert(name.to_string(), value.clone());
        env::set_var(name, value);
        Ok(())
    }
    
    pub fn change_dir(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let path = path.as_ref();
        env::set_current_dir(path)
            .context("Failed to change directory")?;
        self.current_dir = env::current_dir()?;
        Ok(())
    }
    
    pub fn change_dir_home(&mut self) -> Result<()> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;
        self.change_dir(home)
    }
    
    pub fn execute_command(&mut self, cmd: &str, args: &[impl AsRef<str>]) -> Result<()> {
        let output = Command::new(cmd)
            .args(args.iter().map(|s| s.as_ref()))
            .output()
            .context("Failed to execute command")?;
        
        self.last_exit_status = output.status.code().unwrap_or(-1);
        
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        
        Ok(())
    }
    
    pub fn last_exit_status(&self) -> i32 {
        self.last_exit_status
    }
    
    pub fn register_function(&mut self, name: &str, func: fn(&mut ShellRuntime, &[String]) -> Result<()>) {
        self.functions.insert(name.to_string(), func);
    }
}
"#.to_string())
    }
    
    fn generate_embedded_files(&self) -> Result<String> {
        Ok(r#"// Include auto-generated embedded files
include!(concat!(env!("OUT_DIR"), "/embedded_files.rs"));
"#.to_string())
    }
    
    fn generate_commands(&self) -> Result<String> {
        Ok(r#"use anyhow::Result;

pub fn pwd(_args: &[&str]) -> Result<()> {
    println!("{}", std::env::current_dir()?.display());
    Ok(())
}

pub fn true_cmd(_args: &[&str]) -> Result<()> {
    Ok(())
}

pub fn false_cmd(_args: &[&str]) -> Result<()> {
    std::process::exit(1);
}
"#.to_string())
    }
    
    fn generate_ui(&self) -> Result<String> {
        Ok(r#"pub mod theme;

pub use theme::Theme;
"#.to_string())
    }
}

fn escape_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            c => vec![c],
        })
        .collect()
}

fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "pwd" | "true" | "false")
}

impl CodeGenerator {
    fn generate_main_with_terminal(&mut self, terminal_analysis: &crate::resolver::TerminalAnalysis) -> Result<String> {
        let mut code = String::new();
        
        // Headers
        code.push_str("mod config;\n");
        code.push_str("mod shell_runtime;\n");
        code.push_str("mod commands;\n");
        code.push_str("mod ui;\n");
        
        if terminal_analysis.needs_terminal() {
            code.push_str("mod terminal;\n");
        }
        
        if !self.project.embedded_files.is_empty() {
            code.push_str("mod embedded_files;\n");
        }
        
        code.push_str("\n");
        code.push_str("use anyhow::{Result, Context};\n");
        code.push_str("use clap::Parser;\n");
        code.push_str("use std::env;\n");
        code.push_str("use std::path::PathBuf;\n");
        
        // Add terminal-specific imports
        if terminal_analysis.features_used.contains(&crate::resolver::TerminalFeature::ColorOutput) {
            code.push_str("use colored::*;\n");
        }
        if terminal_analysis.features_used.contains(&crate::resolver::TerminalFeature::UserInput) {
            code.push_str("use dialoguer::{Input, Password};\n");
        }
        
        // Add automatic terminal detection
        if terminal_analysis.needs_terminal() {
            code.push_str("use std::io::IsTerminal;\n");
        }
        
        code.push_str("\n");
        
        // CLI structure without headless flag - automatic detection
        code.push_str("#[derive(Parser, Debug)]\n");
        code.push_str("#[command(version, about, long_about = None)]\n");
        code.push_str("struct Args {\n");
        code.push_str("    /// Arguments passed to the script\n");
        code.push_str("    #[arg(trailing_var_arg = true)]\n");
        code.push_str("    args: Vec<String>,\n");
        code.push_str("}\n\n");
        
        // Update check function
        if self.project.update_config.enabled {
            code.push_str(self.generate_update_check()?);
        }
        
        // Main function
        code.push_str("fn main() -> Result<()> {\n");
        code.push_str("    let args = Args::parse();\n");
        code.push_str("    let config = config::load_config()?;\n");
        
        // Automatic terminal detection
        if terminal_analysis.needs_terminal() {
            code.push_str("    \n");
            code.push_str("    // Automatic terminal detection\n");
            code.push_str("    let is_terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();\n");
            code.push_str("    \n");
            
            // Add GUI detection for double-click launches
            code.push_str("    // Check if launched from GUI (no terminal)\n");
            code.push_str("    if !is_terminal && std::env::var(\"TERM\").is_err() {\n");
            code.push_str("        // Likely launched from file manager\n");
            code.push_str("        #[cfg(target_os = \"macos\")]\n");
            code.push_str("        {\n");
            code.push_str("            // Open in Terminal.app\n");
            code.push_str("            let exe = std::env::current_exe()?;\n");
            code.push_str("            std::process::Command::new(\"open\")\n");
            code.push_str("                .args(&[\"-a\", \"Terminal\", exe.to_str().unwrap()])\n");
            code.push_str("                .spawn()?;\n");
            code.push_str("            std::process::exit(0);\n");
            code.push_str("        }\n");
            code.push_str("        #[cfg(target_os = \"windows\")]\n");
            code.push_str("        {\n");
            code.push_str("            // Relaunch in cmd.exe\n");
            code.push_str("            let exe = std::env::current_exe()?;\n");
            code.push_str("            std::process::Command::new(\"cmd\")\n");
            code.push_str("                .args(&[\"/k\", exe.to_str().unwrap()])\n");
            code.push_str("                .spawn()?;\n");
            code.push_str("            std::process::exit(0);\n");
            code.push_str("        }\n");
            code.push_str("        #[cfg(target_os = \"linux\")]\n");
            code.push_str("        {\n");
            code.push_str("            // Try common terminal emulators\n");
            code.push_str("            let exe = std::env::current_exe()?;\n");
            code.push_str("            let terminals = [\"gnome-terminal\", \"konsole\", \"xfce4-terminal\", \"xterm\"];\n");
            code.push_str("            for term in &terminals {\n");
            code.push_str("                if std::process::Command::new(\"which\")\n");
            code.push_str("                    .arg(term)\n");
            code.push_str("                    .output()\n");
            code.push_str("                    .map(|o| o.status.success())\n");
            code.push_str("                    .unwrap_or(false)\n");
            code.push_str("                {\n");
            code.push_str("                    std::process::Command::new(term)\n");
            code.push_str("                        .arg(\"--\")\n");
            code.push_str("                        .arg(exe.to_str().unwrap())\n");
            code.push_str("                        .spawn()?;\n");
            code.push_str("                    std::process::exit(0);\n");
            code.push_str("                }\n");
            code.push_str("            }\n");
            code.push_str("        }\n");
            code.push_str("    }\n");
            code.push_str("    \n");
            
            match terminal_analysis.requirement {
                TerminalRequirement::Interactive => {
                    code.push_str("    if !is_terminal {\n");
                    code.push_str("        // Running in non-interactive mode (pipe, redirect, etc.)\n");
                    code.push_str("        // Automatically switch to headless behavior\n");
                    code.push_str("    }\n");
                }
                TerminalRequirement::FullTUI => {
                    code.push_str("    if !is_terminal {\n");
                    code.push_str("        eprintln!(\"Error: This script requires a terminal interface\");\n");
                    code.push_str("        eprintln!(\"It appears you're running in a non-interactive environment (pipe/redirect)\");\n");
                    code.push_str("        std::process::exit(1);\n");
                    code.push_str("    }\n");
                }
                _ => {}
            }
        }
        
        code.push_str("    \n");
        
        if self.project.update_config.enabled {
            code.push_str("    // Check for updates if enabled\n");
            code.push_str("    if config.updates.enabled && config.updates.check_on_start {\n");
            code.push_str("        if let Ok(Some(new_version)) = check_updates() {\n");
            code.push_str("            println!(\"Update available: {}\", new_version);\n");
            code.push_str("        }\n");
            code.push_str("    }\n");
            code.push_str("    \n");
        }
        
        code.push_str("    // Initialize shell runtime\n");
        code.push_str("    let mut runtime = shell_runtime::ShellRuntime::new(args.args)?;\n");
        if terminal_analysis.needs_terminal() {
            code.push_str("    runtime.set_terminal_mode(is_terminal);\n");
        }
        code.push_str("    \n");
        
        // Initialize terminal if needed
        if terminal_analysis.features_used.contains(&crate::resolver::TerminalFeature::AlternateScreen) {
            code.push_str("    // Initialize terminal\n");
            code.push_str("    let _terminal = terminal::init()?;\n");
            code.push_str("    \n");
        }
        
        // Generate the main script logic
        code.push_str("    // Execute main script\n");
        code.push_str("    script_main(&mut runtime)?;\n");
        code.push_str("    \n");
        code.push_str("    Ok(())\n");
        code.push_str("}\n\n");
        
        // Generate script_main function
        code.push_str("fn script_main(runtime: &mut shell_runtime::ShellRuntime) -> Result<()> {\n");
        self.indent_level = 1;
        
        // Generate code for the AST
        let script_code = self.generate_node(&self.ast.root)?;
        code.push_str(&script_code);
        
        self.indent_level = 0;
        code.push_str("    Ok(())\n");
        code.push_str("}\n");
        
        // Add any generated functions
        for (name, func_code) in &self.functions {
            code.push_str("\n");
            code.push_str(func_code);
        }
        
        Ok(code)
    }
    
    fn generate_shell_runtime_with_terminal(&self, terminal_analysis: &crate::resolver::TerminalAnalysis) -> Result<String> {
        let mut code = self.generate_shell_runtime()?;
        
        // Add terminal support to the runtime
        if terminal_analysis.needs_terminal() {
            let terminal_code = r#"
    is_terminal: bool,
}

impl ShellRuntime {
    pub fn set_terminal_mode(&mut self, is_terminal: bool) {
        self.is_terminal = is_terminal;
    }
    
    pub fn is_interactive(&self) -> bool {
        self.is_terminal
    }
    
    pub fn read_input(&mut self, prompt: &str) -> Result<String> {
        if !self.is_terminal {
            // Non-interactive mode: read from stdin without prompt
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            Ok(lines.next().unwrap_or_else(|| Ok(String::new()))?)
        } else {
            // Interactive mode: use dialoguer for nice prompts
            let input: String = dialoguer::Input::new()
                .with_prompt(prompt)
                .interact_text()?;
            Ok(input)
        }
    }
    
    pub fn read_password(&mut self, prompt: &str) -> Result<String> {
        if !self.is_terminal {
            // Non-interactive mode: read from stdin (no masking)
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            Ok(lines.next().unwrap_or_else(|| Ok(String::new()))?)
        } else {
            // Interactive mode: use password masking
            let password = dialoguer::Password::new()
                .with_prompt(prompt)
                .interact()?;
            Ok(password)
        }
    }
    
    pub fn select_option(&mut self, prompt: &str, items: &[&str]) -> Result<usize> {
        if !self.is_terminal {
            // Non-interactive mode: read index from stdin
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            if let Some(Ok(line)) = lines.next() {
                if let Ok(index) = line.trim().parse::<usize>() {
                    if index > 0 && index <= items.len() {
                        return Ok(index - 1);
                    }
                }
            }
            Ok(0) // Default to first option
        } else {
            // Interactive mode: use dialoguer for menu
            use dialoguer::Select;
            let selection = Select::new()
                .with_prompt(prompt)
                .items(items)
                .default(0)
                .interact()?;
            Ok(selection)
        }
    }
    
    pub fn print_colored(&self, text: &str, color: &str) {
        if self.is_terminal {
            // Terminal supports colors
            use colored::*;
            let colored_text = match color {
                "red" => text.red(),
                "green" => text.green(),
                "blue" => text.blue(),
                "yellow" => text.yellow(),
                _ => text.normal(),
            };
            println!("{}", colored_text);
        } else {
            // No terminal or redirected - plain text
            println!("{}", text);
        }
    }
    
    pub fn echo_with_colors(&self, args: &[&str]) {
        let text = args.join(" ");
        if self.is_terminal {
            // Process ANSI escape sequences
            println!("{}", text);
        } else {
            // Strip ANSI codes for non-terminal output
            let clean = strip_ansi_codes(&text);
            println!("{}", clean);
        }
    }
}

fn strip_ansi_codes(text: &str) -> String {
    // Simple ANSI code stripper
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(text, "").to_string()"#;
            
            // Insert the is_terminal field and methods into the runtime
            code = code.replace(
                "    current_dir: PathBuf,\n}",
                &format!("    current_dir: PathBuf,\n{}", terminal_code)
            );
            
            // Update the new() function to detect terminal automatically
            code = code.replace(
                "            current_dir,\n        };",
                "            current_dir,\n            is_terminal: std::io::stdin().is_terminal() && std::io::stdout().is_terminal(),\n        };"
            );
            
            // Add the use statement for IsTerminal
            code = code.replace(
                "use anyhow::{Result, Context};\n",
                "use anyhow::{Result, Context};\nuse std::io::IsTerminal;\n"
            );
        }
        
        Ok(code)
    }
    
    fn generate_terminal_module(&self, terminal_analysis: &crate::resolver::TerminalAnalysis) -> Result<String> {
        let mut code = String::new();
        
        code.push_str("use anyhow::Result;\n");
        
        if terminal_analysis.features_used.contains(&crate::resolver::TerminalFeature::AlternateScreen) {
            code.push_str("use crossterm::{terminal::{EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};\n");
            code.push_str("use std::io::stdout;\n\n");
            
            code.push_str("pub struct Terminal;\n\n");
            
            code.push_str("impl Terminal {\n");
            code.push_str("    pub fn new() -> Result<Self> {\n");
            code.push_str("        stdout().execute(EnterAlternateScreen)?;\n");
            code.push_str("        Ok(Self)\n");
            code.push_str("    }\n");
            code.push_str("}\n\n");
            
            code.push_str("impl Drop for Terminal {\n");
            code.push_str("    fn drop(&mut self) {\n");
            code.push_str("        let _ = stdout().execute(LeaveAlternateScreen);\n");
            code.push_str("    }\n");
            code.push_str("}\n\n");
            
            code.push_str("pub fn init() -> Result<Terminal> {\n");
            code.push_str("    Terminal::new()\n");
            code.push_str("}\n");
        }
        
        if terminal_analysis.features_used.contains(&crate::resolver::TerminalFeature::CursorControl) {
            code.push_str("\npub mod cursor {\n");
            code.push_str("    use crossterm::{cursor::*, ExecutableCommand};\n");
            code.push_str("    use std::io::stdout;\n");
            code.push_str("    use anyhow::Result;\n\n");
            
            code.push_str("    pub fn move_to(x: u16, y: u16) -> Result<()> {\n");
            code.push_str("        stdout().execute(MoveTo(x, y))?;\n");
            code.push_str("        Ok(())\n");
            code.push_str("    }\n\n");
            
            code.push_str("    pub fn move_up(n: u16) -> Result<()> {\n");
            code.push_str("        stdout().execute(MoveUp(n))?;\n");
            code.push_str("        Ok(())\n");
            code.push_str("    }\n");
            code.push_str("}\n");
        }
        
        if terminal_analysis.features_used.contains(&crate::resolver::TerminalFeature::TerminalSize) {
            code.push_str("\npub fn size() -> Result<(u16, u16)> {\n");
            code.push_str("    let (cols, rows) = crossterm::terminal::size()?;\n");
            code.push_str("    Ok((cols, rows))\n");
            code.push_str("}\n");
        }
        
        Ok(code)
    }
}