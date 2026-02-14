#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
baseline_report="$TMP_DIR/failure-drills-baseline.json"
baseline_output="$(
  bash "$ROOT_DIR/scripts/runtime/run_network_signer_finality_failure_drills_lane.sh" \
    --max-seconds 180 \
    --partition-max-seconds 60 \
    --signer-max-seconds 60 \
    --output-json "$baseline_report"
)"

if ! printf '%s\n' "$baseline_output" | grep -q '^status=pass$'; then
  echo "expected failure drills baseline status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^final_decision=GO$'; then
  echo "expected failure drills baseline GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^network_partition_status=verified$'; then
  echo "expected failure drills baseline network partition marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^signer_fault_status=verified$'; then
  echo "expected failure drills baseline signer marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^finality_fault_status=verified$'; then
  echo "expected failure drills baseline finality marker" >&2
  exit 1
fi

fault_report="$TMP_DIR/failure-drills-fault.json"
set +e
fault_output="$(
  bash "$ROOT_DIR/scripts/runtime/run_network_signer_finality_failure_drills_lane.sh" \
    --fault-profile signer \
    --max-seconds 180 \
    --partition-max-seconds 60 \
    --signer-max-seconds 60 \
    --output-json "$fault_report" 2>&1
)"
fault_code=$?
set -e
if [ "$fault_code" -eq 0 ]; then
  echo "expected failure drills signer fault profile to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fault_output" | grep -q 'signer_fault_injection_triggered'; then
  echo "expected signer fault-injection reason marker in failure drills live validation" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "failure drills live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/failure-drills-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.runtime.failure-drills-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "baseline_contract_status": "verified",
  "fault_injection_status": "verified",
  "fail_closed_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "baseline_contract_status=verified"
echo "fault_injection_status=verified"
echo "fail_closed_status=verified"
