# cassh2rs - Universal Script Converter - Technical Specifications

## Project Overview

cassh2rs is a comprehensive command-line tool and GUI application that converts any shell script (bash, zsh, fish, dash, ksh, tcsh, csh, PowerShell) into optimized, self-contained Rust binaries. The tool performs static analysis, dependency resolution, and code generation to produce cross-platform executables with zero runtime dependencies.

- **Repository**: github.com/casapps/cassh2rs
- **License**: MIT

## Core Architecture

### 1. Parser Module (`src/parser/`)
- Multi-shell lexer supporting all major shell dialects
- LALR(1) grammar parser with error recovery
- AST generation with full semantic analysis
- Handles complex quoting, escaping, heredocs, parameter expansion
- Supports all shell constructs: functions, loops, conditionals, pipes, redirections
- Template placeholder resolution (GEN_SCRIPT_REPLACE_*)
- Header metadata extraction (@Version, @Author, etc.)

### 2. Dependency Resolver (`src/resolver/`)
- Recursive source file following with circular dependency detection
- Dynamic path resolution using static analysis
- Smart file classification system with 200+ rules
- Binary dependency detection and bundling
- Network resource caching at build time
- Cross-platform path normalization

### 3. Code Generator (`src/generator/`)
- Shell-to-Rust AST transformation
- Memory-embedded resource system
- Native cross-platform API generation
- Theme-aware UI component generation
- Configuration system generation (TOML format)
- Error handling and logging integration

### 4. Build System (`src/build/`)
- Cross-compilation for all major platforms
- Static linking with optimization
- Binary packaging and distribution
- Dependency bundling and compression

## File Classification Rules

### Always Runtime (Never Embed)
- System paths: `/proc/*`, `/sys/*`, `/dev/*`, `/tmp/*`, `/var/log/*`
- Cache directories: `~/.cache/*`, `$HOME/.local/tmp/*`
- Process files: `*.pid`, `*.lock`, `*.sock`
- Dynamic content: files modified by script
- Large files: > 50MB
- Sensitive files: `*.key`, `*.pem`, `*.password`
- Monitoring contexts: `tail -f`, `watch`, `inotifywait`

### Always Static (Embed)
- Local configs: `*.conf`, `*.config`, `settings.txt` in script directory
- Documentation: `README.*`, `LICENSE.*`, `*.md`
- Data files: `*.json`, `*.yaml`, `*.toml` < 1MB
- Templates and snippets
- Source files for inclusion

### Context Dependent
- Analyzed by usage pattern, file size, modification detection
- Loop context analysis
- Conditional existence checks
- Write-then-read patterns

## Shell Feature Support

### Parameter Expansion
- `${var}`, `${var:-default}`, `${var:+alt}`, `${var:?error}`
- `${var#pattern}`, `${var##pattern}`, `${var%pattern}`, `${var%%pattern}`
- `${var/pattern/replacement}`, `${var//pattern/replacement}`
- `${#var}`, `${!var}`, `${var:offset:length}`
- Array support: `${arr[@]}`, `${arr[*]}`, `${!arr[@]}`

### Command Features
- Command substitution: `$(command)`, `` `command` ``
- Process substitution: `<(command)`, `>(command)`
- Arithmetic expansion: `$((expression))`, `$[expression]`
- All redirection types: `>`, `>>`, `<`, `<<`, `<<<`, `<>`, `>&`, `|&`
- File descriptor manipulation
- Named pipes and FIFO handling

### Control Structures
- All conditionals: `if-elif-else`, `[[ ]]`, `[ ]`
- All loops: `for`, `while`, `until`, `select`, C-style `for`
- Case statements with pattern matching
- Functions with local variables and parameters
- Signal handling and trap commands

### Built-in Commands
Native Rust implementations for all shell built-ins:
- `cd`, `echo`, `printf`, `test`, `read`, `export`, `source`
- `pushd`, `popd`, `dirs`, `history`
- `jobs`, `bg`, `fg`, `kill`, `wait`
- Complete POSIX compliance

## Configuration System

### Default Configuration Structure (`settings.toml`)
```toml
[ui]
theme = "dracula"  # light|dark|dracula

[build]
targets = ["linux_amd64", "darwin_arm64", "windows_amd64"]
optimize = "size"  # size|speed|debug
strip = true
compress = true

[dependencies]
bundle_dirs = ["lib/", "common/"]
external_bins = ["jq", "curl", "git"]
follow_symlinks = false
max_source_depth = 15

[notifications]
enabled = true
desktop = true

[smtp]
enabled = false
server = "$HOSTNAME"
port = 465
from_email = "no-reply@$HOSTNAME"
from_name = "$USER"
to = ["root@$HOSTNAME"]
subject_prefix = "[{app_name}]"
username = ""
password = ""

[webhooks]
slack = ""
discord = ""
teams = ""
custom = []

[paths]
rust_src_dir = "rustsrc/"
build_dir = "target/"
output_dir = "dist/"

[security]
sandbox = false
validate_paths = true
confirm_deletions = true

[updates]
enabled = false  # Disabled by default
check_on_start = false
auto_download = false
channel = "stable"  # stable|beta|nightly
```

