#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
source "$KAMN_ROOT/scripts/lib/test_harness.sh"

GENERATE_SCRIPT="$KAMN_ROOT/scripts/ci/generate_kolme_wrapper_inventory_baseline.sh"
CHECK_SCRIPT="$KAMN_ROOT/scripts/ci/check_kolme_wrapper_inventory_baseline.sh"
PYTHON_SCRIPT="$KAMN_ROOT/scripts/ci/kolme_wrapper_inventory_baseline.py"

usage() {
  cat >&2 <<'USAGE'
Usage: test_wave_wrapper_family_baseline_contract_impl.sh --family <kolme|non_kolme> --wave-id <id>
USAGE
}

FAMILY=""
WAVE_ID=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --family)
      if [ "$#" -lt 2 ]; then
        usage
        exit 1
      fi
      FAMILY="$2"
      shift 2
      ;;
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

if [ -z "$FAMILY" ] || [ -z "$WAVE_ID" ]; then
  usage
  exit 1
fi

if [ "$FAMILY" != "kolme" ] && [ "$FAMILY" != "non_kolme" ]; then
  echo "family must be one of: kolme, non_kolme" >&2
  exit 1
fi

if ! [[ "$WAVE_ID" =~ ^[0-9]+$ ]]; then
  echo "wave id must be numeric: $WAVE_ID" >&2
  exit 1
fi

if [ "$FAMILY" = "kolme" ]; then
  WAVE_LABEL="Kolme wave-${WAVE_ID}"
  MATRIX_FIXTURE="$KAMN_ROOT/fixtures/ci/kolme_wave${WAVE_ID}_wrapper_family_matrix.json"
  BASELINE_FIXTURE="$KAMN_ROOT/fixtures/ci/kolme_wave${WAVE_ID}_wrapper_family_baseline.json"
  MISSING_WRAPPER="scripts/ci/run_missing_kolme_wave${WAVE_ID}_wrapper.sh"
else
  WAVE_LABEL="non-Kolme wave-${WAVE_ID}"
  MATRIX_FIXTURE="$KAMN_ROOT/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_matrix.json"
  BASELINE_FIXTURE="$KAMN_ROOT/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_baseline.json"
  MISSING_WRAPPER="scripts/ci/run_missing_non_kolme_wave${WAVE_ID}_wrapper.sh"
fi

test_harness_setup
TMP_DIR="$test_harness_tmp_dir"

if ! test_harness_require_executable "$GENERATE_SCRIPT" "expected baseline generator wrapper to be executable"; then
  exit 1
fi
if ! test_harness_require_executable "$CHECK_SCRIPT" "expected baseline checker wrapper to be executable"; then
  exit 1
fi
if ! test_harness_require_executable "$PYTHON_SCRIPT" "expected baseline policy python script to be executable"; then
  exit 1
fi
if ! test_harness_require_file "$MATRIX_FIXTURE" "expected ${WAVE_LABEL} wrapper-family matrix fixture to exist"; then
  exit 1
fi
if ! test_harness_require_file "$BASELINE_FIXTURE" "expected ${WAVE_LABEL} wrapper-family baseline fixture to exist"; then
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

if ! test_harness_assert_file_contains "$TMP_DIR/generate.out" '^status=generated$' "expected generated baseline status marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/generate.out" "^wrapper_count=${expected_wrapper_count}$" "expected wrapper count marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/generate.out" "^total_shell_loc=${expected_total_shell_loc}$" "expected total shell loc marker"; then
  exit 1
fi

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

if ! test_harness_assert_file_contains "$TMP_DIR/check-pass.out" '^status=pass$' "expected pass status marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/check-pass.out" '^wrapper_count_delta=0$' "expected wrapper_count_delta marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/check-pass.out" '^total_shell_loc_delta=0$' "expected total_shell_loc_delta marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/check-pass.out" '^violation_count=0$' "expected violation_count marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/check-pass.out" '^reason_codes=none$' "expected reason_codes marker"; then
  exit 1
fi

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
if ! test_harness_assert_file_contains "$TMP_DIR/check-mutated-baseline.out" '^status=fail$' "expected fail status for mutated baseline"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/check-mutated-baseline.out" '^reason_codes=lane_shell_loc_drift$' "expected lane_shell_loc_drift reason code"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/check-mutated-baseline.out" 'shell_loc drifted' "expected shell_loc drift detail"; then
  exit 1
fi

MUTATED_MATRIX="$TMP_DIR/mutated-matrix.json"
cp "$MATRIX_FIXTURE" "$MUTATED_MATRIX"
python3 - "$MUTATED_MATRIX" "$MISSING_WRAPPER" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
missing_wrapper = sys.argv[2]
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lanes"][0]["source_entry"] = missing_wrapper
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --matrix-file "$MUTATED_MATRIX" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/check-mutated-matrix.out" 2>&1; then
  echo "expected ${WAVE_LABEL} baseline checker to fail when matrix source_entry wrapper is missing" >&2
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/check-mutated-matrix.out" 'lane wrapper path does not exist' "expected missing lane wrapper marker"; then
  exit 1
fi

echo "${WAVE_LABEL} wrapper-family baseline contract tests passed."
