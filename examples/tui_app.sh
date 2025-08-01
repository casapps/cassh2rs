#!/bin/bash
# Full TUI application example

# Use dialog for a menu-driven interface
if command -v dialog >/dev/null 2>&1; then
    # Dialog-based TUI
    while true; do
        CHOICE=$(dialog --clear \
                        --backtitle "System Manager" \
                        --title "Main Menu" \
                        --menu "Choose an option:" \
                        15 40 4 \
                        1 "System Information" \
                        2 "Process Manager" \
                        3 "File Browser" \
                        4 "Exit" \
                        2>&1 >/dev/tty)
        
        case $CHOICE in
            1)
                dialog --msgbox "OS: $(uname -s)\nKernel: $(uname -r)\nUptime: $(uptime)" 10 50
                ;;
            2)
                ps aux | dialog --programbox "Process List" 20 70
                ;;
            3)
                FILE=$(dialog --fselect $HOME/ 14 48 2>&1 >/dev/tty)
                dialog --msgbox "You selected: $FILE" 8 50
                ;;
            4)
                clear
                exit 0
                ;;
        esac
    done
elif command -v whiptail >/dev/null 2>&1; then
    # Whiptail alternative
    whiptail --title "TUI App" --msgbox "This is a terminal UI application" 8 45
else
    # Fallback to vim
    vim -c "echo 'TUI mode'" -c "q"
fi