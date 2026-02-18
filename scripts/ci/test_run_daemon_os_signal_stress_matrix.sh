#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/ci/run_daemon_os_signal_stress_matrix.sh"

test_harness_require_executable "$SCRIPT" "expected daemon os-signal stress matrix script to be executable"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cat >"$TMP_DIR/mock-reproducer.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

attempts=1
output_json=""
artifact_dir=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --attempts)
      attempts="${2:-1}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --artifact-dir)
      artifact_dir="${2:-}"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$output_json" ]; then
  echo "mock reproducer expected --output-json" >&2
  exit 2
fi

iteration="${KAMN_DAEMON_STRESS_MATRIX_ITERATION:-0}"
status="pass"
final_decision="GO"
reason_code="stable_success"
exit_code=0

IFS=',' read -r -a fail_iterations <<<"${KAMN_DAEMON_STRESS_FAIL_ITERATIONS:-}"
for candidate in "${fail_iterations[@]}"; do
  if [ "$candidate" = "$iteration" ]; then
    status="fail"
    final_decision="NO-GO"
    reason_code="flaky_pattern_observed"
    exit_code=1
    break
  fi
done

python3 - "$output_json" "$status" "$final_decision" "$reason_code" "$attempts" "$artifact_dir" <<'PY'
import json
import pathlib
import sys

output_json = pathlib.Path(sys.argv[1])
status = sys.argv[2]
final_decision = sys.argv[3]
reason_code = sys.argv[4]
attempts = int(sys.argv[5])
artifact_dir = sys.argv[6]
payload = {
    "schema_version": "kamn.ci.daemon-os-signal-reproducer-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_code": reason_code,
    "attempts": attempts,
    "max_seconds": 10,
    "elapsed_seconds": 0,
    "artifact_dir": artifact_dir,
    "test_count": 3,
    "tests": ["a", "b", "c"],
    "success_count": 3 if status == "pass" else 2,
    "failure_count": 0 if status == "pass" else 1,
    "total_runs": 3,
    "flaky_detected": status != "pass",
    "runs": [],
}
output_json.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "daemon_os_signal_reproducer_status=$status"
echo "daemon_os_signal_reproducer_final_decision=$final_decision"
echo "daemon_os_signal_reproducer_reason_code=$reason_code"
echo "daemon_os_signal_reproducer_attempts=$attempts"
echo "daemon_os_signal_reproducer_artifact_dir=$artifact_dir"
echo "daemon_os_signal_reproducer_report_file=$output_json"

exit "$exit_code"
EOF
chmod +x "$TMP_DIR/mock-reproducer.sh"

stable_artifact_dir="$TMP_DIR/stable-artifacts"
stable_report="$TMP_DIR/stable-report.json"
stable_output="$(
  bash "$SCRIPT" \
    --iterations 3 \
    --attempts-per-iteration 1 \
    --max-seconds 60 \
    --failure-threshold 0 \
    --artifact-dir "$stable_artifact_dir" \
    --output-json "$stable_report" \
    --reproducer-script "$TMP_DIR/mock-reproducer.sh"
)"
for marker in \
  '^daemon_os_signal_stress_matrix_status=pass$' \
  '^daemon_os_signal_stress_matrix_reason_code=stable_success$' \
  '^daemon_os_signal_stress_matrix_final_decision=GO$'; do
  if ! printf '%s\n' "$stable_output" | grep -q "$marker"; then
    echo "expected stable daemon stress matrix marker: $marker" >&2
    exit 1
  fi
done

python3 - "$stable_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.ci.daemon-os-signal-stress-matrix-report.v1":
    raise SystemExit("unexpected daemon stress matrix schema")
if report.get("status") != "pass":
    raise SystemExit("expected daemon stress matrix stable status=pass")
if report.get("final_decision") != "GO":
    raise SystemExit("expected daemon stress matrix stable final_decision=GO")
if report.get("reason_code") != "stable_success":
    raise SystemExit("expected daemon stress matrix stable reason_code=stable_success")
if report.get("iterations") != 3:
    raise SystemExit("expected daemon stress matrix iterations=3")
if report.get("pass_iterations") != 3 or report.get("fail_iterations") != 0:
    raise SystemExit("expected all daemon stress matrix iterations to pass")
if report.get("quarantine_status") != "absent":
    raise SystemExit("expected daemon stress matrix quarantine status absent")
rows = report.get("iteration_results")
if not isinstance(rows, list) or len(rows) != 3:
    raise SystemExit("expected three daemon stress matrix iteration rows")
required_keys = {
    "iteration_index",
    "status",
    "exit_code",
    "reproducer_reason_code",
    "reproducer_report_file",
}
missing = required_keys - set(rows[0].keys())
if missing:
    raise SystemExit(f"missing daemon stress matrix row keys: {sorted(missing)}")
PY

