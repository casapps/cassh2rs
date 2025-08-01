use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cassh2rs::parser::{ShellParser, shell_dialect::ShellDialect};
use cassh2rs::resolver::{FileClassifier, FileContext};
use std::path::Path;

fn bench_lexer(c: &mut Criterion) {
    let script = include_str!("../examples/advanced_monitoring.sh");
    
    c.bench_function("lexer_advanced_script", |b| {
        b.iter(|| {
            let mut lexer = cassh2rs::parser::Lexer::new(
                black_box(script),
                ShellDialect::Bash
            );
            
            while let Ok(token) = lexer.next_token() {
                if token == cassh2rs::parser::Token::Eof {
                    break;
                }
                black_box(token);
            }
        });
    });
}

fn bench_parser(c: &mut Criterion) {
    let simple_script = r#"
#!/bin/bash
echo "Hello World"
for i in 1 2 3; do
    echo $i
done
"#;

    let complex_script = include_str!("../examples/advanced_monitoring.sh");
    
    c.bench_function("parser_simple_script", |b| {
        b.iter(|| {
            let mut parser = ShellParser::new(
                black_box(simple_script.to_string()),
                ShellDialect::Bash
            ).unwrap();
            black_box(parser.parse().unwrap());
        });
    });
    
    c.bench_function("parser_complex_script", |b| {
        b.iter(|| {
            let mut parser = ShellParser::new(
                black_box(complex_script.to_string()),
                ShellDialect::Bash
            ).unwrap();
            black_box(parser.parse().unwrap());
        });
    });
}

fn bench_file_classifier(c: &mut Criterion) {
    let classifier = FileClassifier::new();
    let paths = vec![
        "/proc/cpuinfo",
        "/etc/passwd",
        "config.toml",
        "script.sh",
        "/tmp/test.txt",
        "README.md",
    ];
    
    c.bench_function("file_classification", |b| {
        b.iter(|| {
            for path in &paths {
                let context = FileContext::default();
                black_box(classifier.classify(Path::new(black_box(path)), &context));
            }
        });
    });
}

fn bench_shell_features(c: &mut Criterion) {
    use cassh2rs::parser::shell_dialect::{ShellDialect, ShellFeature};
    
    let dialects = vec![
        ShellDialect::Bash,
        ShellDialect::Zsh,
        ShellDialect::Fish,
        ShellDialect::Dash,
    ];
    
    let features = vec![
        ShellFeature::Arrays,
        ShellFeature::AssociativeArrays,
        ShellFeature::ProcessSubstitution,
        ShellFeature::ExtendedTest,
    ];
    
    c.bench_function("shell_feature_detection", |b| {
        b.iter(|| {
            for dialect in &dialects {
                for feature in &features {
                    black_box(dialect.supports_feature(*feature));
                }
            }
        });
    });
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_file_classifier,
    bench_shell_features
);
criterion_main!(benches);