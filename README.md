# cassh2rs - Universal Shell Script to Rust Converter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Convert shell scripts from any dialect (bash, zsh, fish, etc.) into fast, portable, self-contained Rust binaries.

## Features

- 🚀 **10-100x Performance** - Compiled Rust runs much faster than interpreted shell scripts
- 🌍 **Cross-Platform** - Generate binaries for Linux, macOS, and Windows
- 📦 **Zero Dependencies** - Self-contained executables that run anywhere
- 🎯 **Smart Terminal Detection** - Automatically adapts to environment (GUI, terminal, pipes)
- 🔧 **Multi-Shell Support** - Bash, Zsh, Fish, Dash, Ksh, Tcsh, Csh, PowerShell
- 🛡️ **Security First** - Built-in safety checks and sandboxing options
- 🎨 **Interactive Wizard** - Helps resolve dependencies and configuration

## Quick Start

```bash
# Install cassh2rs
curl -fsSL https://raw.githubusercontent.com/casapps/cassh2rs/main/scripts/install.sh | bash

# Convert a script
cassh2rs script.sh

# Convert and build
cassh2rs script.sh --build

# Check terminal requirements
cassh2rs check script.sh
```

## Terminal Detection

cassh2rs automatically detects terminal requirements and adapts:

- **Headless**: Scripts that can run without any terminal
- **Terminal Features**: Scripts using colors, cursor control, or terminal size
- **Interactive**: Scripts requiring user input (read, select, passwords)
- **Full TUI**: Scripts using dialog, whiptail, or other TUI programs

Generated binaries automatically:
- In a terminal: Full interactive experience with colors and prompts
- In a pipe/redirect: Automatically switches to non-interactive mode
- Double-clicked from GUI: Opens in a terminal window automatically
- No flags needed - it just works!

## Usage

### Basic Conversion

```bash
# Convert a single script
cassh2rs script.sh

# Convert and build binaries
cassh2rs script.sh --build

# Convert with interactive dependency resolution
cassh2rs script.sh --wizard
```

### Directory Conversion

```bash
# Convert all scripts in a directory (separate projects)
cassh2rs mydir/

# Join scripts into single app with subcommands
cassh2rs mydir/ --join

# Specify primary script when joining
cassh2rs mydir/ --join main.sh
```

### Advanced Options

```bash
# Check script for terminal requirements
cassh2rs check script.sh

# Enable security mode
cassh2rs script.sh --secure

# Watch mode for development
cassh2rs script.sh --watch

# Generate cross-platform binaries
cassh2rs script.sh --build --release
```

## Configuration

Create a `settings.toml` file for custom settings:

```toml
[ui]
theme = "dracula"  # light|dark|dracula

[build]
targets = ["linux_amd64", "darwin_arm64", "windows_amd64"]
optimize = "size"  # size|speed|debug
strip = true
compress = true

[updates]
enabled = false
check_on_start = false
auto_download = false
```

## Shell Feature Support

Run `cassh2rs features` to see the full compatibility matrix for different shell dialects.

## Generated Project Structure

```
rustsrc/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration handling
│   ├── shell_runtime.rs     # Shell environment simulation
│   └── commands/            # Shell command implementations
├── Cargo.toml               # Rust dependencies
└── settings.toml            # Runtime configuration
```

## Binary Naming Convention

Generated binaries follow Go-style naming:
- `scriptname_linux_amd64`
- `scriptname_darwin_arm64`
- `scriptname_windows_amd64.exe`

## Contributing

Contributions are welcome! Please read our [contributing guidelines](CONTRIBUTING.md) and submit pull requests to our repository.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Repository

https://github.com/casapps/cassh2rs