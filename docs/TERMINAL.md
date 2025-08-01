# Terminal Support Documentation

This document consolidates all terminal-related documentation for cassh2rs.

## Terminal Detection

cassh2rs automatically analyzes scripts to determine their terminal requirements:

- **Headless**: Scripts that can run without any terminal
- **Terminal Features**: Scripts using colors, cursor control, or terminal size
- **Interactive**: Scripts requiring user input (read, select, passwords)
- **Full TUI**: Scripts using dialog, whiptail, or other TUI programs

## Automatic Adaptation

Generated binaries automatically detect their environment and adapt:

### In a Terminal
- Full interactive experience with colors and prompts
- Password masking for secure input
- Interactive menus and selections
- Colored output and cursor control

### Piped/Redirected
- Automatically switches to non-interactive mode
- Reads input from stdin without prompts
- Strips ANSI color codes from output
- No terminal control sequences

### GUI Launch (Double-clicked)
- Detects non-terminal launch
- Automatically opens in appropriate terminal:
  - **macOS**: Terminal.app
  - **Windows**: cmd.exe (stays open)
  - **Linux**: gnome-terminal, konsole, xterm, etc.
- Provides full interactive experience

## How It Works

### Detection Logic
```rust
let is_terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
```

### Architecture
- No custom terminal required
- Uses standard system terminals
- Sends standard ANSI escape codes
- Works with SSH, Docker, CI/CD

### Terminal Libraries Used
Based on script requirements, these crates may be included:
- `colored` - ANSI color codes
- `crossterm` - Cross-platform terminal manipulation
- `dialoguer` - Interactive prompts and menus
- `indicatif` - Progress bars
- `ratatui` - Full TUI applications

## Examples

### Interactive Script
```bash
#!/bin/bash
read -p "Name: " name
read -s -p "Password: " pass
echo -e "\033[32mHello $name!\033[0m"
```

### Usage
```bash
# Interactive mode (automatic)
./converted_script

# Non-interactive mode (automatic)
echo -e "John\nsecret" | ./converted_script

# GUI double-click (automatic terminal open)
# Just double-click in file manager!
```

## Benefits

1. **Zero Configuration** - No flags or options needed
2. **Universal Compatibility** - Works with any terminal
3. **Automatic Behavior** - Detects and adapts to environment
4. **Professional Output** - Proper colors, prompts, and formatting
5. **CI/CD Ready** - Works in automated pipelines

## Technical Details

The generated Rust code includes automatic detection and adaptation:

```rust
// Automatic terminal detection
if !is_terminal && std::env::var("TERM").is_err() {
    // GUI launch - reopen in terminal
}

// Adaptive input
if is_terminal {
    // Interactive prompt
    dialoguer::Input::new().with_prompt("Name").interact_text()?
} else {
    // Read from stdin
    stdin().read_line(&mut input)?
}
```

This ensures scripts work correctly in all environments without user intervention.