use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellDialect {
    Bash,
    Zsh,
    Fish,
    Dash,
    Ksh,
    Tcsh,
    Csh,
    PowerShell,
    Posix,  // Generic POSIX shell
}

impl ShellDialect {
    /// Detect shell dialect from shebang line
    pub fn from_shebang(shebang: &str) -> Self {
        let shebang = shebang.trim();
        
        if shebang.contains("bash") {
            ShellDialect::Bash
        } else if shebang.contains("zsh") {
            ShellDialect::Zsh
        } else if shebang.contains("fish") {
            ShellDialect::Fish
        } else if shebang.contains("dash") {
            ShellDialect::Dash
        } else if shebang.contains("ksh") {
            ShellDialect::Ksh
        } else if shebang.contains("tcsh") {
            ShellDialect::Tcsh
        } else if shebang.contains("csh") && !shebang.contains("tcsh") {
            ShellDialect::Csh
        } else if shebang.contains("pwsh") || shebang.contains("powershell") {
            ShellDialect::PowerShell
        } else if shebang.contains("sh") {
            ShellDialect::Posix
        } else {
            ShellDialect::Bash // Default to bash
        }
    }
    
    /// Detect shell dialect from file extension
    pub fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "bash" | "sh" => Some(ShellDialect::Bash),
                "zsh" => Some(ShellDialect::Zsh),
                "fish" => Some(ShellDialect::Fish),
                "dash" => Some(ShellDialect::Dash),
                "ksh" => Some(ShellDialect::Ksh),
                "tcsh" => Some(ShellDialect::Tcsh),
                "csh" => Some(ShellDialect::Csh),
                "ps1" | "psm1" | "psd1" => Some(ShellDialect::PowerShell),
                _ => None,
            })
    }
    
    /// Check if this dialect supports a specific feature
    pub fn supports_feature(&self, feature: ShellFeature) -> bool {
        use ShellFeature::*;
        
        match (self, feature) {
            // Arrays are supported by most shells except POSIX sh and csh
            (ShellDialect::Posix | ShellDialect::Csh, Arrays) => false,
            
            // Associative arrays only in bash 4+, zsh, and ksh
            (ShellDialect::Bash | ShellDialect::Zsh | ShellDialect::Ksh, AssociativeArrays) => true,
            (_, AssociativeArrays) => false,
            
            // Process substitution not in POSIX sh, dash, or Windows shells
            (ShellDialect::Posix | ShellDialect::Dash | ShellDialect::PowerShell, ProcessSubstitution) => false,
            
            // Extended test [[ ]] not in pure POSIX
            (ShellDialect::Posix, ExtendedTest) => false,
            
            // Most features are supported by default
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellFeature {
    Arrays,
    AssociativeArrays,
    ProcessSubstitution,
    ExtendedTest,
    FunctionKeyword,
    LocalKeyword,
    SelectLoop,
}