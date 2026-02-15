#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: run_daemon_os_signal_stress_matrix.sh [options]

Runs repeated daemon OS-signal reproducer iterations and emits an auditable
stress-matrix artifact bundle.

Options:
  --iterations <int>              Iteration count (must be > 0).
  --attempts-per-iteration <int>  Reproducer attempts per iteration (must be > 0).
  --max-seconds <int>             Total wall-clock runtime budget in seconds.
  --reproducer-max-seconds <int>  Per-iteration reproducer budget in seconds.
  --failure-threshold <int>       Allowed failed iterations before NO-GO.
  --artifact-dir <path>           Directory for iteration artifacts and matrix report.
  --output-json <path>            Explicit matrix report output path.
  --registry-file <path>          Flaky quarantine registry to audit.
  --quarantine-followup-issue <id> Follow-up issue id (for example: #5000).
  --reproducer-script <path>      Reproducer script path.
  -h, --help                      Show this help.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

iterations="${KAMN_DAEMON_OS_SIGNAL_STRESS_ITERATIONS:-10}"
attempts_per_iteration="${KAMN_DAEMON_OS_SIGNAL_STRESS_ATTEMPTS_PER_ITERATION:-1}"
max_seconds="${KAMN_DAEMON_OS_SIGNAL_STRESS_MAX_SECONDS:-600}"
reproducer_max_seconds="${KAMN_DAEMON_OS_SIGNAL_STRESS_REPRODUCER_MAX_SECONDS:-180}"
failure_threshold="${KAMN_DAEMON_OS_SIGNAL_STRESS_FAILURE_THRESHOLD:-0}"
artifact_dir="${KAMN_DAEMON_OS_SIGNAL_STRESS_ARTIFACT_DIR:-}"
output_json=""
registry_file="${KAMN_DAEMON_OS_SIGNAL_STRESS_REGISTRY_FILE:-.ci/flaky-tests.txt}"
quarantine_followup_issue="${KAMN_DAEMON_OS_SIGNAL_STRESS_QUARANTINE_FOLLOWUP_ISSUE:-}"
reproducer_script="${KAMN_DAEMON_OS_SIGNAL_STRESS_REPRODUCER_SCRIPT:-$script_dir/run_daemon_os_signal_reproducer.sh}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --iterations)
      iterations="${2:-}"
      shift 2
      ;;
    --attempts-per-iteration)
      attempts_per_iteration="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --reproducer-max-seconds)
      reproducer_max_seconds="${2:-}"
      shift 2
      ;;
    --failure-threshold)
      failure_threshold="${2:-}"
      shift 2
      ;;
    --artifact-dir)
      artifact_dir="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --registry-file)
      registry_file="${2:-}"
      shift 2
      ;;
    --quarantine-followup-issue)
      quarantine_followup_issue="${2:-}"
      shift 2
      ;;
    --reproducer-script)
      reproducer_script="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$iterations" =~ ^[0-9]+$ ]]; then
  echo "iterations must be a positive integer" >&2
  exit 1
fi
if [ "$iterations" -le 0 ]; then
  echo "iterations must be greater than zero" >&2
  exit 1
fi
if ! [[ "$attempts_per_iteration" =~ ^[0-9]+$ ]]; then
  echo "attempts-per-iteration must be a positive integer" >&2
  exit 1
fi
if [ "$attempts_per_iteration" -le 0 ]; then
  echo "attempts-per-iteration must be greater than zero" >&2
  exit 1
fi
if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if ! [[ "$reproducer_max_seconds" =~ ^[0-9]+$ ]]; then
  echo "reproducer-max-seconds must be a positive integer" >&2
  exit 1
fi
if [ "$reproducer_max_seconds" -le 0 ]; then
  echo "reproducer-max-seconds must be greater than zero" >&2
  exit 1
fi
if ! [[ "$failure_threshold" =~ ^[0-9]+$ ]]; then
  echo "failure-threshold must be a non-negative integer" >&2
  exit 1
fi
if [ "$failure_threshold" -gt "$iterations" ]; then
  echo "failure-threshold must be less than or equal to iterations" >&2
  exit 1
fi
if [ ! -x "$reproducer_script" ]; then
  echo "reproducer-script must be executable: $reproducer_script" >&2
  exit 1
