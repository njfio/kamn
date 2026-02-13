#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_ignored_test_inventory_drift.sh"
GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_ignored_test_inventory_baseline.sh"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_baseline.json"
METADATA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_metadata.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECK_SCRIPT" ]; then
  echo "expected ignored-test drift checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$GENERATE_SCRIPT" ]; then
  echo "expected ignored-test inventory generator wrapper to be executable" >&2
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

GENERATED_BASELINE="$TMP_DIR/generated-ignored-test-baseline.json"
bash "$GENERATE_SCRIPT" --output-json "$GENERATED_BASELINE" >"$TMP_DIR/generate.out"

grep -q '^status=generated$' "$TMP_DIR/generate.out"
grep -q '^ignored_test_count=' "$TMP_DIR/generate.out"

python3 - "$BASELINE_FILE" "$GENERATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
generated = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
if baseline != generated:
    raise SystemExit("ignored-test generated baseline drift detected")
PY

PASS_REPORT="$TMP_DIR/ignored-test-drift-pass.json"
bash "$CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$METADATA_FILE" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/check-pass.out"

grep -q '^status=pass$' "$TMP_DIR/check-pass.out"
grep -q '^violation_count=0$' "$TMP_DIR/check-pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/check-pass.out"

MUTATED_BASELINE="$TMP_DIR/mutated-ignored-test-baseline.json"
cp "$BASELINE_FILE" "$MUTATED_BASELINE"
python3 - "$MUTATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
if payload.get("ignored_tests"):
    payload["ignored_tests"] = payload["ignored_tests"][:-1]
payload["ignored_test_count"] = len(payload["ignored_tests"])
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --baseline-file "$MUTATED_BASELINE" \
  --metadata-file "$METADATA_FILE" \
  --output-json "$TMP_DIR/check-fail.json" >"$TMP_DIR/check-fail.out" 2>&1; then
  echo "expected ignored-test drift checker to fail on baseline drift" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/check-fail.out"
grep -q 'unexpected_ignored_tests_present' "$TMP_DIR/check-fail.out"

echo "Ignored-test inventory drift checker tests passed."
