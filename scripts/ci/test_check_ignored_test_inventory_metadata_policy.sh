#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_ignored_test_inventory_drift.sh"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_baseline.json"
METADATA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_metadata.json"
PROMOTION_CRITERIA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_promotion_criteria.json"
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

if [ ! -f "$PROMOTION_CRITERIA_FILE" ]; then
  echo "expected ignored-test promotion criteria fixture to exist" >&2
  exit 1
fi

PASS_REPORT="$TMP_DIR/ignored-test-metadata-pass.json"
bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$METADATA_FILE" \
  --promotion-criteria-file "$PROMOTION_CRITERIA_FILE" \
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
  --promotion-criteria-file "$PROMOTION_CRITERIA_FILE" \
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
mutated = False
for entry in payload.get("ignored_tests", []):
    if entry.get("priority") in {"P0", "P1"}:
        entry["tracking_issue"] = ""
        mutated = True
        break
if not mutated and payload.get("ignored_tests"):
    payload["ignored_tests"][0]["priority"] = "P1"
    payload["ignored_tests"][0]["tracking_issue"] = ""
    mutated = True
if not mutated:
    raise SystemExit("expected at least one ignored-test metadata entry")
metadata_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$MUTATED_PRIORITY_METADATA" \
  --promotion-criteria-file "$PROMOTION_CRITERIA_FILE" \
  --output-json "$TMP_DIR/check-priority-fail.json" >"$TMP_DIR/check-priority-fail.out" 2>&1; then
  echo "expected ignored-test checker to fail when high-priority tracking linkage is missing" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/check-priority-fail.out"
grep -q 'high_priority_tracking_issue_missing' "$TMP_DIR/check-priority-fail.out"

MUTATED_CRITERIA="$TMP_DIR/mutated-ignored-test-promotion-criteria.json"
cp "$PROMOTION_CRITERIA_FILE" "$MUTATED_CRITERIA"
python3 - "$MUTATED_CRITERIA" "$METADATA_FILE" <<'PY'
import json
import sys
from pathlib import Path

criteria_path = Path(sys.argv[1])
metadata_path = Path(sys.argv[2])
payload = json.loads(criteria_path.read_text(encoding="utf-8"))
metadata_payload = json.loads(metadata_path.read_text(encoding="utf-8"))
categories = payload.get("categories", [])
used_reasons = {
    entry.get("reason")
    for entry in metadata_payload.get("ignored_tests", [])
    if isinstance(entry, dict) and isinstance(entry.get("reason"), str)
}
if categories:
    filtered = [
        category
        for category in categories
        if category.get("category") not in used_reasons
    ]
    if len(filtered) == len(categories):
        filtered = categories[:-1]
    payload["categories"] = filtered
criteria_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$METADATA_FILE" \
  --promotion-criteria-file "$MUTATED_CRITERIA" \
  --output-json "$TMP_DIR/check-promotion-fail.json" >"$TMP_DIR/check-promotion-fail.out" 2>&1; then
  echo "expected ignored-test checker to fail when promotion criteria category coverage drifts" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/check-promotion-fail.out"
grep -q 'ignored_test_promotion_criteria_missing' "$TMP_DIR/check-promotion-fail.out"

MUTATED_STALE_RATIONALE_METADATA="$TMP_DIR/mutated-stale-rationale-metadata.json"
cp "$METADATA_FILE" "$MUTATED_STALE_RATIONALE_METADATA"
python3 - "$MUTATED_STALE_RATIONALE_METADATA" <<'PY'
import json
import sys
from pathlib import Path

metadata_path = Path(sys.argv[1])
payload = json.loads(metadata_path.read_text(encoding="utf-8"))
if not payload.get("ignored_tests"):
    raise SystemExit("expected ignored-test metadata entries")
payload["ignored_tests"][0]["reason"] = "stale-rationale-legacy"
metadata_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$MUTATED_STALE_RATIONALE_METADATA" \
  --promotion-criteria-file "$PROMOTION_CRITERIA_FILE" \
  --output-json "$TMP_DIR/check-stale-rationale-fail.json" >"$TMP_DIR/check-stale-rationale-fail.out" 2>&1; then
  echo "expected ignored-test checker to fail when metadata rationale drifts from promotion-criteria categories" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/check-stale-rationale-fail.out"
grep -q 'ignored_test_rationale_stale' "$TMP_DIR/check-stale-rationale-fail.out"

echo "Ignored-test metadata policy tests passed."
