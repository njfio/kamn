#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_ignored_test_inventory_drift.sh"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_baseline.json"
METADATA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_metadata.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECK_SCRIPT" ]; then
  echo "expected ignored-test drift checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FILE" ]; then
  echo "expected ignored-test baseline fixture to exist" >&2
  exit 1
fi

if [ ! -f "$METADATA_FILE" ]; then
  echo "expected ignored-test metadata fixture to exist" >&2
  exit 1
fi

PASS_REPORT="$TMP_DIR/ignored-test-metadata-pass.json"
bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$METADATA_FILE" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/check-pass.out"

grep -q '^status=pass$' "$TMP_DIR/check-pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/check-pass.out"

MUTATED_METADATA="$TMP_DIR/mutated-ignored-test-metadata.json"
cp "$METADATA_FILE" "$MUTATED_METADATA"
python3 - "$MUTATED_METADATA" <<'PY'
import json
import sys
from pathlib import Path

metadata_path = Path(sys.argv[1])
payload = json.loads(metadata_path.read_text(encoding="utf-8"))
if payload.get("ignored_tests"):
    payload["ignored_tests"] = payload["ignored_tests"][:-1]
metadata_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$MUTATED_METADATA" \
  --output-json "$TMP_DIR/check-metadata-fail.json" >"$TMP_DIR/check-metadata-fail.out" 2>&1; then
  echo "expected ignored-test checker to fail when metadata coverage drifts" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/check-metadata-fail.out"
grep -q 'ignored_test_metadata_missing' "$TMP_DIR/check-metadata-fail.out"

MUTATED_PRIORITY_METADATA="$TMP_DIR/mutated-priority-metadata.json"
cp "$METADATA_FILE" "$MUTATED_PRIORITY_METADATA"
python3 - "$MUTATED_PRIORITY_METADATA" <<'PY'
import json
import sys
from pathlib import Path

metadata_path = Path(sys.argv[1])
payload = json.loads(metadata_path.read_text(encoding="utf-8"))
for entry in payload.get("ignored_tests", []):
    if entry.get("priority") == "P1":
        entry["tracking_issue"] = ""
        break
metadata_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$MUTATED_PRIORITY_METADATA" \
  --output-json "$TMP_DIR/check-priority-fail.json" >"$TMP_DIR/check-priority-fail.out" 2>&1; then
  echo "expected ignored-test checker to fail when high-priority tracking linkage is missing" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/check-priority-fail.out"
grep -q 'high_priority_tracking_issue_missing' "$TMP_DIR/check-priority-fail.out"

echo "Ignored-test metadata policy tests passed."