fi
if [[ -n "$quarantine_followup_issue" ]] && [[ ! "$quarantine_followup_issue" =~ ^#[0-9]+$ ]]; then
  echo "quarantine-followup-issue must match '#<id>' when provided" >&2
  exit 1
fi

if [[ -z "$artifact_dir" ]]; then
  artifact_dir="/tmp/kamn-daemon-os-signal-stress-matrix"
fi
mkdir -p "$artifact_dir"

if [[ -z "$output_json" ]]; then
  output_json="$artifact_dir/daemon-os-signal-stress-matrix-report.json"
fi

iteration_rows="$artifact_dir/daemon-os-signal-stress-iterations.tsv"
: > "$iteration_rows"

targeted_tests=(
  "daemon_shutdown::tests::regression_daemon_completion_with_os_signals_without_signal_stays_bounded"
  "main_tests::daemon_tests::integration_runtime_daemon_applies_graceful_shutdown_on_os_signal"
  "main_tests::runtime_tests::regression_runtime_full_os_signal_stop_markers_project_shutdown_field_parity"
)

quarantine_status="absent"
quarantine_reason_code="none"
quarantine_references_file="$artifact_dir/quarantine-references.txt"
: > "$quarantine_references_file"

if [ ! -f "$registry_file" ]; then
  quarantine_status="registry-missing"
  quarantine_reason_code="quarantine_registry_missing"
else
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      ""|\#*)
        continue
        ;;
    esac
    IFS='|' read -r _owner test_id _issue _expiry _notes _extra <<<"$line"
    for targeted in "${targeted_tests[@]}"; do
      if [ "$test_id" = "$targeted" ]; then
        printf '%s\n' "$test_id" >> "$quarantine_references_file"
      fi
    done
  done < "$registry_file"
fi

if [ -s "$quarantine_references_file" ]; then
  if [[ -n "$quarantine_followup_issue" ]]; then
    quarantine_status="justified-followup"
    quarantine_reason_code="quarantine_reference_justified_with_followup"
  else
    quarantine_status="present-without-followup"
    quarantine_reason_code="quarantine_reference_present_without_followup"
  fi
fi

start_epoch="$(date +%s)"
pass_iterations=0
fail_iterations=0
reason_code=""
status=""
final_decision=""
runtime_budget_exceeded="false"

for iteration in $(seq 1 "$iterations"); do
  now_epoch="$(date +%s)"
  elapsed_seconds="$(( now_epoch - start_epoch ))"
  if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
    runtime_budget_exceeded="true"
    break
  fi

  iteration_artifact_dir="$artifact_dir/iteration-$iteration"
  mkdir -p "$iteration_artifact_dir"
  iteration_report="$iteration_artifact_dir/daemon-os-signal-reproducer-report.json"
  iteration_output="$iteration_artifact_dir/daemon-os-signal-reproducer.out"

  set +e
  env \
    KAMN_DAEMON_STRESS_MATRIX_ITERATION="$iteration" \
    bash "$reproducer_script" \
      --attempts "$attempts_per_iteration" \
      --max-seconds "$reproducer_max_seconds" \
      --artifact-dir "$iteration_artifact_dir" \
      --output-json "$iteration_report" >"$iteration_output" 2>&1
  iteration_exit_code=$?
  set -e

  iteration_status="pass"
  if [ "$iteration_exit_code" -ne 0 ]; then
    iteration_status="fail"
    fail_iterations=$((fail_iterations + 1))
  else
    pass_iterations=$((pass_iterations + 1))
  fi

  reproducer_status="unknown"
  reproducer_decision="unknown"
  reproducer_reason_code="reproducer_report_missing"
  if [ -f "$iteration_report" ]; then
    read -r reproducer_status reproducer_decision reproducer_reason_code <<<"$(python3 - "$iteration_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(
    report.get("status", "unknown"),
    report.get("final_decision", "unknown"),
    report.get("reason_code", "reproducer_reason_missing"),
)
PY
)"
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$iteration" \
    "$iteration_status" \
    "$iteration_exit_code" \
    "$iteration_report" \
    "$iteration_output" \
    "$reproducer_status" \
    "$reproducer_reason_code" >> "$iteration_rows"
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"

if [[ "$runtime_budget_exceeded" = "true" ]]; then
  status="fail"
  final_decision="NO-GO"
  reason_code="runtime_budget_exceeded"
elif [ "$fail_iterations" -gt "$failure_threshold" ]; then
  status="fail"
  final_decision="NO-GO"
  reason_code="matrix_failure_threshold_exceeded"
elif [ "$quarantine_status" = "registry-missing" ]; then
  status="fail"
  final_decision="NO-GO"
  reason_code="$quarantine_reason_code"
elif [ "$quarantine_status" = "present-without-followup" ]; then
  status="fail"
  final_decision="NO-GO"
  reason_code="$quarantine_reason_code"
elif [ "$fail_iterations" -gt 0 ]; then
  status="pass"
  final_decision="GO"
  reason_code="matrix_failures_within_threshold"
elif [ "$quarantine_status" = "justified-followup" ]; then
  status="pass"
  final_decision="GO"
  reason_code="stable_success_with_quarantine_followup"
else
  status="pass"
  final_decision="GO"
  reason_code="stable_success"
fi

python3 - \
  "$output_json" \
  "$iteration_rows" \
  "$iterations" \
  "$attempts_per_iteration" \
  "$max_seconds" \
  "$reproducer_max_seconds" \
  "$failure_threshold" \
  "$elapsed_seconds" \
  "$pass_iterations" \
  "$fail_iterations" \
  "$status" \
  "$final_decision" \
  "$reason_code" \
  "$registry_file" \
  "$quarantine_status" \
  "$quarantine_reason_code" \
  "$quarantine_followup_issue" \
  "$quarantine_references_file" \
  "$artifact_dir" <<'PY'
import json
import pathlib
import sys

output_json = pathlib.Path(sys.argv[1])
iteration_rows = pathlib.Path(sys.argv[2])
iterations = int(sys.argv[3])
attempts_per_iteration = int(sys.argv[4])
max_seconds = int(sys.argv[5])
reproducer_max_seconds = int(sys.argv[6])
failure_threshold = int(sys.argv[7])
elapsed_seconds = int(sys.argv[8])
pass_iterations = int(sys.argv[9])
fail_iterations = int(sys.argv[10])
status = sys.argv[11]
final_decision = sys.argv[12]
reason_code = sys.argv[13]
registry_file = sys.argv[14]
quarantine_status = sys.argv[15]
quarantine_reason_code = sys.argv[16]
quarantine_followup_issue = sys.argv[17]
quarantine_references_file = pathlib.Path(sys.argv[18])
artifact_dir = sys.argv[19]

iteration_results = []
anti_flake_chain_artifacts = []
for line in iteration_rows.read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    (
        iteration_raw,
        iteration_status,
        exit_code_raw,
        report_file,
        output_file,
        reproducer_status,
        reproducer_reason_code,
    ) = line.split("\t", 6)
    row = {
        "iteration_index": int(iteration_raw),
        "status": iteration_status,
        "exit_code": int(exit_code_raw),
        "reproducer_status": reproducer_status,
        "reproducer_reason_code": reproducer_reason_code,
        "reproducer_report_file": report_file,
        "reproducer_output_file": output_file,
    }
    iteration_results.append(row)
    anti_flake_chain_artifacts.append(report_file)

quarantine_references = []
if quarantine_references_file.exists():
    quarantine_references = [
        line.strip()
        for line in quarantine_references_file.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]

payload = {
    "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_code": reason_code,
    "iterations": iterations,
    "attempts_per_iteration": attempts_per_iteration,
    "failure_threshold": failure_threshold,
    "max_seconds": max_seconds,
    "reproducer_max_seconds": reproducer_max_seconds,
    "elapsed_seconds": elapsed_seconds,
    "pass_iterations": pass_iterations,
    "fail_iterations": fail_iterations,
    "registry_file": registry_file,
    "quarantine_status": quarantine_status,
    "quarantine_reason_code": quarantine_reason_code,
    "quarantine_followup_issue": quarantine_followup_issue or None,
    "quarantine_references": quarantine_references,
    "anti_flake_chain_reproducer_schema_version": "kamn.ci.daemon-os-signal-reproducer-report.v1",
    "anti_flake_chain_artifacts": anti_flake_chain_artifacts,
    "artifact_dir": artifact_dir,
    "iteration_results": iteration_results,
}
output_json.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "daemon_os_signal_stress_matrix_status=$status"
echo "daemon_os_signal_stress_matrix_final_decision=$final_decision"
echo "daemon_os_signal_stress_matrix_reason_code=$reason_code"
echo "daemon_os_signal_stress_matrix_iterations=$iterations"
echo "daemon_os_signal_stress_matrix_pass_iterations=$pass_iterations"
echo "daemon_os_signal_stress_matrix_fail_iterations=$fail_iterations"
echo "daemon_os_signal_stress_matrix_failure_threshold=$failure_threshold"
echo "daemon_os_signal_stress_matrix_quarantine_status=$quarantine_status"
echo "daemon_os_signal_stress_matrix_report_file=$output_json"
echo "daemon_os_signal_stress_matrix_artifact_dir=$artifact_dir"

if [[ "$final_decision" != "GO" ]]; then
  exit 1
fi

exit 0
