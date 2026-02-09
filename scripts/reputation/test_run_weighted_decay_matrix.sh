#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_weighted_decay_matrix.py"
COMPACT_FIXTURE="$ROOT_DIR/fixtures/reputation_decay/compact_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

compact_report="$TMP_DIR/reputation-weighted-decay-compact-report.json"
compact_output="$(
  python3 "$SCRIPT" \
    --fixture "$COMPACT_FIXTURE" \
    --output-json "$compact_report"
)"

if ! printf '%s\n' "$compact_output" | grep -q "^status=pass;"; then
  echo "expected pass status from weighted decay compact matrix" >&2
  exit 1
fi

python3 - "$compact_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.reputation.weighted-decay.matrix.v1"
assert report["status"] == "pass"
assert report["case_count"] == 3
assert report["failed_count"] == 0
PY

subset_output="$(
  python3 "$SCRIPT" \
    --fixture "$COMPACT_FIXTURE" \
    --max-cases 1
)"

if ! printf '%s\n' "$subset_output" | grep -q "^status=pass; cases=1; failed=0$"; then
  echo "expected bounded subset matrix run to pass" >&2
  exit 1
fi

echo "weighted decay matrix tests passed."
