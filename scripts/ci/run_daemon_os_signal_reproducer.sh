#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: run_daemon_os_signal_reproducer.sh [options]

Runs the daemon OS-signal flake corpus repeatedly with bounded runtime and
emits a machine-readable artifact bundle.

Options:
  --attempts <int>             Number of repeated attempts (must be > 0).
  --max-seconds <int>          Runtime budget in seconds (must be > 0).
  --artifact-dir <path>        Directory for per-run logs and report output.
  --output-json <path>         Explicit report output path.
  --stdout-excerpt-lines <int> Number of trailing log lines copied per run.
  --cargo-bin <path>           Cargo executable to run (default: cargo).
  --test <name>                Append explicit test to corpus (repeatable).
  -h, --help                   Show this help.
EOF
}

attempts="${KAMN_DAEMON_OS_SIGNAL_REPRODUCER_ATTEMPTS:-3}"
max_seconds="${KAMN_DAEMON_OS_SIGNAL_REPRODUCER_MAX_SECONDS:-180}"
artifact_dir="${KAMN_DAEMON_OS_SIGNAL_REPRODUCER_ARTIFACT_DIR:-}"
output_json=""
stdout_excerpt_lines="${KAMN_DAEMON_OS_SIGNAL_REPRODUCER_STDOUT_EXCERPT_LINES:-25}"
cargo_bin="${KAMN_DAEMON_OS_SIGNAL_REPRODUCER_CARGO_BIN:-cargo}"
tests=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --attempts)
      attempts="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
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
    --stdout-excerpt-lines)
      stdout_excerpt_lines="${2:-}"
      shift 2
      ;;
    --cargo-bin)
      cargo_bin="${2:-}"
      shift 2
      ;;
    --test)
      tests+=("${2:-}")
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

if ! [[ "$attempts" =~ ^[0-9]+$ ]]; then
  echo "attempts must be a positive integer" >&2
  exit 1
fi
if [ "$attempts" -le 0 ]; then
  echo "attempts must be greater than zero" >&2
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
if ! [[ "$stdout_excerpt_lines" =~ ^[0-9]+$ ]]; then
  echo "stdout-excerpt-lines must be a positive integer" >&2
  exit 1
fi
if [ "$stdout_excerpt_lines" -le 0 ]; then
  echo "stdout-excerpt-lines must be greater than zero" >&2
  exit 1
fi

