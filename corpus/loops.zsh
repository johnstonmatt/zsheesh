#!/bin/zsh

# For loop
for file in *.txt; do
  echo "Processing $file"
done

# For loop with list
for color in red green blue; do
  echo ${color}
done

# While loop
while read -r line; do
  echo "$line"
done
<input.txt

# C-style for loop
for (( i=0; i < 10; i++ )); do
  echo ${i}
done

# Until loop
until [ -f /tmp/done ]; do
  sleep 1
done
