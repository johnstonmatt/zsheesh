#!/bin/zsh

# History configuration
HISTFILE=~/.zsh_history
HISTSIZE=10000
SAVEHIST=10000
setopt SHARE_HISTORY
setopt HIST_IGNORE_DUPS
setopt HIST_IGNORE_SPACE

# Directory navigation
setopt AUTO_CD
setopt AUTO_PUSHD
setopt PUSHD_IGNORE_DUPS

# Completion
autoload -Uz compinit
compinit

# Key bindings
bindkey -e
bindkey '^R' history-incremental-search-backward
bindkey '^P' up-history
bindkey '^N' down-history

# Aliases
alias ll='ls -la'
alias la='ls -A'
alias l='ls -CF'
alias ..='cd ..'
alias ...='cd ../..'

# Functions
mkcd() {
  mkdir -p "$1" && cd "$1"
}

extract() {
  if [ -f "$1" ]; then
    case "$1" in
      *.tar.bz2)
        tar xjf "$1";;
      *.tar.gz)
        tar xzf "$1";;
      *.bz2)
        bunzip2 "$1";;
      *.rar)
        unrar x "$1";;
      *.gz)
        gunzip "$1";;
      *.tar)
        tar xf "$1";;
      *.tbz2)
        tar xjf "$1";;
      *.tgz)
        tar xzf "$1";;
      *.zip)
        unzip "$1";;
      *.Z)
        uncompress "$1";;
      *.7z)
        7z x "$1";;
      *)
        echo "'$1' cannot be extracted"
        ;;
    esac
  else
    echo "'$1' is not a valid file"
  fi
}

# PATH setup
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/bin:$PATH"

# Prompt
export PS1='%n@%m:%~%# '

# Load local config if it exists
if [ -f ~/.zshrc.local ]; then
  source ~/.zshrc.local
fi