## CLI Interface

### Basic Commands
- `cassh2rs script.sh` - Generate Rust source in ./rustsrc/
- `cassh2rs script.sh --build` - Also compile binaries
- `cassh2rs mydir/` - Process directory (separate projects)
- `cassh2rs mydir/ --join` - Single app with subcommands
- `cassh2rs mydir/ --join script1` - Primary + subcommands

### Standard Flags
- `--build, -b` - Compile to binaries
- `--wizard, -w` - Interactive dependency resolution
- `--output, -o DIR` - Output directory
- `--help, -h` - Show help
- `--version, -V` - Show version
- `--verbose, -v` - Verbose output
- `--quiet, -q` - Suppress output
- `--config, -c FILE` - Config file
- `--dry-run, -n` - Show what would be done

### Advanced Flags
- `--secure` - Enable security mode
- `--watch` - Watch mode for development
- `--sandbox` - Sandbox execution
- `--join [primary]` - Multi-script handling
- `--release` - Release build mode
- `--enable-updates` - Enable update checking in release

## Theme System

### Built-in Themes
1. **DRACULA** (default): Dark purple/pink theme
2. **DARK**: Clean dark theme with blue accents
3. **LIGHT**: Professional light theme

### Theme Detection
- Auto-detect from user configs (.vimrc, VS Code settings)
- System dark/light mode detection
- Terminal color capability detection
- Generated apps include theme switching

### UI Components
- Native desktop notifications
- Progress bars and status indicators
- File dialogs and error messages
- Theme-aware syntax highlighting
- Responsive design for all screen sizes

## Cross-Platform Support

### Target Platforms
- Linux: x86_64, aarch64, armv7
- macOS: x86_64, aarch64 (Apple Silicon)
- Windows: x86_64, aarch64
- FreeBSD/OpenBSD: x86_64

### Platform-Specific Code Generation
```rust
#[cfg(target_os = "linux")]
mod linux_impl;
#[cfg(target_os = "windows")]
mod windows_impl;
#[cfg(target_os = "macos")]
mod macos_impl;
```

### Binary Naming Convention
Go-style naming: `{name}_{os}_{arch}`
- `cassh2rs_linux_amd64`
- `cassh2rs_darwin_arm64`
- `cassh2rs_windows_amd64.exe`
- For converted scripts: `scriptname_linux_amd64`

## Update System

### Environment Variables
- `SCRIPT_REPO` - Repository URL (mandatory or null/nil/none to disable)
  - Example: `github.com/user/repo`, `git.mysite.com/team/project`
  - Set to empty string, "null", "nil", or "none" to disable updates
- `RELEASE_API` - Optional custom API path override
  - Example: `/api/v2/releases/latest`

### Update URL Auto-Detection
The system automatically detects the repository type and constructs the appropriate API URL:

1. **GitHub**: `https://api.github.com/repos/{owner}/{repo}/releases/latest`
2. **GitLab**: `https://gitlab.com/api/v4/projects/{encoded_path}/releases`
3. **Gitea/Forgejo**: `https://{domain}/api/v1/repos/{owner}/{repo}/releases`

### Update Behavior
- **Build Time**: Validates update configuration and tests API endpoints
- **Runtime**: Fails silently if update checks fail
- **Git Detection**: Automatically detects git repositories during conversion
- **Default State**: Updates are disabled by default in config
- **Triggers**: Updates only check when:
  - `--update` flag is used
  - Manual update command is run
  - Release build with `--enable-updates`

### Build-Time Validation
```rust
// Test update API during cassh2rs conversion
fn validate_update_config(script_repo: &str, release_api: &str) -> Result<String, Error> {
    // Warns if disabled
    // Tests API endpoint if enabled
    // Requires RELEASE_API if auto-detection fails
}
```

## Dependency Management

### Smart Dependency Detection
- Static analysis of function calls and commands
- External binary detection and bundling
- Library requirement analysis
- Network resource identification

### Auto-Bundling
- Git operations: git2-rs library or git binary
- Network requests: reqwest crate
- JSON processing: serde_json crate
- File operations: std::fs
- Process management: std::process

### Optimization Strategies
- Dead code elimination
- Compile-time evaluation
- Resource deduplication
- Binary size minimization
- Performance optimization per platform

## Security Features

### Path Validation
Automatic injection of safety checks for rm operations:
- Block dangerous paths: `/`, `/home`, `/etc`, `/usr`, `/var`
- Require confirmation for home directory deletion
- Validate file existence and permissions
- Check write permissions before operations

### Remote Code Execution Handling
- Detect `curl|bash` patterns
- Sandbox execution options
- User confirmation for remote scripts
- Download and review workflow

### Sandboxing
- Optional restricted execution environment
- File system access limitations
- Network access controls
- System call filtering

## Notification System

