#!/usr/bin/env bash
# Reject dependencies published less than N days ago.
# Protects against supply chain attacks using freshly-published crates.
#
# Usage: ./scripts/check-dep-age.sh [--min-days N]
#
# Default: 30 days

set -euo pipefail

MIN_DAYS="${1:-30}"
if [[ "$1" == "--min-days" ]]; then
  MIN_DAYS="${2:-30}"
fi

NOW=$(date +%s)
MIN_AGE_SECS=$((MIN_DAYS * 86400))
FAILED=0

echo "Checking all dependencies are at least ${MIN_DAYS} days old..."

# Get all non-workspace dependencies from Cargo.lock
cargo metadata --format-version 1 --no-deps 2>/dev/null \
  | jq -r '.packages[] | .name' > /tmp/workspace_crates.txt

cargo metadata --format-version 1 2>/dev/null \
  | jq -r '.packages[] | "\(.name) \(.version)"' \
  | while read -r name version; do
    # Skip workspace crates
    if grep -qx "$name" /tmp/workspace_crates.txt 2>/dev/null; then
      continue
    fi

    # Query crates.io for publish date
    response=$(curl -sf "https://crates.io/api/v1/crates/${name}/${version}" 2>/dev/null || true)
    if [[ -z "$response" ]]; then
      continue
    fi

    created=$(echo "$response" | jq -r '.version.created_at // empty' 2>/dev/null || true)
    if [[ -z "$created" ]]; then
      continue
    fi

    # Parse date to epoch
    publish_epoch=$(date -d "$created" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%S" "${created%%.*}" +%s 2>/dev/null || true)
    if [[ -z "$publish_epoch" ]]; then
      continue
    fi

    age_secs=$((NOW - publish_epoch))
    age_days=$((age_secs / 86400))

    if [[ $age_secs -lt $MIN_AGE_SECS ]]; then
      echo "FAIL: ${name}@${version} published ${age_days} days ago (min: ${MIN_DAYS})"
      FAILED=1
    fi
  done

rm -f /tmp/workspace_crates.txt

if [[ $FAILED -eq 1 ]]; then
  echo ""
  echo "Some dependencies are too new. Either:"
  echo "  1. Wait for them to age past ${MIN_DAYS} days"
  echo "  2. Pin to an older version"
  echo "  3. Override with: ./scripts/check-dep-age.sh --min-days 0"
  exit 1
fi

echo "All dependencies are at least ${MIN_DAYS} days old."
