#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TREND_CHECKER="$ROOT_DIR/scripts/ci/check_non_kolme_wave6_wrapper_family_budget_trend.sh"
PYTHON_CHECKER="$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave6_wrapper_family_trend_thresholds.json"
BASELINE_FIXTURE="$ROOT_DIR/fixtures/ci/non_kolme_wave6_wrapper_family_baseline.json"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/ci/non_kolme_wave6_wrapper_family_matrix.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$TREND_CHECKER" ]; then
  echo "expected non-Kolme wave-6 trend checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$PYTHON_CHECKER" ]; then
  echo "expected python baseline checker script to be executable" >&2
  exit 1
fi

if [ ! -f "$THRESHOLD_FILE" ]; then
  echo "expected non-Kolme wave-6 trend threshold fixture to exist" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FIXTURE" ]; then
  echo "expected non-Kolme wave-6 baseline fixture to exist" >&2
  exit 1
fi

if [ ! -f "$MATRIX_FIXTURE" ]; then
  echo "expected non-Kolme wave-6 matrix fixture to exist" >&2
  exit 1
fi

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$BASELINE_FIXTURE" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^mode=trend$' "$TMP_DIR/pass.out"
grep -q '^wrapper_count_delta=0$' "$TMP_DIR/pass.out"
grep -q '^total_shell_loc_delta=0$' "$TMP_DIR/pass.out"
grep -q '^violation_count=0$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"

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
  echo "expected non-Kolme wave-6 trend checker to fail when total shell LOC delta exceeds threshold" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-total.out"
grep -q '^mode=trend$' "$TMP_DIR/fail-total.out"
grep -q 'total_shell_loc_delta_threshold_exceeded' "$TMP_DIR/fail-total.out"

MUTATED_LANE_MATRIX="$TMP_DIR/mutated-lane-matrix.json"
MUTATED_LANE_WRAPPER="$TMP_DIR/run_bridge_outbound_quorum_contract_lane.sh"
cp "$MATRIX_FIXTURE" "$MUTATED_LANE_MATRIX"
cat >"$MUTATED_LANE_WRAPPER" <<'INNER'
#!/usr/bin/env bash
set -euo pipefail

printf 'mutated bridge outbound quorum lane\n'
INNER

python3 - "$MUTATED_LANE_MATRIX" "$MUTATED_LANE_WRAPPER" <<'PY'
import json
import sys
from pathlib import Path

matrix_path = Path(sys.argv[1])
wrapper_path = Path(sys.argv[2]).resolve()
payload = json.loads(matrix_path.read_text(encoding="utf-8"))

for lane in payload["lanes"]:
    if lane.get("lane_id") == "non_kolme.bridge.outbound_quorum":
        lane["source_entry"] = str(wrapper_path)
        break

matrix_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$TREND_CHECKER" \
  --matrix-file "$MUTATED_LANE_MATRIX" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/fail-lane.out" 2>&1; then
  echo "expected non-Kolme wave-6 trend checker to fail when lane source_entry drifts from baseline" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-lane.out"
grep -q '^mode=trend$' "$TMP_DIR/fail-lane.out"
grep -q 'lane_source_entry_drift' "$TMP_DIR/fail-lane.out"

MUTATED_STALE_BASELINE="$TMP_DIR/mutated-stale-baseline.json"
cp "$BASELINE_FIXTURE" "$MUTATED_STALE_BASELINE"
python3 - "$MUTATED_STALE_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["lanes"] = payload["lanes"][:-1]
payload["wrapper_count"] = len(payload["lanes"])
payload["regular_file_wrapper_count"] = len(payload["lanes"])
payload["symlink_wrapper_count"] = sum(1 for lane in payload["lanes"] if lane.get("wrapper_kind") == "symlink")
payload["total_shell_loc"] = sum(int(lane["shell_loc"]) for lane in payload["lanes"])
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_STALE_BASELINE" >"$TMP_DIR/fail-stale.out" 2>&1; then
  echo "expected non-Kolme wave-6 trend checker to fail on stale baseline lane inventory" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-stale.out"
grep -q '^mode=trend$' "$TMP_DIR/fail-stale.out"
grep -q 'unexpected_new_lanes_in_current_inventory' "$TMP_DIR/fail-stale.out"

echo "non-Kolme wave-6 wrapper-family budget trend checker tests passed."
