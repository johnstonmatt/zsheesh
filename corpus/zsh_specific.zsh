#!/bin/zsh

# Parameter expansion flags (tree-sitter-bash treats as opaque expansion)
echo "${(k)my_hash}"
echo "${(v)my_hash}"
echo "${(@)my_array}"

# Setopt / unsetopt
setopt AUTO_CD
setopt EXTENDED_GLOB
unsetopt BEEP

# typeset variants
typeset -gA GLOBAL_HASH
typeset -U path

# Print with flags
print -r -- "raw output"
print -P "%F{red}colored%f"

# Glob qualifiers (appear as words to the parser)
ls *.txt

# Arithmetic
(( count++ ))
(( result = a + b * c ))
