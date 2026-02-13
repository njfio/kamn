#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_wrapper_inventory_baseline.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_wrapper_inventory_baseline.sh"
PYTHON_SCRIPT="$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py"

usage() {
  cat >&2 <<'USAGE'
Usage: test_non_kolme_wave_wrapper_family_baseline_contract_impl.sh --wave-id <id>
USAGE
}

WAVE_ID=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --wave-id)
      if [ "$#" -lt 2 ]; then
        usage
        exit 1
      fi
      WAVE_ID="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [ -z "$WAVE_ID" ]; then
  usage
  exit 1
fi

if ! [[ "$WAVE_ID" =~ ^[0-9]+$ ]]; then
  echo "wave id must be numeric: $WAVE_ID" >&2
  exit 1
fi

WAVE_LABEL="non-Kolme wave-${WAVE_ID}"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_matrix.json"
BASELINE_FIXTURE="$ROOT_DIR/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_baseline.json"

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
  echo "expected ${WAVE_LABEL} wrapper-family matrix fixture to exist" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FIXTURE" ]; then
  echo "expected ${WAVE_LABEL} wrapper-family baseline fixture to exist" >&2
  exit 1
fi

EXPECTED_METRICS_ENV="$TMP_DIR/expected-metrics.env"
python3 - "$BASELINE_FIXTURE" <<'PY' >"$EXPECTED_METRICS_ENV"
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(f"expected_wrapper_count={int(payload['wrapper_count'])}")
print(f"expected_symlink_wrapper_count={int(payload['symlink_wrapper_count'])}")
print(f"expected_regular_file_wrapper_count={int(payload['regular_file_wrapper_count'])}")
print(f"expected_total_shell_loc={int(payload['total_shell_loc'])}")
PY

# shellcheck disable=SC1090
source "$EXPECTED_METRICS_ENV"

GENERATED_BASELINE="$TMP_DIR/generated-baseline.json"
bash "$GENERATE_SCRIPT" \
  --matrix-file "$MATRIX_FIXTURE" \
  --output-json "$GENERATED_BASELINE" >"$TMP_DIR/generate.out"

grep -q '^status=generated$' "$TMP_DIR/generate.out"
grep -q "^wrapper_count=${expected_wrapper_count}$" "$TMP_DIR/generate.out"
grep -q "^total_shell_loc=${expected_total_shell_loc}$" "$TMP_DIR/generate.out"

python3 - "$GENERATED_BASELINE" "$EXPECTED_METRICS_ENV" "$WAVE_LABEL" <<'PY'
import json
import sys
from pathlib import Path

generated = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
expected = {}
for line in Path(sys.argv[2]).read_text(encoding="utf-8").splitlines():
    key, value = line.split("=", 1)
    expected[key] = int(value)
wave_label = sys.argv[3]

if int(generated["wrapper_count"]) != expected["expected_wrapper_count"]:
    raise SystemExit(f"expected {wave_label} wrapper_count to be {expected['expected_wrapper_count']}")
if int(generated["symlink_wrapper_count"]) != expected["expected_symlink_wrapper_count"]:
    raise SystemExit(
        f"expected {wave_label} symlink_wrapper_count to be {expected['expected_symlink_wrapper_count']}"
    )
if int(generated["regular_file_wrapper_count"]) != expected["expected_regular_file_wrapper_count"]:
    raise SystemExit(
        f"expected {wave_label} regular_file_wrapper_count to be {expected['expected_regular_file_wrapper_count']}"
    )
if int(generated["total_shell_loc"]) != expected["expected_total_shell_loc"]:
    raise SystemExit(
        f"expected {wave_label} total_shell_loc to be {expected['expected_total_shell_loc']}"
    )
PY

python3 - "$BASELINE_FIXTURE" "$GENERATED_BASELINE" "$WAVE_LABEL" <<'PY'
import json
import sys
from pathlib import Path

baseline = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
generated = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
wave_label = sys.argv[3]
if baseline != generated:
    raise SystemExit(f"{wave_label} generated baseline fixture drift detected")
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
  echo "expected ${WAVE_LABEL} baseline checker to fail on shell_loc drift" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/check-mutated-baseline.out"
grep -q '^reason_codes=lane_shell_loc_drift$' "$TMP_DIR/check-mutated-baseline.out"
grep -q 'shell_loc drifted' "$TMP_DIR/check-mutated-baseline.out"

MUTATED_MATRIX="$TMP_DIR/mutated-matrix.json"
cp "$MATRIX_FIXTURE" "$MUTATED_MATRIX"
python3 - "$MUTATED_MATRIX" "$WAVE_ID" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
wave_id = sys.argv[2]
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lanes"][0]["source_entry"] = f"scripts/ci/run_missing_non_kolme_wave{wave_id}_wrapper.sh"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --matrix-file "$MUTATED_MATRIX" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/check-mutated-matrix.out" 2>&1; then
  echo "expected ${WAVE_LABEL} baseline checker to fail when matrix source_entry wrapper is missing" >&2
  exit 1
fi
grep -q 'lane wrapper path does not exist' "$TMP_DIR/check-mutated-matrix.out"

echo "${WAVE_LABEL} wrapper-family baseline contract tests passed."
