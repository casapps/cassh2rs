use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::env;
use std::io::{self, Write};

/// Echo command - print arguments to stdout
pub fn echo(args: &[&str]) -> Result<()> {
    let mut output = String::new();
    let mut no_newline = false;
    let mut escape = false;
    let mut skip_next = false;
    
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        
        match *arg {
            "-n" if i == 0 => no_newline = true,
            "-e" if i == 0 => escape = true,
            "-E" if i == 0 => escape = false,
            _ => {
                if !output.is_empty() {
                    output.push(' ');
                }
                
                if escape {
                    output.push_str(&process_escape_sequences(arg));
                } else {
                    output.push_str(arg);
                }
            }
        }
    }
    
    if no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
    
    io::stdout().flush()?;
    Ok(())
}

/// Printf command - formatted output
pub fn printf(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    
    let format_str = args[0];
    let values = &args[1..];
    
    // Simple printf implementation
    // TODO: Implement full printf formatting
    let mut output = format_str.to_string();
    let mut value_idx = 0;
    
    // Replace %s with string values
    while let Some(pos) = output.find("%s") {
        if value_idx < values.len() {
            output.replace_range(pos..pos+2, values[value_idx]);
            value_idx += 1;
        } else {
            break;
        }
    }
    
    // Replace %d with integer values
    value_idx = 0;
    while let Some(pos) = output.find("%d") {
        if value_idx < values.len() {
            output.replace_range(pos..pos+2, values[value_idx]);
            value_idx += 1;
        } else {
            break;
        }
    }
    
    // Process escape sequences
    output = process_escape_sequences(&output);
    
    print!("{}", output);
    io::stdout().flush()?;
    Ok(())
}

/// Read command - read input from stdin
pub fn read(args: &[&str]) -> Result<String> {
    let mut prompt = None;
    let mut timeout = None;
    let mut var_names = Vec::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-p" => {
                if i + 1 < args.len() {
                    prompt = Some(args[i + 1]);
                    i += 1;
                }
            }
            "-t" => {
                if i + 1 < args.len() {
                    timeout = args[i + 1].parse::<u64>().ok();
                    i += 1;
                }
            }
            _ => {
                var_names.push(args[i]);
            }
        }
        i += 1;
    }
    
    // Display prompt if provided
    if let Some(p) = prompt {
        print!("{}", p);
        io::stdout().flush()?;
    }
    
    // Read line from stdin
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    
    // Remove trailing newline
    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }
    
    Ok(buffer)
}

/// Test command - evaluate conditional expressions
pub fn test(args: &[&str]) -> Result<bool> {
    if args.is_empty() {
        return Ok(false);
    }
    
    // Handle [ ] syntax
    let args = if args.last() == Some(&"]") {
        &args[..args.len()-1]
    } else {
        args
    };
    
    match args.len() {
        0 => Ok(false),
        1 => {
            // Single argument: true if non-empty
            Ok(!args[0].is_empty())
        }
        2 => {
            // Unary operators
            match args[0] {
                "-e" => Ok(Path::new(args[1]).exists()),
                "-f" => Ok(Path::new(args[1]).is_file()),
                "-d" => Ok(Path::new(args[1]).is_dir()),
                "-r" => {
                    let path = Path::new(args[1]);
                    Ok(path.exists() && is_readable(path))
                }
                "-w" => {
                    let path = Path::new(args[1]);
                    Ok(path.exists() && is_writable(path))
                }
                "-x" => {
                    let path = Path::new(args[1]);
                    Ok(path.exists() && is_executable(path))
                }
                "-s" => {
                    let path = Path::new(args[1]);
                    Ok(path.exists() && path.metadata()?.len() > 0)
                }
                "-z" => Ok(args[1].is_empty()),
                "-n" => Ok(!args[1].is_empty()),
                _ => Ok(false),
            }
        }
        3 => {
            // Binary operators
            match args[1] {
                "=" | "==" => Ok(args[0] == args[2]),
                "!=" => Ok(args[0] != args[2]),
                "-eq" => {
                    let a = args[0].parse::<i64>().unwrap_or(0);
                    let b = args[2].parse::<i64>().unwrap_or(0);
                    Ok(a == b)
                }
                "-ne" => {
                    let a = args[0].parse::<i64>().unwrap_or(0);
                    let b = args[2].parse::<i64>().unwrap_or(0);
                    Ok(a != b)
                }
                "-lt" => {
                    let a = args[0].parse::<i64>().unwrap_or(0);
                    let b = args[2].parse::<i64>().unwrap_or(0);
                    Ok(a < b)
                }
                "-le" => {
                    let a = args[0].parse::<i64>().unwrap_or(0);
                    let b = args[2].parse::<i64>().unwrap_or(0);
                    Ok(a <= b)
                }
                "-gt" => {
                    let a = args[0].parse::<i64>().unwrap_or(0);
                    let b = args[2].parse::<i64>().unwrap_or(0);
                    Ok(a > b)
                }
                "-ge" => {
                    let a = args[0].parse::<i64>().unwrap_or(0);
                    let b = args[2].parse::<i64>().unwrap_or(0);
                    Ok(a >= b)
                }
                _ => Ok(false),
            }
        }
        _ => {
            // Complex expressions
            // TODO: Implement full test expression parsing
            Ok(false)
        }
    }
}

