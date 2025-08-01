# Fish completion for cassh2rs

# Disable file completion for certain options
complete -c cassh2rs -f

# Main options
complete -c cassh2rs -s h -l help -d "Show help information"
complete -c cassh2rs -s V -l version -d "Show version information"
complete -c cassh2rs -s b -l build -d "Build binaries after generating Rust source"
complete -c cassh2rs -s w -l wizard -d "Interactive wizard for dependency resolution"
complete -c cassh2rs -s o -l output -d "Output directory" -r -F
complete -c cassh2rs -s c -l config -d "Configuration file" -r -F
complete -c cassh2rs -s v -l verbose -d "Verbose output"
complete -c cassh2rs -s q -l quiet -d "Suppress output"
complete -c cassh2rs -s n -l dry-run -d "Show what would be done without making changes"
complete -c cassh2rs -l secure -d "Enable security mode"
complete -c cassh2rs -l watch -d "Watch mode for development"
complete -c cassh2rs -l sandbox -d "Sandbox execution"
complete -c cassh2rs -l join -d "Join multiple scripts into single app" -r -F
complete -c cassh2rs -l release -d "Release build mode"
complete -c cassh2rs -l enable-updates -d "Enable update checking in release builds"
complete -c cassh2rs -s U -l update -d "Check for updates"

# Subcommands
complete -c cassh2rs -n __fish_use_subcommand -a init -d "Initialize a new cassh2rs project"
complete -c cassh2rs -n __fish_use_subcommand -a check -d "Validate shell scripts without converting"
complete -c cassh2rs -n __fish_use_subcommand -a features -d "Show supported shell features"

# Shell types for features subcommand
complete -c cassh2rs -n "__fish_seen_subcommand_from features" -l shell -d "Filter by shell dialect" \
    -a "bash zsh fish dash ksh tcsh csh powershell"

# Enable file completion for script arguments
complete -c cassh2rs -n "not __fish_seen_subcommand_from init features" -F -k -a '*.sh'