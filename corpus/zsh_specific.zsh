#!/bin/zsh

# Parameter expansion flags
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
((count++))
((result = a + b * c))

# &| background + disown
sleep 10 &|

# >! force-clobber redirect
echo data >! /tmp/output

# $+var parameter set check
if (($+commands[git])); then
  echo "git found"
fi

# ${+var} expansion set check
echo ${+PATH}
echo ${+commands[git]}

# Nested parameter expansion
0="${${ZERO:-foo}:-bar}"

# Triple-nested expansion
0="${${ZERO:-${0:#$ZSH_ARGZERO}}:-${(%):-%N}}"

# Expansion with :# operator
echo "${(M)0:#/*}"
echo "${var:#pattern}"

# for var (list) short form
for v (a b c); do
  echo ${v}
done

# function { } anonymous function
function {
  echo "anonymous"
}

# { } always { } try-always
{
  echo try
} always {
  echo cleanup
}

# Flags with string target
lines=(${(f)"$(git status)"})

# Flags with command substitution
a=(${(@f)"$(cmd)"})

# ${(%):-%N} prompt expansion (empty target)
echo "${(%):-%N}"

# :gs global substitution modifier
echo "${rvm_prompt:gs/%/%%}"

# :l lowercase modifier
echo ${issue_arg:l}

# Flag separator pattern
parts=(${(s:.:)HOST})
opt=(${(s:\t:)line})

# $#var array length
if (( $#remotes > 0 )); then
  echo "has remotes"
fi

# ${=var} word splitting
echo ${=icon:+--icon "$icon"}

# Subscript flags
echo ${available_profiles[(r)$1]}

# ${^var} distribute prefix
echo ${^PATH}
for file in "${^PYTHON_VENV_NAMES[@]}"/bin/activate; do
  echo "${file}"
done

# ${~var} glob prefix
echo ${~var}

# Flag separator with / delimiter: s/sep/
echo ${(@s/:/)var}

# Multi-character subscript flags (Ie), (Re)
if [[ ${tools[(Ie)$TOOL]} -eq 0 ]]; then
  echo "missing"
fi

# :| array difference operator
bundled=(${bundled:|UNBUNDLED})

# ${(flags)@} — @ as target variable
local query="${(j:,:)@}"

# ${${var}[-1]} — nested expansion with subscript
local word=${${(Az)LBUFFER}[-1]}

# ${"string"#pattern} — string in expansion body
local fzf_ver=${"$(fzf --version)"#fzf }

# ${$(cmd)#v} — command substitution in expansion
local nvm_prompt=${$(nvm current)#v}

# ${#${nested}} — length of nested expansion
echo ${#${var}}
echo ${#${=emotty}}

# ${@[2,-1]} — special variable with subscript
echo ${@[2,-1]}

# for k v in — multiple loop variables
for k v in a b c d; do
  echo "${k}" "${v}"
done

# if [[ ]] then — no semicolon before then
if [[ -z "$1" ]] then
  echo "empty"
fi

# function name1 name2 — multi-name function
function man foo {
  echo "shared body"
}

# :P modifier (path expansion)
echo ${commands[aws]:P}

# &>/dev/null <<< herestring after redirect
command grep -E 'test' &>/dev/null <<< "$status"
