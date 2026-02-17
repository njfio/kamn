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
baseline_report="$TMP_DIR/go-no-go-gate-baseline.json"
baseline_output="$(
  KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash "$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh" \
    --mode run \
    --max-seconds 120 \
    --output-json "$baseline_report"
)"

if ! printf '%s\n' "$baseline_output" | grep -q '^status=pass$'; then
  echo "expected go/no-go gate baseline status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^final_decision=GO$'; then
  echo "expected go/no-go gate baseline GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^lane_mode=run$'; then
  echo "expected go/no-go gate baseline run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^run_mode_command_status=executed$'; then
  echo "expected go/no-go gate baseline run command status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^ci_fast_gate_eligible=false$'; then
  echo "expected go/no-go gate baseline fast-gate exclusion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^ci_fast_gate_scope=local-only$'; then
  echo "expected go/no-go gate baseline local-only scope marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^go_no_go_evidence_status=verified$'; then
  echo "expected go/no-go gate baseline evidence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^rollback_readiness_status=verified$'; then
  echo "expected go/no-go gate baseline rollback marker" >&2
  exit 1
fi
if ! printf '%s\n' "$baseline_output" | grep -q '^dr_readiness_status=verified$'; then
  echo "expected go/no-go gate baseline dr marker" >&2
  exit 1
fi

fault_report="$TMP_DIR/go-no-go-gate-fault.json"
set +e
fault_output="$(
  KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash "$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh" \
    --mode run \
    --fault-profile gate_decision \
    --max-seconds 120 \
    --output-json "$fault_report" 2>&1
)"
fault_code=$?
set -e
if [ "$fault_code" -eq 0 ]; then
  echo "expected go/no-go gate decision fault profile to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fault_output" | grep -q 'gate_decision_fault_injection_triggered'; then
  echo "expected go/no-go decision fault reason marker in live validation" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "go/no-go gate live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/go-no-go-gate-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.go-no-go-gate-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "run",
  "run_mode_command_status": "executed",
  "ci_fast_gate_scope": "local-only",
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
echo "lane_mode=run"
echo "run_mode_command_status=executed"
echo "ci_fast_gate_scope=local-only"
echo "baseline_contract_status=verified"
echo "fault_injection_status=verified"
echo "fail_closed_status=verified"
