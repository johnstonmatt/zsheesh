#!/bin/zsh

# Normal code that gets formatted
echo "hello"
echo "world"

# fmt: skip
echo    "this    line    is   preserved"

# Normal code resumes
if [ -f /tmp/test ]; then
  echo "found"
fi

# fmt: off
echo    "this    region"
echo    "is   not   formatted"
if   [   -f  /tmp/test   ];   then
echo    "weird    spacing    preserved"
fi
# fmt: on

# Back to normal formatting
for f in *.txt; do
  echo "$f"
done
