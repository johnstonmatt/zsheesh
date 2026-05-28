#!/usr/bin/env bash
#
# validate-ohmyzsh.sh — run zsheesh against ohmyzsh/plugins and check for
# crashes or destructive formatting.
#
# Usage:
#   ./scripts/validate-ohmyzsh.sh [--keep]
#
# What it checks:
#   1. No crashes (exit code 0 or expected parse-error skip)
#   2. Idempotent (formatting twice = same output)
#   3. No destructive changes (formatted output still parses without new errors)
#
# Options:
#   --keep    Keep the cloned ohmyzsh repo after the run (default: clean up)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ZSHEESH="${REPO_ROOT}/target/release/zsheesh"
KEEP=false
TMPDIR=""

for arg in "$@"; do
  case "$arg" in
    --keep) KEEP=true ;;
  esac
done

cleanup() {
  if [[ "$KEEP" == false ]] && [[ -n "$TMPDIR" ]]; then
    rm -rf "$TMPDIR"
  fi
}
trap cleanup EXIT

# --- Build zsheesh ---

echo "==> Building zsheesh (release)"
(cd "$REPO_ROOT" && cargo build --release 2>&1 | tail -1)

if [[ ! -x "$ZSHEESH" ]]; then
  echo "Error: zsheesh binary not found at $ZSHEESH" >&2
  exit 1
fi

# --- Clone ohmyzsh ---

TMPDIR="$(mktemp -d)"
OMZ_DIR="$TMPDIR/ohmyzsh"

echo "==> Cloning ohmyzsh (shallow)"
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/ohmyzsh/ohmyzsh.git "$OMZ_DIR" 2>&1 | tail -1

(cd "$OMZ_DIR" && git sparse-checkout set plugins lib themes)

# --- Collect zsh files ---

FILES=()
while IFS= read -r -d '' f; do
  FILES+=("$f")
done < <(find "$OMZ_DIR" -type f \( -name "*.zsh" -o -name ".zshrc" \) -print0)

TOTAL=${#FILES[@]}
echo "==> Found $TOTAL zsh files"

# --- Run checks ---

CRASHES=0
DESTRUCTIVE=0
NOT_IDEMPOTENT=0
SKIPPED=0
FORMATTED=0
CRASH_FILES=()
DESTRUCTIVE_FILES=()
IDEMPOTENT_FILES=()

for f in "${FILES[@]}"; do
  rel="${f#$OMZ_DIR/}"

  # --- Check 1: Does it crash? ---
  first_pass=$("$ZSHEESH" --force "$f" 2>/tmp/zsheesh_stderr || true)
  exit_code=$?
  stderr=$(cat /tmp/zsheesh_stderr 2>/dev/null || true)

  # A crash is a signal death or unexpected exit
  if [[ $exit_code -gt 1 ]] && [[ $exit_code -ne 2 ]]; then
    echo "CRASH: $rel (exit $exit_code)"
    if [[ -n "$stderr" ]]; then
      echo "  stderr: $(head -3 <<< "$stderr")"
    fi
    CRASHES=$((CRASHES + 1))
    CRASH_FILES+=("$rel")
    continue
  fi

  # Check if it was skipped due to parse errors (without --force)
  skip_output=$("$ZSHEESH" "$f" 2>/tmp/zsheesh_stderr_nf || true)
  skip_stderr=$(cat /tmp/zsheesh_stderr_nf 2>/dev/null || true)
  if echo "$skip_stderr" | grep -q "skipped (parse errors)"; then
    SKIPPED=$((SKIPPED + 1))
    continue
  fi

  FORMATTED=$((FORMATTED + 1))

  # --- Check 2: Is it idempotent? ---
  second_pass=$(echo "$first_pass" | "$ZSHEESH" --force 2>/dev/null || true)

  if [[ "$first_pass" != "$second_pass" ]]; then
    echo "NOT IDEMPOTENT: $rel"
    NOT_IDEMPOTENT=$((NOT_IDEMPOTENT + 1))
    IDEMPOTENT_FILES+=("$rel")
    # Show first diff
    diff <(echo "$first_pass") <(echo "$second_pass") | head -20 || true
    echo "  ..."
  fi

  # --- Check 3: Does the formatted output introduce new parse errors? ---
  # Compare parse errors before and after
  original_errors=$("$ZSHEESH" --dump-ast "$f" 2>/dev/null | grep -c "ERROR\|MISSING" || true)
  formatted_errors=$(echo "$first_pass" | "$ZSHEESH" --dump-ast 2>/dev/null | grep -c "ERROR\|MISSING" || true)

  if [[ "$formatted_errors" -gt "$original_errors" ]]; then
    echo "DESTRUCTIVE: $rel (parse errors: $original_errors → $formatted_errors)"
    DESTRUCTIVE=$((DESTRUCTIVE + 1))
    DESTRUCTIVE_FILES+=("$rel")
  fi
done

rm -f /tmp/zsheesh_stderr /tmp/zsheesh_stderr_nf

# --- Report ---

echo ""
echo "============================================"
echo "  ohmyzsh validation report"
echo "============================================"
echo "  Total files:      $TOTAL"
echo "  Formatted:        $FORMATTED"
echo "  Skipped (parse):  $SKIPPED"
echo "  Crashes:          $CRASHES"
echo "  Not idempotent:   $NOT_IDEMPOTENT"
echo "  Destructive:      $DESTRUCTIVE"
echo "============================================"

if [[ ${#CRASH_FILES[@]} -gt 0 ]]; then
  echo ""
  echo "Crashed files:"
  printf "  %s\n" "${CRASH_FILES[@]}"
fi

if [[ ${#DESTRUCTIVE_FILES[@]} -gt 0 ]]; then
  echo ""
  echo "Destructive files:"
  printf "  %s\n" "${DESTRUCTIVE_FILES[@]}"
fi

if [[ ${#IDEMPOTENT_FILES[@]} -gt 0 ]]; then
  echo ""
  echo "Non-idempotent files:"
  printf "  %s\n" "${IDEMPOTENT_FILES[@]}"
fi

if [[ "$KEEP" == true ]]; then
  echo ""
  echo "ohmyzsh repo kept at: $OMZ_DIR"
fi

# Exit with failure if any crashes or destructive formatting
if [[ $CRASHES -gt 0 ]] || [[ $DESTRUCTIVE -gt 0 ]]; then
  exit 1
fi

echo ""
echo "All clear."
