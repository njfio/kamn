#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/reputation/run_weighted_decay_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/reputation_decay/compact_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

report_json="$TMP_DIR/reputation-weighted-decay-contract-report.json"

cargo test -p kamn-core --test trust_score_engine >/dev/null

matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$FIXTURE_FILE" \
    --output-json "$report_json"
)"

if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected weighted decay compact matrix to pass" >&2
  exit 1
fi

echo "weighted decay contract lane tests passed."
