#!/bin/zsh

# Output redirect
echo "hello" >output.txt

# Append redirect
echo "world" >>output.txt

# Input redirect
sort <input.txt

# Stderr redirect
command 2>/dev/null

# Redirect both stdout and stderr
command >output.txt 2>&1

# Herestring
grep -q "pattern" <<< "$string_var"

# Process substitution as a redirect target — the space is load-bearing
cat < <(generate)
tee_out > >(tee log)

# Redirects on compound statements stay on the closing keyword's line
while read -r line; do
  print "$line"
done < <(git worktree list --porcelain)

while read -r row; do
  print "$row"
done <"$exclude"

for f in a b c; do
  print "$f"
done >out.txt

if true; then
  print x
fi >out.txt

case ${x} in
  a)
    print y;;
esac >out.txt

f() { print a; } >out.txt