if [[ ${#tests[@]} -eq 0 ]]; then
  tests=(
    "daemon_shutdown::tests::regression_daemon_completion_with_os_signals_without_signal_stays_bounded"
    "main_tests::daemon_tests::integration_runtime_daemon_applies_graceful_shutdown_on_os_signal"
    "main_tests::runtime_tests::regression_runtime_full_os_signal_stop_markers_project_shutdown_field_parity"
  )
fi

for test_name in "${tests[@]}"; do
  if [[ -z "$test_name" ]]; then
    echo "test identifiers must be non-empty" >&2
    exit 1
  fi
done

if [[ -z "$artifact_dir" ]]; then
  artifact_dir="/tmp/kamn-daemon-os-signal-reproducer"
fi
mkdir -p "$artifact_dir"

if [[ -z "$output_json" ]]; then
  output_json="$artifact_dir/daemon-os-signal-reproducer-report.json"
fi

attempt_rows="$artifact_dir/daemon-os-signal-runs.tsv"
: > "$attempt_rows"

start_epoch="$(date +%s)"
success_count=0
failure_count=0
reason_code=""
status=""
final_decision=""
flaky_detected="false"
runtime_budget_exceeded="false"

for attempt in $(seq 1 "$attempts"); do
  for test_name in "${tests[@]}"; do
    now_epoch="$(date +%s)"
    elapsed_seconds="$(( now_epoch - start_epoch ))"
    if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
      runtime_budget_exceeded="true"
      reason_code="runtime_budget_exceeded"
      status="fail"
      final_decision="NO-GO"
      break 2
    fi

    safe_test_name="$(printf '%s' "$test_name" | tr ':/' '__' | tr -cd '[:alnum:]_.-')"
    log_file="$artifact_dir/attempt-${attempt}-${safe_test_name}.log"

    set +e
    "$cargo_bin" test -p kamn-node "$test_name" -- --exact >"$log_file" 2>&1
    exit_code=$?
    set -e

    run_status="pass"
    if [ "$exit_code" -ne 0 ]; then
      run_status="fail"
      failure_count=$((failure_count + 1))
    else
      success_count=$((success_count + 1))
    fi

    printf '%s\t%s\t%s\t%s\t%s\n' \
      "$attempt" "$test_name" "$run_status" "$exit_code" "$log_file" >> "$attempt_rows"
  done
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
total_runs="$(( success_count + failure_count ))"

if [[ "$runtime_budget_exceeded" != "true" ]]; then
  if [ "$failure_count" -eq 0 ] && [ "$total_runs" -gt 0 ]; then
    reason_code="stable_success"
    status="pass"
    final_decision="GO"
  elif [ "$success_count" -eq 0 ] && [ "$failure_count" -gt 0 ]; then
    reason_code="reproducible_failure"
    status="fail"
    final_decision="NO-GO"
  else
    reason_code="flaky_pattern_observed"
    status="fail"
    final_decision="NO-GO"
    flaky_detected="true"
  fi
fi

python3 - \
  "$output_json" \
  "$attempt_rows" \
  "$attempts" \
  "$max_seconds" \
  "$elapsed_seconds" \
  "$reason_code" \
  "$status" \
  "$final_decision" \
  "$flaky_detected" \
  "$success_count" \
  "$failure_count" \
  "$total_runs" \
  "$artifact_dir" \
  "$stdout_excerpt_lines" \
  "${tests[@]}" <<'PY'
import json
import pathlib
import sys

output_json = pathlib.Path(sys.argv[1])
attempt_rows = pathlib.Path(sys.argv[2])
attempts = int(sys.argv[3])
max_seconds = int(sys.argv[4])
elapsed_seconds = int(sys.argv[5])
reason_code = sys.argv[6]
status = sys.argv[7]
final_decision = sys.argv[8]
flaky_detected = sys.argv[9] == "true"
success_count = int(sys.argv[10])
failure_count = int(sys.argv[11])
total_runs = int(sys.argv[12])
artifact_dir = sys.argv[13]
stdout_excerpt_lines = int(sys.argv[14])
tests = sys.argv[15:]


def extract_failure_markers(text: str, exit_code: int, run_status: str) -> list[str]:
    markers: list[str] = []
    if run_status == "fail":
        if "test result: FAILED" in text:
            markers.append("test_result_failed")
        if "panicked at" in text or "panic:" in text:
            markers.append("panic_detected")
        if "error:" in text:
            markers.append("error_marker_detected")
        if exit_code != 0 and not markers:
            markers.append("nonzero_exit_without_marker")
    return markers


runs = []
for run_index, line in enumerate(
    [row for row in attempt_rows.read_text(encoding="utf-8").splitlines() if row.strip()],
    start=1,
):
    attempt_raw, test_name, run_status, exit_code_raw, log_file = line.split("\t", 4)
    exit_code = int(exit_code_raw)
    log_path = pathlib.Path(log_file)
    text = ""
    if log_path.exists():
        text = log_path.read_text(encoding="utf-8", errors="replace")
    excerpt = "\n".join(text.splitlines()[-stdout_excerpt_lines:])
    runs.append(
        {
            "test_name": test_name,
            "attempt_index": int(attempt_raw),
            "run_index": run_index,
            "status": run_status,
            "exit_code": exit_code,
            "log_file": log_file,
            "stdout_excerpt": excerpt,
            "failure_markers": extract_failure_markers(text, exit_code, run_status),
        }
    )

payload = {
    "schema_version": "kamn.ci.daemon-os-signal-reproducer-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_code": reason_code,
    "attempts": attempts,
    "max_seconds": max_seconds,
    "elapsed_seconds": elapsed_seconds,
    "artifact_dir": artifact_dir,
    "test_count": len(tests),
    "tests": tests,
    "success_count": success_count,
    "failure_count": failure_count,
    "total_runs": total_runs,
    "flaky_detected": flaky_detected,
    "runs": runs,
}

output_json.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "daemon_os_signal_reproducer_status=$status"
echo "daemon_os_signal_reproducer_final_decision=$final_decision"
echo "daemon_os_signal_reproducer_reason_code=$reason_code"
echo "daemon_os_signal_reproducer_attempts=$attempts"
echo "daemon_os_signal_reproducer_total_runs=$total_runs"
echo "daemon_os_signal_reproducer_elapsed_seconds=$elapsed_seconds"
echo "daemon_os_signal_reproducer_artifact_dir=$artifact_dir"
echo "daemon_os_signal_reproducer_report_file=$output_json"

if [[ "$final_decision" != "GO" ]]; then
  exit 1
fi

exit 0
