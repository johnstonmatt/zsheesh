#!/bin/zsh

# Simple assignment
FOO="bar"
BAR='baz'

# Export
export PATH="/usr/local/bin:$PATH"

# Local
local my_var="hello"

# Arrays
typeset -a my_array
my_array=(one two three four)

# Associative array
typeset -A my_hash

# Readonly
readonly CONST="immutable"

# Variable expansion
echo "${FOO}"
echo "${FOO:-default}"
echo "${FOO:=default}"
echo "${#FOO}"
