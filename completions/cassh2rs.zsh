#compdef cassh2rs
# Zsh completion for cassh2rs

_cassh2rs() {
    local -a options
    local -a subcommands
    
    options=(
        '(-h --help)'{-h,--help}'[Show help information]'
        '(-V --version)'{-V,--version}'[Show version information]'
        '(-b --build)'{-b,--build}'[Build binaries after generating Rust source]'
        '(-w --wizard)'{-w,--wizard}'[Interactive wizard for dependency resolution]'
        '(-o --output)'{-o,--output}'[Output directory]:directory:_directories'
        '(-c --config)'{-c,--config}'[Configuration file]:config file:_files -g "*.toml"'
        '(-v --verbose)'{-v,--verbose}'[Verbose output]'
        '(-q --quiet)'{-q,--quiet}'[Suppress output]'
        '(-n --dry-run)'{-n,--dry-run}'[Show what would be done without making changes]'
        '--secure[Enable security mode]'
        '--watch[Watch mode for development]'
        '--sandbox[Sandbox execution]'
        '--join[Join multiple scripts into single app]::primary script:_files -g "*.sh"'
        '--release[Release build mode]'
        '--enable-updates[Enable update checking in release builds]'
        '(-U --update)'{-U,--update}'[Check for updates]'
    )
    
    subcommands=(
        'init:Initialize a new cassh2rs project'
        'check:Validate shell scripts without converting'
        'features:Show supported shell features'
    )
    
    # First argument handling
    if (( CURRENT == 2 )); then
        _describe -t subcommands 'subcommand' subcommands
        _files -g '*.sh'
        _directories
    elif (( CURRENT == 3 )); then
        case "${words[2]}" in
            init)
                _message 'project name'
                ;;
            check)
                _files -g '*.sh'
                ;;
            features)
                _arguments '--shell[Filter by shell dialect]:shell:(bash zsh fish dash ksh tcsh csh powershell)'
                ;;
            *)
                _arguments $options
                ;;
        esac
    else
        _arguments $options
    fi
}

_cassh2rs "$@"