### Native Notifications
- Desktop: notify-rust crate for cross-platform
- Email: lettre crate with SMTP support
- Webhooks: reqwest for HTTP POST
- Multiple simultaneous channels

### Generated Notification Code
```rust
fn send_notification(title: &str, message: &str) {
    if config.desktop {
        notify_rust::Notification::new()
            .summary(title)
            .body(message)
            .show().ok();
    }
    if !config.email.is_empty() {
        send_email(title, message);
    }
    if !config.webhooks.is_empty() {
        send_webhooks(title, message);
    }
}
```

## Version Management

### Auto-Version Generation
- Extract version from script headers
- Generate version.txt at build time
- Embed version in binary
- Runtime version file creation when needed

### Version Format
- Default: `YYYYMMDDHHMM-git`
- Configurable format strings
- Git commit integration
- Semantic versioning support

## Project Structure

### Generated Rust Project
```
rustsrc/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration handling
│   ├── notifications.rs     # Notification system
│   ├── shell_runtime.rs     # Shell environment simulation
│   ├── embedded_files.rs    # All sourced files as constants
│   ├── commands/            # Shell command implementations
│   ├── ui/                  # Theme and UI components
│   └── platform/            # Platform-specific code
├── Cargo.toml               # Dependencies and metadata
├── build.rs                 # Build script for embedding
├── settings.toml            # Runtime configuration
├── README.md                # Preserved or generated
├── LICENSE                  # Preserved or generated
└── .gitignore              # Generated
```

### Build Artifacts
```
dist/
├── cassh2rs_linux_amd64
├── cassh2rs_linux_arm64
├── cassh2rs_darwin_amd64
├── cassh2rs_darwin_arm64
├── cassh2rs_windows_amd64.exe
└── cassh2rs_windows_arm64.exe
```

## Testing Framework

### Generated Tests
- Behavior comparison tests (shell vs Rust)
- Performance benchmarks
- Cross-platform compatibility tests
- Regression test suite
- Property-based testing for shell semantics

### Test Structure
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_shell_compatibility() {
        // Compare shell and Rust outputs
    }
    
    #[test]
    fn test_file_operations() {
        // Test file handling
    }
}
```

## Error Handling

### Comprehensive Error System
- Parse errors with line/column information
- Missing dependency warnings
- Unsupported feature notifications
- Runtime error propagation
- Graceful degradation strategies

### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Parse error at line {line}, column {col}: {msg}")]
    ParseError { line: usize, col: usize, msg: String },
    #[error("Missing dependency: {dep}")]
    MissingDependency { dep: String },
    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature { feature: String },
}
```

## Wizard System

### Interactive Resolution
- Dynamic dependency resolution
- Ambiguous file handling
- Security decision prompts
- Configuration generation assistance

### Wizard Flow
```
🔍 Dynamic variable reference detected: ${!DB_*}
   Available variables: DB_HOST, DB_PORT, DB_USER
   Action: [Include all] [Select specific] [Skip] [Help]

🔍 External command: mysql --version
   Action: [Bundle binary] [System dependency] [Skip feature] [Help]

🔍 Remote execution: curl url | bash
   Action: [Download & verify] [Sandbox] [Block] [Help]
```

## Performance Optimizations

### Compile-Time Optimizations
- Constant folding for shell variables
- Dead code elimination
- Loop unrolling where beneficial
- Inline function expansion

### Runtime Optimizations
- Efficient string handling
- Memory-mapped file access
- Parallel execution where safe
- Lazy loading for large resources

## Development Workflow

### Watch Mode
- File system monitoring
- Automatic regeneration
- Hot reload for development
- Incremental compilation

### CI/CD Integration
- GitHub Actions templates
- Cross-platform build matrices
- Release automation
- Binary distribution

## Implementation Requirements

### Core Dependencies
- clap 4.x - CLI argument parsing
- serde 1.x - Configuration serialization
- tokio 1.x - Async runtime
- anyhow 1.x - Error handling
- thiserror 1.x - Error types
- walkdir 3.x - File system traversal
- regex 1.x - Pattern matching
- git2 0.18.x - Git operations
- reqwest 0.11.x - HTTP client
- notify-rust 4.x - Desktop notifications
- lettre 0.11.x - Email sending

### Development Tools
- cargo-cross - Cross compilation
- cargo-audit - Security auditing
- cargo-bloat - Binary size analysis
- cargo-flamegraph - Performance profiling

### Build Requirements
- Rust 1.70+ - Latest stable
- Cross-compilation toolchains
- Platform-specific linkers
- OpenSSL development libraries

### Runtime Requirements
Generated binaries have ZERO runtime requirements - completely self-contained.

## Success Criteria
- Converts 95%+ of real-world shell scripts
- Generated binaries run 10-100x faster than shell
- Zero false positives in dependency detection
- Cross-platform compatibility verification
- Production-ready error handling
- Beautiful, responsive UI across all platforms
- Complete documentation and examples
- Comprehensive test coverage
- Single-command build and deployment