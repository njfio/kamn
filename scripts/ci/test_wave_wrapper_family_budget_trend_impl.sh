#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
source "$KAMN_ROOT/scripts/lib/test_harness.sh"

PYTHON_CHECKER="$KAMN_ROOT/scripts/ci/kolme_wrapper_inventory_baseline.py"

assert_dispatch_registry_entry() {
  local wrapper_rel="$1"
  local expected_target="$2"
  local expected_wave_id="$3"

  python3 - "$KAMN_ROOT/scripts/lib/exec_registry.json" "$wrapper_rel" "$expected_target" "$expected_wave_id" <<'PY'
import json
import sys
from pathlib import Path

registry_path = Path(sys.argv[1])
wrapper_rel = sys.argv[2]
expected_target = sys.argv[3]
expected_wave_id = sys.argv[4]

payload = json.loads(registry_path.read_text(encoding="utf-8"))
entries = payload.get("entries")
if not isinstance(entries, dict):
    raise SystemExit("expected exec registry entries map")

entry = entries.get(wrapper_rel)
if not isinstance(entry, dict):
    raise SystemExit(f"missing exec registry entry for {wrapper_rel}")

if entry.get("interpreter") != "bash":
    raise SystemExit(f"expected interpreter=bash for {wrapper_rel}")
if entry.get("target") != expected_target:
    raise SystemExit(
        f"expected target={expected_target} for {wrapper_rel}; found {entry.get('target')!r}"
    )
if entry.get("passthrough") is not False:
    raise SystemExit(f"expected passthrough=false for {wrapper_rel}")

expected_args = ["--wave-id", expected_wave_id]
if entry.get("args_prefix") != expected_args:
    raise SystemExit(
        f"expected args_prefix={expected_args!r} for {wrapper_rel}; "
        f"found {entry.get('args_prefix')!r}"
    )
PY
}

