#!/bin/bash
# Script that can run without a terminal

# Process files
for file in *.txt; do
    if [ -f "$file" ]; then
        # Count lines
        lines=$(wc -l < "$file")
        echo "$file: $lines lines"
    fi
done

# Write to log file
echo "$(date): Script completed" >> process.log

# Perform calculations
sum=0
for i in {1..100}; do
    ((sum += i))
done
echo "Sum: $sum"

# Check system info
echo "Hostname: $(hostname)"
echo "User: $USER"
echo "Current directory: $PWD"