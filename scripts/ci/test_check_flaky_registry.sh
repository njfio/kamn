#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_flaky_registry.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

today="$(date -u +%Y-%m-%d)"
future="$(date -u -d '+7 days' +%Y-%m-%d)"
past="$(date -u -d '-2 days' +%Y-%m-%d)"

# Valid registry
cat > "$TMP_DIR/valid.txt" <<EOF2
# header
owner1|crate::test_a|#70|$future|temporary quarantine
EOF2

"$SCRIPT" "$TMP_DIR/valid.txt" >/dev/null

# Invalid issue format
cat > "$TMP_DIR/bad_issue.txt" <<EOF2
owner1|crate::test_a|issue-70|$future|temporary quarantine
EOF2
if "$SCRIPT" "$TMP_DIR/bad_issue.txt" >/dev/null 2>&1; then
  echo "Expected invalid issue format failure" >&2
  exit 1
fi

# Expired entry
cat > "$TMP_DIR/expired.txt" <<EOF2
owner1|crate::test_a|#70|$past|temporary quarantine
EOF2
if "$SCRIPT" "$TMP_DIR/expired.txt" >/dev/null 2>&1; then
  echo "Expected expired entry failure" >&2
  exit 1
fi

# Missing field
cat > "$TMP_DIR/missing.txt" <<EOF2
owner1|crate::test_a|#70|$future|
EOF2
if "$SCRIPT" "$TMP_DIR/missing.txt" >/dev/null 2>&1; then
  echo "Expected missing field failure" >&2
  exit 1
fi

echo "check_flaky_registry tests passed."
