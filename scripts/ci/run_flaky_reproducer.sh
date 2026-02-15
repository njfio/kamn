#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: run_flaky_reproducer.sh [options] -- <command...>

Runs a command repeatedly with a deterministic seed and captures attempt artifacts.

Options:
  --seed <int>            Deterministic seed to export to each attempt.
  --attempts <int>        Number of repeated attempts (must be > 0).
  --max-seconds <int>     Runtime budget in seconds (must be > 0).
  --artifact-dir <path>   Directory for per-attempt logs and report output.
  --label <value>         Stable harness label for reporting.
  --output-json <path>    Explicit report output path.
  -h, --help              Show this help.
EOF
}

seed="${KAMN_FLAKY_REPRODUCER_SEED:-13}"
attempts="${KAMN_FLAKY_REPRODUCER_ATTEMPTS:-5}"
max_seconds="${KAMN_FLAKY_REPRODUCER_MAX_SECONDS:-120}"
artifact_dir="${KAMN_FLAKY_REPRODUCER_ARTIFACT_DIR:-}"
label="${KAMN_FLAKY_REPRODUCER_LABEL:-flaky-reproducer}"
output_json=""
command=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --seed)
      seed="${2:-}"
      shift 2
      ;;
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
    --label)
      label="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    --)
      shift
      command=("$@")
      break
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ ${#command[@]} -eq 0 ]]; then
  echo "expected command after --" >&2
  usage >&2
  exit 1
fi

if ! [[ "$seed" =~ ^[0-9]+$ ]]; then
  echo "seed must be a non-negative integer" >&2
  exit 1
fi
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
if [[ -z "$artifact_dir" ]]; then
  artifact_dir="/tmp/kamn-flaky-reproducer-seed-${seed}"
fi

mkdir -p "$artifact_dir"
if [[ -z "$output_json" ]]; then
  output_json="$artifact_dir/flaky-reproducer-report.json"
fi

attempt_rows="$artifact_dir/attempt-results.tsv"
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
  now_epoch="$(date +%s)"
  elapsed_seconds="$(( now_epoch - start_epoch ))"
  if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
    runtime_budget_exceeded="true"
    reason_code="runtime_budget_exceeded"
    status="fail"
    final_decision="NO-GO"
    break
  fi

  log_file="$artifact_dir/attempt-${attempt}.log"
  set +e
  env \
    KAMN_FLAKY_REPRODUCER_SEED="$seed" \
    KAMN_FLAKY_REPRODUCER_ATTEMPT="$attempt" \
    "${command[@]}" >"$log_file" 2>&1
  exit_code=$?
  set -e

  attempt_status="pass"
  if [ "$exit_code" -ne 0 ]; then
    attempt_status="fail"
    failure_count=$((failure_count + 1))
  else
    success_count=$((success_count + 1))
  fi
  printf '%s\t%s\t%s\t%s\n' "$attempt" "$attempt_status" "$exit_code" "$log_file" >> "$attempt_rows"
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"

if [[ "$runtime_budget_exceeded" != "true" ]]; then
  if [ "$success_count" -eq "$attempts" ]; then
    reason_code="stable_success"
    status="pass"
    final_decision="GO"
  elif [ "$failure_count" -eq "$attempts" ]; then
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

python3 - "$output_json" "$attempt_rows" "$seed" "$attempts" "$max_seconds" "$label" "$elapsed_seconds" "$reason_code" "$status" "$final_decision" "$flaky_detected" "$success_count" "$failure_count" "$artifact_dir" "${command[@]}" <<'PY'
import json
import pathlib
import sys

output_json = pathlib.Path(sys.argv[1])
attempt_rows = pathlib.Path(sys.argv[2])
seed = int(sys.argv[3])
attempts = int(sys.argv[4])
max_seconds = int(sys.argv[5])
label = sys.argv[6]
elapsed_seconds = int(sys.argv[7])
reason_code = sys.argv[8]
status = sys.argv[9]
final_decision = sys.argv[10]
flaky_detected = sys.argv[11] == "true"
success_count = int(sys.argv[12])
failure_count = int(sys.argv[13])
artifact_dir = sys.argv[14]
command = sys.argv[15:]

attempt_results = []
for line in attempt_rows.read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    attempt_raw, status_raw, exit_code_raw, log_file = line.split("\t", 3)
    attempt_results.append(
        {
            "attempt": int(attempt_raw),
            "status": status_raw,
            "exit_code": int(exit_code_raw),
            "log_file": log_file,
        }
    )

payload = {
    "schema_version": "kamn.ci.flaky-reproducer-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_code": reason_code,
    "seed": seed,
    "attempts": attempts,
    "max_seconds": max_seconds,
    "label": label,
    "elapsed_seconds": elapsed_seconds,
    "command": command,
    "command_string": " ".join(command),
    "flaky_detected": flaky_detected,
    "success_count": success_count,
    "failure_count": failure_count,
    "artifact_dir": artifact_dir,
    "attempt_results": attempt_results,
}
output_json.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "flaky_reproducer_status=$status"
echo "flaky_reproducer_final_decision=$final_decision"
echo "flaky_reproducer_reason_code=$reason_code"
echo "flaky_reproducer_seed=$seed"
echo "flaky_reproducer_attempts=$attempts"
echo "flaky_reproducer_elapsed_seconds=$elapsed_seconds"
echo "flaky_reproducer_artifact_dir=$artifact_dir"
echo "flaky_reproducer_report_file=$output_json"

if [[ "$final_decision" != "GO" ]]; then
  exit 1
fi

exit 0
