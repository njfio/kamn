#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/sync_flaky_registry_issues.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

future="$(date -u -d '+10 days' +%Y-%m-%d)"
cat > "$TMP_DIR/registry.txt" <<EOF2
owner1|crate::test_a|#70|$future|temporary quarantine
owner2|crate::test_b|#70|$future|intermittent timeout
owner3|crate::test_c|#69|$future|tracking bug
EOF2

out="$TMP_DIR/out.txt"
"$SCRIPT" --repo njfio/kamn --registry "$TMP_DIR/registry.txt" --dry-run > "$out"

grep -q '\[dry-run\] issue #70' "$out"
grep -q '\[dry-run\] issue #69' "$out"
grep -q 'owner1 | crate::test_a' "$out"
grep -q 'owner2 | crate::test_b' "$out"
grep -q 'owner3 | crate::test_c' "$out"
grep -q 'Flaky registry sync complete for 2 issue(s).' "$out"

echo "sync_flaky_registry_issues tests passed."
