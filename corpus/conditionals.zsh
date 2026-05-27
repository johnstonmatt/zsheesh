#!/bin/zsh

# Basic if
if [ -f ~/.zshrc ]; then
  source ~/.zshrc
fi

# If-elif-else
if [ "$1" = "start" ]; then
  echo "starting"
elif [ "$1" = "stop" ]; then
  echo "stopping"
else
  echo "usage: $0 {start|stop}"
fi

# Nested if
if [ -d /tmp ]; then
  if [ -w /tmp ]; then
    echo "writable tmp"
  fi
fi

# Test with double brackets
if [[ -n "${VAR}" ]]; then
  echo "VAR is set"
fi
