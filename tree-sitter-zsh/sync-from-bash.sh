#!/usr/bin/env bash
#
# sync-from-bash.sh — rebuild tree-sitter-zsh from a tree-sitter-bash source
#
# Usage:
#   ./sync-from-bash.sh <path-to-tree-sitter-bash>
#
# Example:
#   ./sync-from-bash.sh ../node_modules/tree-sitter-bash
#   ./sync-from-bash.sh ~/.cargo/registry/src/*/tree-sitter-bash-0.25.1
#
# What it does:
#   1. Copies grammar.js from tree-sitter-bash
#   2. Applies zsh-specific patches (parameter expansion flags, &! operator)
#   3. Runs `tree-sitter generate` to rebuild the parser
#   4. Copies and renames scanner.c symbols
#
# Requirements:
#   - tree-sitter CLI (`cargo install tree-sitter-cli`)
#   - The source must contain grammar.js and src/scanner.c

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <path-to-tree-sitter-bash>" >&2
  exit 1
fi

BASH_DIR="$1"

if [[ ! -f "$BASH_DIR/grammar.js" ]]; then
  echo "Error: $BASH_DIR/grammar.js not found" >&2
  exit 1
fi

if [[ ! -f "$BASH_DIR/src/scanner.c" ]]; then
  echo "Error: $BASH_DIR/src/scanner.c not found" >&2
  exit 1
fi

echo "==> Copying grammar.js from $BASH_DIR"
cp "$BASH_DIR/grammar.js" "$SCRIPT_DIR/grammar.js"

echo "==> Applying zsh patches to grammar.js"
node "$SCRIPT_DIR/patch-grammar.js" "$SCRIPT_DIR/grammar.js"

echo "==> Running tree-sitter generate"
(cd "$SCRIPT_DIR" && tree-sitter generate)

echo "==> Copying and patching scanner.c"
cp "$BASH_DIR/src/scanner.c" "$SCRIPT_DIR/src/scanner.c"
sed -i 's/tree_sitter_bash/tree_sitter_zsh/g' "$SCRIPT_DIR/src/scanner.c"

echo "==> Done. Verify with: cd $SCRIPT_DIR && tree-sitter parse <(echo 'echo \${(k)hash}')"
