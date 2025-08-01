# Contributing to cassh2rs

Thank you for your interest in contributing to cassh2rs! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

Please note that this project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Issues

- Check if the issue already exists in the [issue tracker](https://github.com/casapps/cassh2rs/issues)
- Use the issue templates when creating new issues
- Provide as much detail as possible, including:
  - Shell script that demonstrates the issue
  - Expected vs actual behavior
  - Error messages and stack traces
  - OS and Rust version

### Suggesting Features

- Open a discussion in the [Discussions](https://github.com/casapps/cassh2rs/discussions) tab
- Describe the use case and benefits
- Consider how it fits with existing features
- Be open to feedback and alternative approaches

### Pull Requests

1. **Fork the repository** and create your branch from `develop`
2. **Follow the coding style** (see below)
3. **Add tests** for new functionality
4. **Update documentation** as needed
5. **Run the test suite** to ensure nothing is broken
6. **Submit a pull request** with a clear description

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- cargo-cross (for cross-compilation testing)

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/cassh2rs.git
cd cassh2rs

# Add upstream remote
git remote add upstream https://github.com/casapps/cassh2rs.git

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin cargo-audit

# Run tests
cargo test

# Run with example
cargo run -- examples/deploy.sh --dry-run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test lexer_tests

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench

# Check code coverage
cargo tarpaulin --out Html
```

## Coding Guidelines

### Rust Style

- Follow standard Rust naming conventions
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Keep functions small and focused
- Prefer explicit error handling over panics

### Code Organization

```rust
// Good: Clear module structure
pub mod lexer {
    pub struct Lexer { ... }
    
    impl Lexer {
        pub fn new() -> Self { ... }
        pub fn next_token(&mut self) -> Result<Token> { ... }
    }
}

// Good: Descriptive error messages
bail!("Unsupported shell feature '{}' at line {}", feature, line);

// Good: Documentation
/// Parses a shell script and returns an AST.
/// 
/// # Arguments
/// * `input` - The shell script content
/// * `dialect` - The shell dialect to use
/// 
/// # Returns
/// * `Result<AST>` - The parsed AST or an error
pub fn parse(input: &str, dialect: ShellDialect) -> Result<AST> {
    // Implementation
}
```

### Testing Guidelines

- Write unit tests for all new functions
- Use property-based testing for complex logic
- Test error cases and edge conditions
- Keep test data in `tests/fixtures/`

Example test:
```rust
#[test]
fn test_parse_complex_pipeline() {
    let input = "cat file | grep pattern | wc -l";
    let ast = parse(input, ShellDialect::Bash).unwrap();
    
    match &ast.root {
        ASTNode::Pipeline(cmds) => {
            assert_eq!(cmds.len(), 3);
            // More assertions...
        }
        _ => panic!("Expected pipeline"),
    }
}
```

## Adding New Features

### Supporting New Shell Dialects

1. Add variant to `ShellDialect` enum
2. Update detection logic in `from_shebang()` and `from_extension()`
3. Add dialect-specific lexer rules
4. Update parser for dialect-specific syntax
5. Add tests for the new dialect

### Adding New Shell Built-ins

1. Implement in `src/commands/builtins.rs`
2. Add to the `is_builtin()` function
3. Update code generator to use the implementation
4. Add tests for the built-in

### Adding File Classification Rules

1. Update patterns in `src/resolver/file_classifier.rs`
2. Add tests for new patterns
3. Document the rule in SPECIFICATIONS.md

## Documentation

- Update README.md for user-facing changes
- Update USAGE.md with examples
- Update SPECIFICATIONS.md for technical changes
- Add inline documentation for public APIs
- Include examples in doc comments

## Release Process

1. Create a release branch from `develop`
2. Update version in `Cargo.toml`
3. Update CHANGELOG.md
4. Create PR to `main`
5. After merge, tag the release
6. GitHub Actions will build and publish

## Getting Help

- Join our [Discord server](https://discord.gg/cassh2rs)
- Ask questions in [Discussions](https://github.com/casapps/cassh2rs/discussions)
- Check the [Wiki](https://github.com/casapps/cassh2rs/wiki)

## Recognition

Contributors will be recognized in:
- The README.md contributors section
- Release notes
- The project website

Thank you for contributing to cassh2rs!