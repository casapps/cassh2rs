use cassh2rs::parser::{ShellParser, shell_dialect::ShellDialect};
use cassh2rs::generator::code_gen::CodeGenerator;

#[test]
fn test_generate_simple_echo() {
    let input = "echo 'Hello, World!'";
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    // Check that main.rs was generated
    assert!(project.files.contains_key(&"src/main.rs".into()));
    
    // Check that the generated code contains println!
    let main_content = &project.files[&"src/main.rs".into()];
    assert!(main_content.contains("println!"));
}

#[test]
fn test_generate_variable_assignment() {
    let input = r#"
NAME="John"
echo "Hello, $NAME"
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    let main_content = &project.files[&"src/main.rs".into()];
    assert!(main_content.contains("runtime.set_var"));
    assert!(main_content.contains("runtime.get_var"));
}

#[test]
fn test_generate_if_statement() {
    let input = r#"
if [ -f "test.txt" ]; then
    echo "File exists"
else
    echo "File not found"
fi
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    let main_content = &project.files[&"src/main.rs".into()];
    assert!(main_content.contains("if"));
    assert!(main_content.contains("std::path::Path::new"));
    assert!(main_content.contains("is_file()"));
}

#[test]
fn test_generate_for_loop() {
    let input = r#"
for i in 1 2 3; do
    echo "Number: $i"
done
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    let main_content = &project.files[&"src/main.rs".into()];
    assert!(main_content.contains("for"));
    assert!(main_content.contains("runtime.set_var"));
}

#[test]
fn test_generate_function() {
    let input = r#"
function greet() {
    echo "Hello, $1!"
}

greet "World"
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    let main_content = &project.files[&"src/main.rs".into()];
    assert!(main_content.contains("fn shell_func_greet"));
    assert!(main_content.contains("runtime.register_function"));
}

#[test]
fn test_project_structure() {
    let input = "echo test";
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    // Check all required files are generated
    assert!(project.files.contains_key(&"src/main.rs".into()));
    assert!(project.files.contains_key(&"src/config.rs".into()));
    assert!(project.files.contains_key(&"src/shell_runtime.rs".into()));
    assert!(project.files.contains_key(&"src/commands/mod.rs".into()));
    assert!(project.files.contains_key(&"src/ui/mod.rs".into()));
}

#[test]
fn test_update_config() {
    let input = "echo test";
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let mut generator = CodeGenerator::new(ast, "test_script");
    let mut project = generator.generate().unwrap();
    
    // Test update configuration
    project.set_update_config(
        Some("github.com/user/repo".to_string()),
        Some("/api/v1/releases".to_string())
    );
    
    assert!(project.update_config.enabled);
    assert_eq!(project.update_config.repo, Some("github.com/user/repo".to_string()));
    assert_eq!(project.update_config.api_path, Some("/api/v1/releases".to_string()));
}

#[test]
fn test_metadata_extraction() {
    let input = r#"#!/bin/bash
# @Version: 2.0.0
# @Author: Test Developer
# @Description: Advanced test script

echo "Script with metadata"
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    assert_eq!(project.version, "2.0.0");
    assert_eq!(project.author, "Test Developer");
    assert_eq!(project.description, "Advanced test script");
}

#[test]
fn test_external_command_handling() {
    let input = r#"
git status
curl https://example.com
jq '.data' file.json
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    let generator = CodeGenerator::new(ast, "test_script");
    let project = generator.generate().unwrap();
    
    let main_content = &project.files[&"src/main.rs".into()];
    assert!(main_content.contains("runtime.execute_command"));
    assert!(main_content.contains(r#""git""#));
    assert!(main_content.contains(r#""curl""#));
    assert!(main_content.contains(r#""jq""#));
}