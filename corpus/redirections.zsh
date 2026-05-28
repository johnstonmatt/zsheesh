#!/bin/zsh

# Output redirect
echo "hello" > output.txt

# Append redirect
echo "world" >> output.txt

# Input redirect
sort < input.txt

# Stderr redirect
command 2>/dev/null

# Redirect both stdout and stderr
command > output.txt 2>&1

# Herestring
grep -q "pattern" <<< "$string_var"
