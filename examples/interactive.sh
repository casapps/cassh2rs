#!/bin/bash
# Interactive script that requires terminal

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}Welcome to the Interactive Demo${NC}"

# Read user input
echo -n "Enter your name: "
read NAME

# Password input
echo -n "Enter password: "
read -s PASSWORD
echo

# Menu selection
echo -e "\n${GREEN}Select an option:${NC}"
select OPTION in "Show info" "Run test" "Exit"; do
    case $OPTION in
        "Show info")
            echo -e "${GREEN}Hello, $NAME!${NC}"
            echo "Terminal size: $(tput cols) x $(tput lines)"
            ;;
        "Run test")
            # Progress bar simulation
            echo -n "Processing: "
            for i in {1..20}; do
                echo -n "#"
                sleep 0.1
            done
            echo " Done!"
            ;;
        "Exit")
            break
            ;;
    esac
done

# Clear screen
clear
echo "Goodbye!"