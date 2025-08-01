use cassh2rs::parser::{ShellParser, AST, ASTNode, shell_dialect::ShellDialect};

#[test]
fn test_parse_simple_command() {
    let input = "echo hello world";
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    match &ast.root {
        ASTNode::Script(statements) => {
            assert_eq!(statements.len(), 1);
            match statements[0].as_ref() {
                ASTNode::Command { name, args, .. } => {
                    assert_eq!(name, "echo");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected command node"),
            }
        }
        _ => panic!("Expected script node"),
    }
}

#[test]
fn test_parse_variable_assignment() {
    let input = "NAME=value\nexport PATH=/usr/bin";
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    match &ast.root {
        ASTNode::Script(statements) => {
            assert_eq!(statements.len(), 2);
            
            // First assignment
            match statements[0].as_ref() {
                ASTNode::Assignment { name, export, .. } => {
                    assert_eq!(name, "NAME");
                    assert!(!export);
                }
                _ => panic!("Expected assignment node"),
            }
            
            // Export statement
            match statements[1].as_ref() {
                ASTNode::Assignment { name, export, .. } => {
                    assert_eq!(name, "PATH");
                    assert!(export);
                }
                _ => panic!("Expected export assignment"),
            }
        }
        _ => panic!("Expected script node"),
    }
}

#[test]
fn test_parse_if_statement() {
    let input = r#"
if [ -f file.txt ]; then
    echo "File exists"
else
    echo "File not found"
fi
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    match &ast.root {
        ASTNode::Script(statements) => {
            assert_eq!(statements.len(), 1);
            match statements[0].as_ref() {
                ASTNode::If { then_block, else_block, .. } => {
                    assert!(!then_block.is_empty_block());
                    assert!(else_block.is_some());
                }
                _ => panic!("Expected if node"),
            }
        }
        _ => panic!("Expected script node"),
    }
}

#[test]
fn test_parse_for_loop() {
    let input = r#"
for i in 1 2 3; do
    echo $i
done
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    match &ast.root {
        ASTNode::Script(statements) => {
            assert_eq!(statements.len(), 1);
            match statements[0].as_ref() {
                ASTNode::For { variable, body, .. } => {
                    assert_eq!(variable, "i");
                    assert!(!body.is_empty_block());
                }
                _ => panic!("Expected for loop"),
            }
        }
        _ => panic!("Expected script node"),
    }
}

#[test]
fn test_parse_function() {
    let input = r#"
function greet() {
    echo "Hello, $1!"
}
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    match &ast.root {
        ASTNode::Script(statements) => {
            assert_eq!(statements.len(), 1);
            match statements[0].as_ref() {
                ASTNode::Function { name, body } => {
                    assert_eq!(name, "greet");
                    assert!(!body.is_empty_block());
                }
                _ => panic!("Expected function"),
            }
        }
        _ => panic!("Expected script node"),
    }
}

#[test]
fn test_parse_pipeline() {
    let input = "cat file.txt | grep pattern | wc -l";
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    match &ast.root {
        ASTNode::Script(statements) => {
            assert_eq!(statements.len(), 1);
            match statements[0].as_ref() {
                ASTNode::Pipeline(commands) => {
                    assert_eq!(commands.len(), 3);
                }
                _ => panic!("Expected pipeline"),
            }
        }
        _ => panic!("Expected script node"),
    }
}

#[test]
fn test_extract_metadata() {
    let input = r#"#!/bin/bash
# @Version: 1.0.0
# @Author: Test User
# @Description: Test script
# @Dependency: curl
# @Dependency: jq

echo "Script content"
"#;
    
    let mut parser = ShellParser::new(input.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    assert_eq!(ast.metadata.shebang, Some("#!/bin/bash".to_string()));
    assert_eq!(ast.metadata.version, Some("1.0.0".to_string()));
    assert_eq!(ast.metadata.author, Some("Test User".to_string()));
    assert_eq!(ast.metadata.description, Some("Test script".to_string()));
    assert_eq!(ast.metadata.dependencies.len(), 2);
    assert!(ast.metadata.dependencies.contains(&"curl".to_string()));
    assert!(ast.metadata.dependencies.contains(&"jq".to_string()));
}