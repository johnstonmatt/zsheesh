#!/bin/zsh

# Simple function with parens
greet() {
  echo "Hello, $1"
}

# Function with local variables
calculate() {
  local result=$(( $1 + $2 ))
  echo ${result}
}

# Nested function calls
outer() {
  inner() {
    echo "inner"
  }
  inner
  echo "outer"
}
