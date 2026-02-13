#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_wrapper_inventory_baseline.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_wrapper_inventory_baseline.sh"
PYTHON_SCRIPT="$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/ci/non_kolme_wave17_wrapper_family_matrix.json"
BASELINE_FIXTURE="$ROOT_DIR/fixtures/ci/non_kolme_wave17_wrapper_family_baseline.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATE_SCRIPT" ]; then
  echo "expected baseline generator wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECK_SCRIPT" ]; then
  echo "expected baseline checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$PYTHON_SCRIPT" ]; then
  echo "expected baseline policy python script to be executable" >&2
  exit 1
fi

if [ ! -f "$MATRIX_FIXTURE" ]; then
  echo "expected non-Kolme wave-17 wrapper-family matrix fixture to exist" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FIXTURE" ]; then
  echo "expected non-Kolme wave-17 wrapper-family baseline fixture to exist" >&2
  exit 1
fi

GENERATED_BASELINE="$TMP_DIR/generated-baseline.json"
bash "$GENERATE_SCRIPT" \
  --matrix-file "$MATRIX_FIXTURE" \
  --output-json "$GENERATED_BASELINE" >"$TMP_DIR/generate.out"

grep -q '^status=generated$' "$TMP_DIR/generate.out"
grep -q '^wrapper_count=3$' "$TMP_DIR/generate.out"
grep -q '^total_shell_loc=3$' "$TMP_DIR/generate.out"

python3 - "$GENERATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload["symlink_wrapper_count"] != 3:
    raise SystemExit("expected non-Kolme wave-17 symlink_wrapper_count to be 3")
if payload["regular_file_wrapper_count"] != 0:
    raise SystemExit("expected non-Kolme wave-17 regular_file_wrapper_count to be 0")
if payload["total_shell_loc"] != 3:
    raise SystemExit("expected non-Kolme wave-17 total_shell_loc to be 3")
PY

python3 - "$BASELINE_FIXTURE" "$GENERATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
generated = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
if baseline != generated:
    raise SystemExit("non-Kolme wave-17 generated baseline fixture drift detected")
PY

DELTA_REPORT="$TMP_DIR/delta-report.json"
bash "$CHECK_SCRIPT" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$BASELINE_FIXTURE" \
  --output-json "$DELTA_REPORT" >"$TMP_DIR/check-pass.out"

grep -q '^status=pass$' "$TMP_DIR/check-pass.out"
grep -q '^wrapper_count_delta=0$' "$TMP_DIR/check-pass.out"
grep -q '^total_shell_loc_delta=0$' "$TMP_DIR/check-pass.out"
grep -q '^violation_count=0$' "$TMP_DIR/check-pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/check-pass.out"

MUTATED_BASELINE="$TMP_DIR/mutated-baseline.json"
cp "$BASELINE_FIXTURE" "$MUTATED_BASELINE"
python3 - "$MUTATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lanes"][0]["shell_loc"] = payload["lanes"][0]["shell_loc"] + 1
payload["total_shell_loc"] = payload["total_shell_loc"] + 1
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_BASELINE" >"$TMP_DIR/check-mutated-baseline.out" 2>&1; then
  echo "expected non-Kolme wave-17 baseline checker to fail on shell_loc drift" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/check-mutated-baseline.out"
grep -q '^reason_codes=lane_shell_loc_drift$' "$TMP_DIR/check-mutated-baseline.out"
grep -q 'shell_loc drifted' "$TMP_DIR/check-mutated-baseline.out"

MUTATED_MATRIX="$TMP_DIR/mutated-matrix.json"
cp "$MATRIX_FIXTURE" "$MUTATED_MATRIX"
python3 - "$MUTATED_MATRIX" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lanes"][0]["source_entry"] = "scripts/signer/run_missing_wave17_wrapper.sh"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --matrix-file "$MUTATED_MATRIX" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/check-mutated-matrix.out" 2>&1; then
  echo "expected non-Kolme wave-17 baseline checker to fail when matrix source_entry wrapper is missing" >&2
  exit 1
fi
grep -q 'lane wrapper path does not exist' "$TMP_DIR/check-mutated-matrix.out"

echo "non-Kolme wave-17 wrapper-family baseline contract tests passed."
