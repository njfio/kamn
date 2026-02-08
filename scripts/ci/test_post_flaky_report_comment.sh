#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/post_flaky_report_comment.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

future="$(date -u -d '+7 days' +%Y-%m-%d)"
cat > "$TMP_DIR/registry.txt" <<EOF2
owner1|crate::test_a|#70|$future|temporary quarantine
EOF2

out="$TMP_DIR/out.txt"
"$SCRIPT" --repo njfio/kamn --issue 70 --registry "$TMP_DIR/registry.txt" --dry-run > "$out"

grep -q '^Automated flaky registry report' "$out"
grep -q '^# Flaky Registry Report' "$out"
grep -q '| owner1 | crate::test_a | #70 |' "$out"

echo "post_flaky_report_comment tests passed."