usage() {
  cat >&2 <<'USAGE'
Usage: test_wave_wrapper_family_budget_trend_impl.sh --family <kolme|non_kolme> --wave-id <id>
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
  TREND_CHECKER="$KAMN_ROOT/scripts/ci/check_kolme_wave${WAVE_ID}_wrapper_family_budget_trend.sh"
  THRESHOLD_FILE="$KAMN_ROOT/fixtures/ci/kolme_wave${WAVE_ID}_wrapper_family_trend_thresholds.json"
  BASELINE_FIXTURE="$KAMN_ROOT/fixtures/ci/kolme_wave${WAVE_ID}_wrapper_family_baseline.json"
  MATRIX_FIXTURE="$KAMN_ROOT/fixtures/ci/kolme_wave${WAVE_ID}_wrapper_family_matrix.json"
  DISPATCH_WRAPPER_REL="scripts/ci/test_check_kolme_wave${WAVE_ID}_wrapper_family_budget_trend.sh"
  DISPATCH_TARGET="scripts/ci/test_check_kolme_wave_wrapper_family_budget_trend_impl.sh"
else
  WAVE_LABEL="non-Kolme wave-${WAVE_ID}"
  TREND_CHECKER="$KAMN_ROOT/scripts/ci/check_non_kolme_wave${WAVE_ID}_wrapper_family_budget_trend.sh"
  THRESHOLD_FILE="$KAMN_ROOT/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_trend_thresholds.json"
  BASELINE_FIXTURE="$KAMN_ROOT/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_baseline.json"
  MATRIX_FIXTURE="$KAMN_ROOT/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_matrix.json"
  DISPATCH_WRAPPER_REL="scripts/ci/test_check_non_kolme_wave${WAVE_ID}_wrapper_family_budget_trend.sh"
  DISPATCH_TARGET="scripts/ci/test_check_non_kolme_wave_wrapper_family_budget_trend_impl.sh"
fi

DISPATCH_WRAPPER_PATH="$KAMN_ROOT/$DISPATCH_WRAPPER_REL"
if [ ! -L "$DISPATCH_WRAPPER_PATH" ]; then
  echo "expected wrapper family trend entrypoint to be symlink-backed: $DISPATCH_WRAPPER_REL" >&2
  exit 1
fi

if [ "$(readlink "$DISPATCH_WRAPPER_PATH")" != "../lib/exec_dispatch.sh" ]; then
  echo "expected $DISPATCH_WRAPPER_REL to target ../lib/exec_dispatch.sh" >&2
  exit 1
fi

assert_dispatch_registry_entry "$DISPATCH_WRAPPER_REL" "$DISPATCH_TARGET" "$WAVE_ID"

test_harness_setup
TMP_DIR="$test_harness_tmp_dir"

if ! test_harness_require_executable "$TREND_CHECKER" "expected ${WAVE_LABEL} trend checker wrapper to be executable"; then
  exit 1
fi
if ! test_harness_require_executable "$PYTHON_CHECKER" "expected python baseline checker script to be executable"; then
  exit 1
fi
if ! test_harness_require_file "$THRESHOLD_FILE" "expected ${WAVE_LABEL} trend threshold fixture to exist"; then
  exit 1
fi
if ! test_harness_require_file "$BASELINE_FIXTURE" "expected ${WAVE_LABEL} baseline fixture to exist"; then
  exit 1
fi
if ! test_harness_require_file "$MATRIX_FIXTURE" "expected ${WAVE_LABEL} matrix fixture to exist"; then
  exit 1
fi

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$BASELINE_FIXTURE" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"

if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^status=pass$' "expected pass status marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^mode=trend$' "expected trend mode marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^wrapper_count_delta=0$' "expected wrapper_count_delta marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^total_shell_loc_delta=0$' "expected total_shell_loc_delta marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^violation_count=0$' "expected violation_count marker"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^reason_codes=none$' "expected reason_codes marker"; then
  exit 1
fi

if [ "$FAMILY" = "non_kolme" ]; then
  if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^reason_taxonomy_version=kamn.ci.wrapper-budget-trend-reason-taxonomy.v1$' "expected reason taxonomy marker"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^reason_codes_value=none$' "expected reason_codes_value marker"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^policy_decision=GO$' "expected policy_decision marker"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/pass.out" '^ci_smoke_budget_status=within$' "expected ci_smoke_budget_status marker"; then
    exit 1
  fi
fi

MUTATED_TOTAL_BASELINE="$TMP_DIR/mutated-total-baseline.json"
cp "$BASELINE_FIXTURE" "$MUTATED_TOTAL_BASELINE"
python3 - "$MUTATED_TOTAL_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["total_shell_loc"] = 0
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_TOTAL_BASELINE" >"$TMP_DIR/fail-total.out" 2>&1; then
  echo "expected ${WAVE_LABEL} trend checker to fail when total shell LOC delta exceeds threshold" >&2
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-total.out" '^status=fail$' "expected fail status for total baseline mutation"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-total.out" '^mode=trend$' "expected trend mode marker for total baseline mutation"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-total.out" 'total_shell_loc_delta_threshold_exceeded' "expected total shell loc threshold marker"; then
  exit 1
fi
if [ "$FAMILY" = "non_kolme" ]; then
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-total.out" '^policy_decision=NO-GO$' "expected policy_decision no-go marker"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-total.out" '^reason_taxonomy_version=kamn.ci.wrapper-budget-trend-reason-taxonomy.v1$' "expected reason taxonomy marker for total baseline mutation"; then
    exit 1
  fi
fi

MUTATED_LANE_MATRIX="$TMP_DIR/mutated-lane-matrix.json"
cp "$MATRIX_FIXTURE" "$MUTATED_LANE_MATRIX"
if [ "$FAMILY" = "non_kolme" ]; then
  MUTATED_LANE_WRAPPER="$TMP_DIR/run_mutated_contract_lane.sh"
  cat >"$MUTATED_LANE_WRAPPER" <<'INNER'
#!/usr/bin/env bash
set -euo pipefail

printf 'mutated wrapper lane\n'
INNER

  python3 - "$MUTATED_LANE_MATRIX" "$MUTATED_LANE_WRAPPER" <<'PY'
import json
import sys
from pathlib import Path

matrix_path = Path(sys.argv[1])
wrapper_path = Path(sys.argv[2]).resolve()
payload = json.loads(matrix_path.read_text(encoding="utf-8"))

if not payload.get("lanes"):
    raise SystemExit("expected at least one lane in matrix fixture")

payload["lanes"][0]["source_entry"] = str(wrapper_path)
matrix_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
else
  ALTERNATE_LANE_WRAPPER="$KAMN_ROOT/scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh"
  if ! test_harness_require_executable "$ALTERNATE_LANE_WRAPPER" "expected alternate lane wrapper to exist for source-entry drift test: $ALTERNATE_LANE_WRAPPER"; then
    exit 1
  fi

  python3 - "$MUTATED_LANE_MATRIX" "$ALTERNATE_LANE_WRAPPER" <<'PY'
import json
import sys
from pathlib import Path

matrix_path = Path(sys.argv[1])
wrapper_path = str(Path(sys.argv[2]).resolve())
payload = json.loads(matrix_path.read_text(encoding="utf-8"))

if not payload.get("lanes"):
    raise SystemExit("expected at least one lane in matrix fixture")

payload["lanes"][0]["source_entry"] = wrapper_path
matrix_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
fi

if bash "$TREND_CHECKER" \
  --matrix-file "$MUTATED_LANE_MATRIX" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/fail-lane.out" 2>&1; then
  echo "expected ${WAVE_LABEL} trend checker to fail when lane source_entry drifts from baseline" >&2
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-lane.out" '^status=fail$' "expected fail status for lane-source drift mutation"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-lane.out" '^mode=trend$' "expected trend mode marker for lane-source drift mutation"; then
  exit 1
fi
if [ "$FAMILY" = "non_kolme" ]; then
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-lane.out" 'lane_source_entry_drift' "expected lane_source_entry_drift marker"; then
    exit 1
  fi
else
  if ! grep -Eq 'lane_source_entry_drift|policy_validation_failed' "$TMP_DIR/fail-lane.out"; then
    echo "expected lane_source_entry_drift or policy_validation_failed marker" >&2
    exit 1
  fi
fi

MUTATED_STALE_BASELINE="$TMP_DIR/mutated-stale-baseline.json"
cp "$BASELINE_FIXTURE" "$MUTATED_STALE_BASELINE"
python3 - "$MUTATED_STALE_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))

if not payload.get("lanes"):
    raise SystemExit("expected at least one lane in baseline fixture")

first_lane = payload["lanes"][0]
lane_id = first_lane.get("lane_id")
if not isinstance(lane_id, str) or not lane_id.strip():
    raise SystemExit("expected first baseline lane to include a non-empty lane_id")
first_lane["lane_id"] = f"{lane_id}__stale_baseline"
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_STALE_BASELINE" >"$TMP_DIR/fail-stale.out" 2>&1; then
  echo "expected ${WAVE_LABEL} trend checker to fail on stale baseline lane inventory" >&2
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-stale.out" '^status=fail$' "expected fail status for stale baseline mutation"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-stale.out" '^mode=trend$' "expected trend mode marker for stale baseline mutation"; then
  exit 1
fi
if ! test_harness_assert_file_contains "$TMP_DIR/fail-stale.out" 'unexpected_new_lanes_in_current_inventory' "expected stale baseline marker"; then
  exit 1
fi

if [ "$FAMILY" = "non_kolme" ]; then
  if bash "$TREND_CHECKER" \
    --matrix-file "$MATRIX_FIXTURE" \
    --baseline-file "$BASELINE_FIXTURE" \
    --max-runtime-seconds 0 >"$TMP_DIR/fail-runtime-budget.out" 2>&1; then
    echo "expected ${WAVE_LABEL} trend checker to fail when CI smoke runtime budget is exceeded" >&2
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-runtime-budget.out" '^status=fail$' "expected fail status for runtime budget mutation"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-runtime-budget.out" '^mode=trend$' "expected trend mode marker for runtime budget mutation"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-runtime-budget.out" 'ci_smoke_runtime_budget_exceeded' "expected runtime budget marker"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-runtime-budget.out" '^ci_smoke_budget_status=exceeded$' "expected ci_smoke_budget_status exceeded marker"; then
    exit 1
  fi
  if ! test_harness_assert_file_contains "$TMP_DIR/fail-runtime-budget.out" '^policy_decision=NO-GO$' "expected policy_decision no-go marker for runtime budget mutation"; then
    exit 1
  fi
fi

echo "${WAVE_LABEL} wrapper-family budget trend checker tests passed."
