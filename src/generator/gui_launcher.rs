use anyhow::Result;

/// Generate a launcher script that ensures terminal stays open
pub fn generate_launcher_script(binary_name: &str, platform: &str) -> Result<String> {
    match platform {
        "macos" => Ok(format!(
            r#"#!/bin/bash
# Launcher for {}
osascript -e 'tell app "Terminal" to do script "cd \"$(dirname \"$0\")\"; ./{}; echo; echo \"Press any key to exit...\"; read -n 1"'
"#,
            binary_name, binary_name
        )),
        
        "windows" => Ok(format!(
            r#"@echo off
rem Launcher for {}
start cmd /k "cd /d %~dp0 && {} && echo. && pause"
"#,
            binary_name, binary_name
        )),
        
        "linux" => Ok(format!(
            r#"#!/bin/bash
# Launcher for {}
if command -v gnome-terminal >/dev/null; then
    gnome-terminal -- bash -c "cd \"$(dirname \"$0\")\"; ./{}; echo; echo \"Press Enter to exit...\"; read"
elif command -v konsole >/dev/null; then
    konsole -e bash -c "cd \"$(dirname \"$0\")\"; ./{}; echo; echo \"Press Enter to exit...\"; read"
elif command -v xterm >/dev/null; then
    xterm -hold -e "cd \"$(dirname \"$0\")\"; ./{}"
else
    # Fallback: try to launch in current terminal
    cd "$(dirname "$0")"
    ./{}
    echo
    echo "Press Enter to exit..."
    read
fi
"#,
            binary_name, binary_name, binary_name, binary_name, binary_name
        )),
        
        _ => Ok(format!(
            r#"#!/bin/sh
# Generic launcher for {}
./{} || echo "Error: Failed to run {}"
echo "Press Enter to exit..."
read dummy
"#,
            binary_name, binary_name, binary_name
        ))
    }
}

/// Generate a desktop entry file for Linux
pub fn generate_desktop_entry(app_name: &str, binary_path: &str, description: &str) -> String {
    format!(
        r#"[Desktop Entry]
Version=1.0
Type=Application
Name={}
Comment={}
Exec={}
Terminal=true
Icon=utilities-terminal
Categories=Utility;ConsoleOnly;
"#,
        app_name, description, binary_path
    )
}

/// Generate macOS app bundle structure
pub fn generate_macos_app_bundle(app_name: &str, binary_name: &str) -> Result<Vec<(String, String)>> {
    let mut files = Vec::new();
    
    // Info.plist
    files.push((
        "Contents/Info.plist".to_string(),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>com.cassh2rs.{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>"#,
            binary_name, 
            app_name.to_lowercase().replace(" ", "-"),
            app_name
        )
    ));
    
    // Launcher script
    files.push((
        "Contents/MacOS/launcher".to_string(),
        format!(
            r#"#!/bin/bash
cd "$(dirname "$0")"
osascript -e 'tell app "Terminal" to do script "cd \"'$(pwd)'\"; ./{}; echo; echo \"Press any key to exit...\"; read -n 1"'
"#,
            binary_name
        )
    ));
    
    Ok(files)
}