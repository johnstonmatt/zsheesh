#!/bin/zsh

# Simple pipeline
cat /etc/passwd | grep root

# Multi-stage pipeline (two stages)
ps aux | grep root

# Pipeline with sort
ls -la | sort -k 5

# Command list with && and ||
mkdir -p /tmp/test && echo "created" || echo "failed"

# Background process
long_running_task &

# Subshell
(
  export PATH="/custom/bin:${PATH}"
  run_isolated_command
)
