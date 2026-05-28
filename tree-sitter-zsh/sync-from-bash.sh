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
#   1. Verifies the source version and integrity
#   2. Copies grammar.js from tree-sitter-bash
#   3. Applies zsh-specific patches (parameter expansion flags, &! operator)
#   4. Runs `tree-sitter generate` to rebuild the parser
#   5. Copies and renames scanner.c symbols
#
# Requirements:
#   - tree-sitter CLI (`cargo install tree-sitter-cli`)
#   - The source must contain grammar.js and src/scanner.c
#   - Node.js (for patch-grammar.js)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Known good upstream versions and their grammar.js SHA-256 checksums.
# Add new entries when upgrading to a new tree-sitter-bash release.
declare -A KNOWN_CHECKSUMS=(
  ["0.25.1"]="3c125330d995968a7e3a4fc71617aa73472449f5a6c254f39da2d8860a47bf65"
)

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

# --- Integrity check ---

ACTUAL_SHA=$(shasum -a 256 "$BASH_DIR/grammar.js" | cut -d' ' -f1)

# Try to detect the version from Cargo.toml or package.json
DETECTED_VERSION=""
if [[ -f "$BASH_DIR/Cargo.toml" ]]; then
  DETECTED_VERSION=$(grep '^version' "$BASH_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
elif [[ -f "$BASH_DIR/package.json" ]]; then
  DETECTED_VERSION=$(grep '"version"' "$BASH_DIR/package.json" | head -1 | sed 's/.*"\([0-9][^"]*\)".*/\1/')
fi

EXPECTED_SHA=""
if [[ -n "$DETECTED_VERSION" ]] && [[ -n "${KNOWN_CHECKSUMS[$DETECTED_VERSION]+x}" ]]; then
  EXPECTED_SHA="${KNOWN_CHECKSUMS[$DETECTED_VERSION]}"
fi

if [[ -n "$EXPECTED_SHA" ]]; then
  if [[ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]]; then
    echo "Error: grammar.js checksum mismatch for tree-sitter-bash v${DETECTED_VERSION}" >&2
    echo "  Expected: ${EXPECTED_SHA}" >&2
    echo "  Actual:   ${ACTUAL_SHA}" >&2
    echo "" >&2
    echo "The file may have been tampered with, or the version detection is wrong." >&2
    echo "If this is a new version, add its checksum to KNOWN_CHECKSUMS in this script." >&2
    exit 1
  fi
  echo "==> Verified: tree-sitter-bash v${DETECTED_VERSION} (SHA-256 matches)"
else
  echo "==> Warning: unknown version '${DETECTED_VERSION:-<undetected>}'" >&2
  echo "   grammar.js SHA-256: ${ACTUAL_SHA}" >&2
  echo "   To approve this version, add it to KNOWN_CHECKSUMS in sync-from-bash.sh" >&2
  echo "" >&2
  read -p "Continue with unverified source? [y/N] " -n 1 -r
  echo
  if [[ ! "$REPLY" =~ ^[Yy]$ ]]; then
    echo "Aborted." >&2
    exit 1
  fi
fi

# --- Sync ---

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
