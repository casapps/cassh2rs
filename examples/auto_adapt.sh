#!/bin/bash
# Demo script showing automatic terminal adaptation

# Colors that work in terminal, plain text otherwise
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}Automatic Terminal Detection Demo${NC}"
echo

# This will use interactive prompt in terminal, read from stdin otherwise
echo "What's your name?"
read -p "Name: " NAME

echo -e "${GREEN}Hello, $NAME!${NC}"

# Password input - masked in terminal, plain in pipes
read -s -p "Enter a secret: " SECRET
echo

# Menu selection - interactive in terminal, reads number in pipes
echo "Choose an option:"
select CHOICE in "Option A" "Option B" "Exit"; do
    case $CHOICE in
        "Option A")
            echo -e "${GREEN}You chose A${NC}"
            break
            ;;
        "Option B")
            echo -e "${GREEN}You chose B${NC}"
            break
            ;;
        "Exit")
            break
            ;;
    esac
done

echo "Done!"