flaky_artifact_dir="$TMP_DIR/flaky-artifacts"
flaky_report="$TMP_DIR/flaky-report.json"
flaky_output_file="$TMP_DIR/flaky-output.log"
set +e
env \
  KAMN_DAEMON_STRESS_FAIL_ITERATIONS=2 \
  bash "$SCRIPT" \
  --iterations 3 \
  --attempts-per-iteration 1 \
  --max-seconds 60 \
  --failure-threshold 0 \
  --artifact-dir "$flaky_artifact_dir" \
  --output-json "$flaky_report" \
  --reproducer-script "$TMP_DIR/mock-reproducer.sh" >"$flaky_output_file" 2>&1
flaky_status=$?
set -e
flaky_output="$(cat "$flaky_output_file")"
if [ "$flaky_status" -eq 0 ]; then
  echo "expected daemon stress matrix to fail closed when threshold is exceeded" >&2
  exit 1
fi
for marker in \
  '^daemon_os_signal_stress_matrix_status=fail$' \
  '^daemon_os_signal_stress_matrix_final_decision=NO-GO$' \
  '^daemon_os_signal_stress_matrix_reason_code=matrix_failure_threshold_exceeded$'; do
  if ! printf '%s\n' "$flaky_output" | grep -q "$marker"; then
    echo "expected flaky daemon stress matrix marker: $marker" >&2
    exit 1
  fi
done

python3 - "$flaky_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("status") != "fail":
    raise SystemExit("expected flaky daemon stress matrix status=fail")
if report.get("reason_code") != "matrix_failure_threshold_exceeded":
    raise SystemExit("expected flaky daemon stress matrix threshold reason")
if report.get("fail_iterations") != 1:
    raise SystemExit("expected one failed daemon stress matrix iteration")
if report.get("pass_iterations") != 2:
    raise SystemExit("expected two passed daemon stress matrix iterations")
PY

cat >"$TMP_DIR/flaky-tests.txt" <<'EOF'
# owner|test-id|issue|expiry|notes
qa|main_tests::runtime_tests::regression_runtime_full_os_signal_stop_markers_project_shutdown_field_parity|#3762|2099-12-31|stale quarantine marker
EOF

quarantine_artifact_dir="$TMP_DIR/quarantine-artifacts"
quarantine_report="$TMP_DIR/quarantine-report.json"
set +e
quarantine_output="$(
  bash "$SCRIPT" \
    --iterations 1 \
    --attempts-per-iteration 1 \
    --max-seconds 60 \
    --failure-threshold 0 \
    --artifact-dir "$quarantine_artifact_dir" \
    --output-json "$quarantine_report" \
    --registry-file "$TMP_DIR/flaky-tests.txt" \
    --reproducer-script "$TMP_DIR/mock-reproducer.sh" 2>&1
)"
quarantine_status=$?
set -e
if [ "$quarantine_status" -eq 0 ]; then
  echo "expected daemon stress matrix to fail when quarantine references remain without follow-up" >&2
  exit 1
fi
if ! printf '%s\n' "$quarantine_output" | grep -q '^daemon_os_signal_stress_matrix_reason_code=quarantine_reference_present_without_followup$'; then
  echo "expected quarantine reason marker for daemon stress matrix" >&2
  exit 1
fi

quarantine_waived_artifact_dir="$TMP_DIR/quarantine-waived-artifacts"
quarantine_waived_report="$TMP_DIR/quarantine-waived-report.json"
quarantine_waived_output="$(
  bash "$SCRIPT" \
    --iterations 1 \
    --attempts-per-iteration 1 \
    --max-seconds 60 \
    --failure-threshold 0 \
    --artifact-dir "$quarantine_waived_artifact_dir" \
    --output-json "$quarantine_waived_report" \
    --registry-file "$TMP_DIR/flaky-tests.txt" \
    --quarantine-followup-issue '#5000' \
    --reproducer-script "$TMP_DIR/mock-reproducer.sh"
)"
if ! printf '%s\n' "$quarantine_waived_output" | grep -q '^daemon_os_signal_stress_matrix_status=pass$'; then
  echo "expected daemon stress matrix to pass when quarantine references have follow-up issue" >&2
  exit 1
fi

python3 - "$quarantine_waived_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("quarantine_status") != "justified-followup":
    raise SystemExit("expected daemon stress matrix quarantine_status=justified-followup")
if report.get("quarantine_followup_issue") != "#5000":
    raise SystemExit("expected daemon stress matrix follow-up issue marker")
PY

set +e
invalid_iterations_output="$(
  bash "$SCRIPT" \
    --iterations 0 \
    --attempts-per-iteration 1 \
    --max-seconds 60 \
    --failure-threshold 0 \
    --artifact-dir "$TMP_DIR/invalid-artifacts" \
    --output-json "$TMP_DIR/invalid-report.json" \
    --reproducer-script "$TMP_DIR/mock-reproducer.sh" 2>&1
)"
invalid_iterations_status=$?
set -e
if [ "$invalid_iterations_status" -eq 0 ]; then
  echo "expected iterations=0 validation failure for daemon stress matrix" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_iterations_output" | grep -q "iterations must be greater than zero"; then
  echo "expected deterministic daemon stress matrix iteration validation marker" >&2
  exit 1
fi

echo "run_daemon_os_signal_stress_matrix tests passed."
