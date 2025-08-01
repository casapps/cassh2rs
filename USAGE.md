# cassh2rs Usage Guide

This guide demonstrates how to use cassh2rs to convert shell scripts into self-contained Rust binaries.

## Quick Start

### Basic Conversion

Convert a simple shell script to Rust:

```bash
# Convert script.sh to Rust source
cassh2rs script.sh

# Convert and build binary
cassh2rs script.sh --build

# Build optimized release binary
cassh2rs script.sh --build --release
```

### Interactive Mode

Use the wizard for dependency resolution:

```bash
cassh2rs deploy.sh --wizard
```

The wizard will guide you through:
- File dependency classification (embed vs runtime)
- External command handling (bundle vs system)
- Network resource management
- Security checks

### Watch Mode

Automatically rebuild when files change:

```bash
cassh2rs script.sh --watch --build
```

## Examples

### 1. Simple Script Conversion

Given `hello.sh`:
```bash
#!/bin/bash
echo "Hello, World!"
NAME="${1:-User}"
echo "Welcome, $NAME!"
```

Convert with:
```bash
cassh2rs hello.sh --build
```

This generates:
- `rustsrc/src/main.rs` - Rust implementation
- `rustsrc/target/release/hello` - Compiled binary

### 2. Script with Dependencies

Given `deploy.sh` (see examples/deploy.sh):
```bash
#!/bin/bash
# @Version: 1.0.0
# @Dependency: git
# @Dependency: docker

# ... deployment logic ...
```

Convert with wizard:
```bash
cassh2rs deploy.sh --wizard --build
```

The wizard will ask about:
- How to handle git/docker commands
- Whether to bundle or require system installation
- Configuration file handling

### 3. Cross-Platform Build

Build for multiple platforms:

```bash
# Edit settings.toml to specify targets
[build]
targets = ["linux_amd64", "darwin_arm64", "windows_amd64"]

# Build all targets
cassh2rs script.sh --build --release
```

Output binaries:
- `dist/script_linux_amd64`
- `dist/script_darwin_arm64`
- `dist/script_windows_amd64.exe`

### 4. Self-Updating Scripts

Enable self-updates for deployed scripts:

```bash
# Set repository during build
SCRIPT_REPO=github.com/myuser/myrepo cassh2rs script.sh --build --enable-updates

# Or for custom Git hosting
SCRIPT_REPO=git.company.com/team/project RELEASE_API=/api/v1/releases cassh2rs script.sh --build --enable-updates
```

### 5. Directory Processing

Convert all scripts in a directory:

```bash
# Separate projects for each script
cassh2rs scripts/

# Single app with subcommands
cassh2rs scripts/ --join

# Specify primary script
cassh2rs scripts/ --join main.sh
```

## Configuration

### Project Settings (settings.toml)

```toml
[ui]
theme = "dracula"  # light, dark, dracula

[build]
targets = ["linux_amd64", "darwin_arm64"]
optimize = "size"  # size, speed, debug
strip = true
compress = false

[notifications]
enabled = true
desktop = true

[smtp]
enabled = true
server = "smtp.gmail.com"
port = 587
from_email = "alerts@example.com"
to = ["admin@example.com"]

[updates]
enabled = false  # Enable for production releases
check_on_start = false
auto_download = false
```

### Environment Variables

- `SCRIPT_REPO` - Repository URL for updates
- `RELEASE_API` - Custom API endpoint
- `CONFIG_PATH` - Override settings.toml location

## Advanced Features

### File Classification

cassh2rs automatically classifies files:

**Always Embedded**:
- Local config files (*.conf, *.toml, *.json)
- Documentation (README, LICENSE)
- Templates and static data

**Always Runtime**:
- System files (/proc/*, /sys/*, /dev/*)
- Cache directories
- Files modified by script
- Large files (>50MB)

**Context-Dependent**:
- Analyzed based on usage patterns
- Wizard helps resolve ambiguous cases

### Shell Feature Support

Run `cassh2rs features` to see supported features:

```bash
cassh2rs features --shell bash
```

### Security Features

**Path Validation**:
- Dangerous rm operations blocked
- Sensitive path access warnings
- Sandbox mode available

**Remote Code Protection**:
- Detection of curl|bash patterns
- Optional blocking of remote execution
- Download verification workflows

### Notifications

Generated apps support multiple notification channels:

```rust
// Desktop notification
send_notification("Build Complete", "Successfully built project");

// Email notification (if configured)
send_email("Deployment Failed", "Error details...");

// Webhook notifications
send_webhook("slack", json!({
    "text": "Deployment successful"
}));
```

## Troubleshooting

### Common Issues

1. **Parse Errors**
   ```
   Error: Parse error at line 10, column 5
   ```
   Solution: Check for unsupported shell syntax

2. **Missing Dependencies**
   ```
   Error: Missing dependency: jq
   ```
   Solution: Use `--wizard` to configure handling

3. **Build Failures**
   ```
   Error: Failed to build: cargo not found
   ```
   Solution: Install Rust toolchain

### Debug Mode

Enable verbose output:
```bash
cassh2rs script.sh --verbose
```

Dry run (no files written):
```bash
cassh2rs script.sh --dry-run
```

## Best Practices

1. **Add Metadata**: Include version and dependencies in script headers
   ```bash
   #!/bin/bash
   # @Version: 1.0.0
   # @Author: Your Name
   # @Dependency: curl
   ```

2. **Use Local Configs**: Keep configuration files near scripts for embedding

3. **Test Conversions**: Use `--dry-run` first to preview changes

4. **Version Control**: Commit both shell scripts and generated Rust code

5. **Cross-Platform**: Test on target platforms before deployment

## Terminal Detection

cassh2rs automatically detects terminal requirements in your scripts:

```bash
# Check terminal requirements
cassh2rs check script.sh
```

Example output:
```
✓ script.sh - Valid Bash script
  Terminal: Requires terminal (interactive)
  Interactive commands: read, select
  Terminal crates needed: dialoguer, colored
```

Generated binaries automatically adapt to their environment:
```bash
# Run interactively (colors, prompts, menus)
./converted_script

# Automatic non-interactive mode (when piped or redirected)
echo "input_data" | ./converted_script
./converted_script < input.txt > output.txt

# Works in CI/CD without any special flags
./converted_script
```

## Performance

Converted binaries typically show:
- 10-100x faster execution than shell scripts
- Zero startup overhead (no shell interpreter)
- Efficient memory usage
- Native performance for file operations

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Support

- GitHub Issues: https://github.com/casapps/cassh2rs/issues
- Documentation: https://github.com/casapps/cassh2rs/wiki