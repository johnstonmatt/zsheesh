#!/bin/zsh

# Simple pipeline
cat /etc/passwd | grep root

# Multi-stage pipeline
ps aux | grep nginx | grep -v grep | awk '{print $2}'

# Pipeline with redirection
cat input.txt | sort | uniq > output.txt

# Command list with && and ||
mkdir -p /tmp/test && echo "created" || echo "failed"

# Background process
long_running_task &

# Subshell
(
  export PATH="/custom/bin:$PATH"
  run_isolated_command
)
