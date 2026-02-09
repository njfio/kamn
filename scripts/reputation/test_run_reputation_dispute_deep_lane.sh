#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_reputation_dispute_deep_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

report_json="$TMP_DIR/reputation-dispute-deep-report.json"
output="$(bash "$SCRIPT" --output-json "$report_json")"

if ! printf '%s\n' "$output" | grep -q "reputation dispute deep lane tests passed."; then
  echo "expected deep lane success output for reputation dispute lane" >&2
  exit 1
fi

python3 - "$report_json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["status"] == "pass"
assert report["failed_count"] == 0
assert report["case_count"] >= 1
PY

echo "reputation dispute deep lane script tests passed."
