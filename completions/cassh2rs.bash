#!/bin/bash
# Bash completion for cassh2rs

_cassh2rs() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Main options
    opts="--help --version --build --wizard --output --config --verbose --quiet \
          --dry-run --secure --watch --sandbox --join --release --enable-updates \
          --update init check features"
    
    # Options that require arguments
    case "${prev}" in
        --output|-o)
            # Complete with directories
            COMPREPLY=( $(compgen -d -- "${cur}") )
            return 0
            ;;
        --config|-c)
            # Complete with .toml files
            COMPREPLY=( $(compgen -f -X '!*.toml' -- "${cur}") )
            return 0
            ;;
        --join)
            # Complete with .sh files for primary script
            COMPREPLY=( $(compgen -f -X '!*.sh' -- "${cur}") )
            return 0
            ;;
        --shell)
            # Complete with shell types
            COMPREPLY=( $(compgen -W "bash zsh fish dash ksh tcsh csh powershell" -- "${cur}") )
            return 0
            ;;
        init)
            # Project name
            return 0
            ;;
        check)
            # Complete with .sh files
            COMPREPLY=( $(compgen -f -X '!*.sh' -- "${cur}") )
            return 0
            ;;
    esac
    
    # Complete options or files
    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
    else
        # Complete with shell script files and directories
        COMPREPLY=( $(compgen -f -X '!*.sh' -- "${cur}") )
        COMPREPLY+=( $(compgen -d -- "${cur}") )
    fi
}

complete -F _cassh2rs cassh2rs