use crate::parser::{AST, ASTNode};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum TerminalRequirement {
    /// No terminal needed - can run headless
    None,
    /// Requires terminal for input/output
    Interactive,
    /// Uses terminal features (colors, cursor control)
    TerminalFeatures,
    /// Full TUI application
    FullTUI,
}

#[derive(Debug, Clone)]
pub struct TerminalAnalysis {
    pub requirement: TerminalRequirement,
    pub features_used: HashSet<TerminalFeature>,
    pub interactive_commands: Vec<String>,
    pub tui_indicators: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerminalFeature {
    ColorOutput,
    CursorControl,
    TerminalSize,
    RawMode,
    AlternateScreen,
    UserInput,
    PasswordInput,
    MenuSelection,
    ProgressBars,
    LiveOutput,
}

pub struct TerminalDetector;

impl TerminalDetector {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze(ast: &AST) -> TerminalAnalysis {
        let mut analysis = TerminalAnalysis {
            requirement: TerminalRequirement::None,
            features_used: HashSet::new(),
            interactive_commands: Vec::new(),
            tui_indicators: Vec::new(),
        };
        
        Self::analyze_node(&ast.root, &mut analysis);
        
        // Determine overall requirement level
        analysis.requirement = Self::determine_requirement(&analysis);
        
        analysis
    }
    
    fn analyze_node(node: &ASTNode, analysis: &mut TerminalAnalysis) {
        match node {
            ASTNode::Script(statements) | ASTNode::Block(statements) => {
                for stmt in statements {
                    Self::analyze_node(stmt, analysis);
                }
            }
            
            ASTNode::Command { name, args, .. } => {
                Self::analyze_command(name, args, analysis);
            }
            
            ASTNode::Pipeline(commands) => {
                for cmd in commands {
                    Self::analyze_node(cmd, analysis);
                }
            }
            
            ASTNode::String(content, _) => {
                Self::analyze_string_content(content, analysis);
            }
            
            ASTNode::If { condition, then_block, elif_blocks, else_block } => {
                Self::analyze_node(condition, analysis);
                Self::analyze_node(then_block, analysis);
                for (cond, block) in elif_blocks {
                    Self::analyze_node(cond, analysis);
                    Self::analyze_node(block, analysis);
                }
                if let Some(block) = else_block {
                    Self::analyze_node(block, analysis);
                }
            }
            
            ASTNode::While { condition, body } | ASTNode::Until { condition, body } => {
                Self::analyze_node(condition, analysis);
                Self::analyze_node(body, analysis);
            }
            
            ASTNode::For { items, body, .. } => {
                match items {
                    crate::parser::ast::ForItems::List(list) => {
                        for item in list {
                            Self::analyze_node(item, analysis);
                        }
                    }
                    crate::parser::ast::ForItems::Command(cmd) => {
                        Self::analyze_node(cmd, analysis);
                    }
                    crate::parser::ast::ForItems::CStyle { init, condition, update } => {
                        if let Some(init) = init {
                            Self::analyze_node(init, analysis);
                        }
                        if let Some(condition) = condition {
                            Self::analyze_node(condition, analysis);
                        }
                        if let Some(update) = update {
                            Self::analyze_node(update, analysis);
                        }
                    }
                }
                Self::analyze_node(body, analysis);
            }
            
            ASTNode::Function { body, .. } => {
                Self::analyze_node(body, analysis);
            }
            
            ASTNode::CommandSubstitution(cmd) | ASTNode::Subshell(cmd) => {
                Self::analyze_node(cmd, analysis);
            }
            
            _ => {}
        }
    }
    
