#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_wrapper_inventory_baseline.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_wrapper_inventory_baseline.sh"
PYTHON_SCRIPT="$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/lane_migration_matrix.json"
BASELINE_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/wrapper_inventory_baseline.json"
WAVE10_MATRIX_FIXTURE="$ROOT_DIR/fixtures/ci/kolme_wave10_wrapper_family_matrix.json"
WAVE10_BASELINE_FIXTURE="$ROOT_DIR/fixtures/ci/kolme_wave10_wrapper_family_baseline.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$GENERATE_SCRIPT" "expected baseline generator wrapper to be executable"

test_harness_require_executable "$CHECK_SCRIPT" "expected baseline checker wrapper to be executable"

test_harness_require_executable "$PYTHON_SCRIPT" "expected baseline policy python script to be executable"

test_harness_require_file "$MATRIX_FIXTURE" "expected lane migration matrix fixture to exist"

test_harness_require_file "$BASELINE_FIXTURE" "expected wrapper inventory baseline fixture to exist"

test_harness_require_file "$WAVE10_MATRIX_FIXTURE" "expected wave-10 wrapper-family matrix fixture to exist"

test_harness_require_file "$WAVE10_BASELINE_FIXTURE" "expected wave-10 wrapper-family baseline fixture to exist"

GENERATED_BASELINE="$TMP_DIR/generated-baseline.json"
bash "$GENERATE_SCRIPT" \
  --matrix-file "$MATRIX_FIXTURE" \
  --output-json "$GENERATED_BASELINE" >"$TMP_DIR/generate.out"

grep -q '^status=generated$' "$TMP_DIR/generate.out"
grep -q '^wrapper_count=11$' "$TMP_DIR/generate.out"
grep -q '^total_shell_loc=11$' "$TMP_DIR/generate.out"

python3 - "$GENERATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload["symlink_wrapper_count"] != 11:
    raise SystemExit("expected symlink_wrapper_count to be 11")
if payload["regular_file_wrapper_count"] != 0:
    raise SystemExit("expected regular_file_wrapper_count to be 0")
if payload["total_shell_loc"] != 11:
    raise SystemExit("expected total_shell_loc to be 11")
PY

python3 - "$BASELINE_FIXTURE" "$GENERATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
generated = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
if baseline != generated:
    raise SystemExit("generated baseline fixture drift detected")
PY

WAVE10_GENERATED_BASELINE="$TMP_DIR/generated-wave10-baseline.json"
bash "$GENERATE_SCRIPT" \
  --matrix-file "$WAVE10_MATRIX_FIXTURE" \
  --output-json "$WAVE10_GENERATED_BASELINE" >"$TMP_DIR/generate-wave10.out"

grep -q '^status=generated$' "$TMP_DIR/generate-wave10.out"
grep -q '^wrapper_count=2$' "$TMP_DIR/generate-wave10.out"
grep -q '^total_shell_loc=2$' "$TMP_DIR/generate-wave10.out"

python3 - "$WAVE10_GENERATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload["symlink_wrapper_count"] != 2:
    raise SystemExit("expected wave-10 symlink_wrapper_count to be 2")
if payload["regular_file_wrapper_count"] != 0:
    raise SystemExit("expected wave-10 regular_file_wrapper_count to be 0")
if payload["total_shell_loc"] != 2:
    raise SystemExit("expected wave-10 total_shell_loc to be 2")
PY

python3 - "$WAVE10_BASELINE_FIXTURE" "$WAVE10_GENERATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
generated = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
if baseline != generated:
    raise SystemExit("wave-10 generated baseline fixture drift detected")
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
  echo "expected baseline checker to fail on shell_loc drift" >&2
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
payload["lanes"][0]["source_entry"] = "scripts/kolme/run_missing_wrapper_contract_lane.sh"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --matrix-file "$MUTATED_MATRIX" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/check-mutated-matrix.out" 2>&1; then
  echo "expected baseline checker to fail when matrix source_entry wrapper is missing" >&2
  exit 1
fi
grep -q 'lane wrapper path does not exist' "$TMP_DIR/check-mutated-matrix.out"

echo "Kolme wrapper inventory baseline contract tests passed."