/// Change directory
pub fn cd(args: &[&str]) -> Result<()> {
    let path = if args.is_empty() {
        dirs::home_dir().context("Could not find home directory")?
    } else if args[0] == "-" {
        // Return to previous directory
        // TODO: Implement OLDPWD tracking
        PathBuf::from(env::var("OLDPWD").unwrap_or_else(|_| ".".to_string()))
    } else {
        PathBuf::from(args[0])
    };
    
    env::set_current_dir(&path)
        .with_context(|| format!("Failed to change directory to {}", path.display()))?;
    
    Ok(())
}

/// Print working directory
pub fn pwd(_args: &[&str]) -> Result<()> {
    let cwd = env::current_dir()?;
    println!("{}", cwd.display());
    Ok(())
}

/// Export variables to environment
pub fn export(args: &[&str]) -> Result<()> {
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            env::set_var(key, value);
        } else {
            // Export existing variable
            // In a real shell runtime, we'd look this up in the variable table
        }
    }
    Ok(())
}

/// Unset variables
pub fn unset(args: &[&str]) -> Result<()> {
    for var in args {
        env::remove_var(var);
    }
    Ok(())
}

/// Source a script file
pub fn source(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        bail!("source: filename argument required");
    }
    
    let script_path = Path::new(args[0]);
    if !script_path.exists() {
        bail!("source: {}: No such file or directory", args[0]);
    }
    
    // In a real implementation, this would parse and execute the script
    // For now, we just return success
    println!("TODO: Execute script {}", script_path.display());
    Ok(())
}

/// Exit the shell
pub fn exit(args: &[&str]) -> Result<()> {
    let code = if args.is_empty() {
        0
    } else {
        args[0].parse::<i32>().unwrap_or(0)
    };
    
    std::process::exit(code);
}

// Helper functions

fn process_escape_sequences(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\'", "'")
}

#[cfg(unix)]
fn is_readable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = path.metadata() {
        let mode = metadata.permissions().mode();
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        
        // Check owner, group, and other read permissions
        if uid == 0 {
            true // root can read anything
        } else if uid == metadata.uid() {
            mode & 0o400 != 0
        } else if gid == metadata.gid() {
            mode & 0o040 != 0
        } else {
            mode & 0o004 != 0
        }
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_readable(path: &Path) -> bool {
    path.metadata().is_ok()
}

#[cfg(unix)]
fn is_writable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = path.metadata() {
        let mode = metadata.permissions().mode();
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        
        if uid == 0 {
            true // root can write anything
        } else if uid == metadata.uid() {
            mode & 0o200 != 0
        } else if gid == metadata.gid() {
            mode & 0o020 != 0
        } else {
            mode & 0o002 != 0
        }
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_writable(path: &Path) -> bool {
    !path.metadata().map(|m| m.permissions().readonly()).unwrap_or(true)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = path.metadata() {
        let mode = metadata.permissions().mode();
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        
        if uid == 0 {
            mode & 0o111 != 0 // root can execute if any execute bit is set
        } else if uid == metadata.uid() {
            mode & 0o100 != 0
        } else if gid == metadata.gid() {
            mode & 0o010 != 0
        } else {
            mode & 0o001 != 0
        }
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    // On Windows, check file extension
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().map(|s| s.to_lowercase()).as_deref(),
            Some("exe") | Some("bat") | Some("cmd") | Some("com")
        )
    } else {
        false
    }
}

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;