    fn analyze_command(name: &str, args: &[Box<ASTNode>], analysis: &mut TerminalAnalysis) {
        // Check for interactive commands
        match name {
            // User input commands
            "read" => {
                analysis.features_used.insert(TerminalFeature::UserInput);
                analysis.interactive_commands.push("read".to_string());
                
                // Check for password input
                for arg in args {
                    if let ASTNode::String(s, _) = arg.as_ref() {
                        if s == "-s" {
                            analysis.features_used.insert(TerminalFeature::PasswordInput);
                        }
                    }
                }
            }
            
            // Menu/selection commands
            "select" => {
                analysis.features_used.insert(TerminalFeature::MenuSelection);
                analysis.interactive_commands.push("select".to_string());
            }
            
            // Terminal control commands
            "tput" => {
                analysis.features_used.insert(TerminalFeature::CursorControl);
                analysis.features_used.insert(TerminalFeature::ColorOutput);
                
                // Check specific tput commands
                if let Some(first_arg) = args.first() {
                    if let ASTNode::String(cmd, _) = first_arg.as_ref() {
                        match cmd.as_str() {
                            "cols" | "lines" => {
                                analysis.features_used.insert(TerminalFeature::TerminalSize);
                            }
                            "cup" | "cuu" | "cud" | "cuf" | "cub" => {
                                analysis.features_used.insert(TerminalFeature::CursorControl);
                            }
                            "smcup" | "rmcup" => {
                                analysis.features_used.insert(TerminalFeature::AlternateScreen);
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Color output commands
            "colorize" | "lolcat" => {
                analysis.features_used.insert(TerminalFeature::ColorOutput);
            }
            
            // Terminal UI programs
            "dialog" | "whiptail" | "zenity" => {
                analysis.features_used.insert(TerminalFeature::FullTUI);
                analysis.tui_indicators.push(name.to_string());
            }
            
            // Pagers and editors
            "less" | "more" | "vim" | "vi" | "nano" | "emacs" => {
                analysis.features_used.insert(TerminalFeature::FullTUI);
                analysis.tui_indicators.push(name.to_string());
            }
            
            // Progress indicators
            "pv" | "progress" => {
                analysis.features_used.insert(TerminalFeature::ProgressBars);
            }
            
            // Live monitoring
            "watch" | "tail" if args.iter().any(|a| {
                matches!(a.as_ref(), ASTNode::String(s, _) if s == "-f")
            }) => {
                analysis.features_used.insert(TerminalFeature::LiveOutput);
                analysis.interactive_commands.push(format!("{} -f", name));
            }
            
            // Clear screen
            "clear" | "reset" => {
                analysis.features_used.insert(TerminalFeature::CursorControl);
            }
            
            // Stty for terminal settings
            "stty" => {
                analysis.features_used.insert(TerminalFeature::RawMode);
                for arg in args {
                    if let ASTNode::String(s, _) = arg.as_ref() {
                        if s == "-echo" {
                            analysis.features_used.insert(TerminalFeature::PasswordInput);
                        }
                    }
                }
            }
            
            _ => {}
        }
        
        // Check arguments for terminal-related flags
        for arg in args {
            Self::analyze_node(arg, analysis);
        }
    }
    
    fn analyze_string_content(content: &str, analysis: &mut TerminalAnalysis) {
        // Check for ANSI escape sequences
        if content.contains("\x1b[") || content.contains("\\033[") || content.contains("\\e[") {
            analysis.features_used.insert(TerminalFeature::ColorOutput);
            
            // Check for specific sequences
            if content.contains("?1049h") || content.contains("?1049l") {
                analysis.features_used.insert(TerminalFeature::AlternateScreen);
            }
            
            if content.contains("H") || content.contains("J") || content.contains("K") {
                analysis.features_used.insert(TerminalFeature::CursorControl);
            }
        }
        
        // Check for color variables
        if content.contains("${RED}") || content.contains("${GREEN}") || 
           content.contains("${BLUE}") || content.contains("${NC}") ||
           content.contains("\\033[0;3") {
            analysis.features_used.insert(TerminalFeature::ColorOutput);
        }
        
        // Check for terminal size references
        if content.contains("$COLUMNS") || content.contains("$LINES") ||
           content.contains("$(tput cols)") || content.contains("$(tput lines)") {
            analysis.features_used.insert(TerminalFeature::TerminalSize);
        }
    }
    
    fn determine_requirement(analysis: &TerminalAnalysis) -> TerminalRequirement {
        if analysis.features_used.contains(&TerminalFeature::FullTUI) ||
           !analysis.tui_indicators.is_empty() {
            TerminalRequirement::FullTUI
        } else if analysis.features_used.contains(&TerminalFeature::UserInput) ||
                  analysis.features_used.contains(&TerminalFeature::MenuSelection) ||
                  analysis.features_used.contains(&TerminalFeature::PasswordInput) ||
                  analysis.features_used.contains(&TerminalFeature::LiveOutput) {
            TerminalRequirement::Interactive
        } else if analysis.features_used.contains(&TerminalFeature::ColorOutput) ||
                  analysis.features_used.contains(&TerminalFeature::CursorControl) ||
                  analysis.features_used.contains(&TerminalFeature::TerminalSize) {
            TerminalRequirement::TerminalFeatures
        } else {
            TerminalRequirement::None
        }
    }
}

impl TerminalAnalysis {
    pub fn needs_terminal(&self) -> bool {
        self.requirement != TerminalRequirement::None
    }
    
    pub fn can_run_headless(&self) -> bool {
        self.requirement == TerminalRequirement::None
    }
    
    pub fn is_interactive(&self) -> bool {
        matches!(self.requirement, TerminalRequirement::Interactive | TerminalRequirement::FullTUI)
    }
    
    pub fn uses_tui(&self) -> bool {
        self.requirement == TerminalRequirement::FullTUI
    }
    
    pub fn get_required_crates(&self) -> Vec<(&'static str, &'static str)> {
        let mut crates = Vec::new();
        
        if self.features_used.contains(&TerminalFeature::ColorOutput) {
            crates.push(("colored", "2.1"));
        }
        
        if self.features_used.contains(&TerminalFeature::CursorControl) ||
           self.features_used.contains(&TerminalFeature::TerminalSize) {
            crates.push(("crossterm", "0.27"));
        }
        
        if self.features_used.contains(&TerminalFeature::UserInput) ||
           self.features_used.contains(&TerminalFeature::MenuSelection) {
            crates.push(("dialoguer", "0.11"));
        }
        
        if self.features_used.contains(&TerminalFeature::ProgressBars) {
            crates.push(("indicatif", "0.17"));
        }
        
        if self.features_used.contains(&TerminalFeature::FullTUI) {
            crates.push(("ratatui", "0.25"));
        }
        
        if self.features_used.contains(&TerminalFeature::RawMode) {
            crates.push(("termion", "2.0"));
        }
        
        crates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ShellParser, shell_dialect::ShellDialect};
    
    #[test]
    fn test_detect_interactive_script() {
        let script = r#"#!/bin/bash
echo "Enter your name:"
read NAME
echo "Hello, $NAME!"
"#;
        
        let mut parser = ShellParser::new(script.to_string(), ShellDialect::Bash).unwrap();
        let ast = parser.parse().unwrap();
        let analysis = TerminalDetector::analyze(&ast);
        
        assert!(analysis.needs_terminal());
        assert!(analysis.is_interactive());
        assert!(analysis.features_used.contains(&TerminalFeature::UserInput));
    }
    
    #[test]
    fn test_detect_color_output() {
        let script = r#"#!/bin/bash
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
echo -e "${RED}Error${NC}"
echo -e "${GREEN}Success${NC}"
"#;
        
        let mut parser = ShellParser::new(script.to_string(), ShellDialect::Bash).unwrap();
        let ast = parser.parse().unwrap();
        let analysis = TerminalDetector::analyze(&ast);
        
        assert!(analysis.features_used.contains(&TerminalFeature::ColorOutput));
        assert_eq!(analysis.requirement, TerminalRequirement::TerminalFeatures);
    }
    
    #[test]
    fn test_detect_tui_application() {
        let script = r#"#!/bin/bash
dialog --title "Menu" --menu "Choose:" 15 40 4 \
    1 "Option 1" \
    2 "Option 2"
"#;
        
        let mut parser = ShellParser::new(script.to_string(), ShellDialect::Bash).unwrap();
        let ast = parser.parse().unwrap();
        let analysis = TerminalDetector::analyze(&ast);
        
        assert!(analysis.uses_tui());
        assert!(analysis.tui_indicators.contains(&"dialog".to_string()));
    }
    
    #[test]
    fn test_headless_script() {
        let script = r#"#!/bin/bash
cp file1.txt file2.txt
echo "Done" > log.txt
"#;
        
        let mut parser = ShellParser::new(script.to_string(), ShellDialect::Bash).unwrap();
        let ast = parser.parse().unwrap();
        let analysis = TerminalDetector::analyze(&ast);
        
        assert!(analysis.can_run_headless());
        assert_eq!(analysis.requirement, TerminalRequirement::None);
    }
}