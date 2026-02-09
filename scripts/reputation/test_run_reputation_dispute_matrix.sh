#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_reputation_dispute_matrix.py"
FIXTURE="$ROOT_DIR/fixtures/reputation_dispute/replay_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

report_json="$TMP_DIR/reputation-dispute-report.json"
output="$(
  python3 "$SCRIPT" \
    --fixture "$FIXTURE" \
    --output-json "$report_json"
)"

if ! printf '%s\n' "$output" | grep -q "^status=pass;"; then
  echo "expected pass status from reputation dispute matrix" >&2
  exit 1
fi

python3 - "$report_json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.reputation.dispute.replay-matrix.v1"
assert report["status"] == "pass"
assert report["case_count"] == 3
assert report["failed_count"] == 0
PY

subset_output="$(
  python3 "$SCRIPT" \
    --fixture "$FIXTURE" \
    --max-cases 1
)"

if ! printf '%s\n' "$subset_output" | grep -q "^status=pass; cases=1; failed=0$"; then
  echo "expected bounded subset matrix run to pass" >&2
  exit 1
fi

echo "reputation dispute matrix tests passed."
