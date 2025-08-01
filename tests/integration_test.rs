use std::path::Path;
use std::fs;

#[test]
fn test_simple_script_parsing() {
    let script = r#"#!/bin/bash
echo "Hello World"
"#;
    
    // Test that we can parse a simple script
    use cassh2rs::parser::{ShellParser, shell_dialect::ShellDialect};
    
    let mut parser = ShellParser::new(script.to_string(), ShellDialect::Bash).unwrap();
    let ast = parser.parse().unwrap();
    
    assert!(ast.metadata.shebang.is_some());
    assert_eq!(ast.metadata.shebang.as_ref().unwrap(), "#!/bin/bash");
}

#[test]
fn test_file_classification() {
    use cassh2rs::resolver::{FileClassifier, FileClassification, FileContext};
    
    let classifier = FileClassifier::new();
    let mut context = FileContext::default();
    
    // Test system paths
    let info = classifier.classify(Path::new("/proc/cpuinfo"), &context);
    assert_eq!(info.classification, FileClassification::Runtime);
    
    // Test local config
    context.is_local_to_script = true;
    let info = classifier.classify(Path::new("config.toml"), &context);
    assert_eq!(info.classification, FileClassification::Static);
}