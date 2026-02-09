#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/runtime/generate_live_network_pilot_artifact_summary.sh \
    --output-file <path> \
    --event-name <schedule|workflow_dispatch> \
    --cadence <scheduled|manual> \
    --smoke-status <pass|fail> \
    --smoke-decision <GO|NO-GO> \
    --smoke-elapsed-seconds <int> \
    --deep-status <pass|fail> \
    --deep-decision <GO|NO-GO> \
    --deep-elapsed-seconds <int> \
    --budget-status <within|exceeded> \
    --evidence-complete <true|false> \
    --ci-fast-gate <PASS|FAIL>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

to_bool() {
  local field="$1"
  local value="$2"
  case "$value" in
    true|false)
      printf '%s' "$value"
      ;;
    *)
      fail "${field} must be true or false"
      ;;
  esac
}

to_int() {
  local field="$1"
  local value="$2"
  if [[ "$value" =~ ^[0-9]+$ ]]; then
    printf '%s' "$value"
  else
    fail "${field} must be a non-negative integer"
  fi
}

output_file=""
event_name=""
cadence=""
smoke_status=""
smoke_decision=""
smoke_elapsed_seconds=""
deep_status=""
deep_decision=""
deep_elapsed_seconds=""
budget_status=""
evidence_complete_raw=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --event-name)
      event_name="${2:-}"
      shift 2
      ;;
    --cadence)
      cadence="${2:-}"
      shift 2
      ;;
    --smoke-status)
      smoke_status="${2:-}"
      shift 2
      ;;
    --smoke-decision)
      smoke_decision="${2:-}"
      shift 2
      ;;
    --smoke-elapsed-seconds)
      smoke_elapsed_seconds="${2:-}"
      shift 2
      ;;
    --deep-status)
      deep_status="${2:-}"
      shift 2
      ;;
    --deep-decision)
      deep_decision="${2:-}"
      shift 2
      ;;
    --deep-elapsed-seconds)
      deep_elapsed_seconds="${2:-}"
      shift 2
      ;;
    --budget-status)
      budget_status="${2:-}"
      shift 2
      ;;
    --evidence-complete)
      evidence_complete_raw="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$output_file" || -z "$event_name" || -z "$cadence" || -z "$smoke_status" || -z "$smoke_decision" || -z "$smoke_elapsed_seconds" || -z "$deep_status" || -z "$deep_decision" || -z "$deep_elapsed_seconds" || -z "$budget_status" || -z "$evidence_complete_raw" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all arguments are required"
fi

case "$event_name" in
  schedule|workflow_dispatch) ;;
  *)
    fail "--event-name must be schedule or workflow_dispatch"
    ;;
esac

case "$cadence" in
  scheduled|manual) ;;
  *)
    fail "--cadence must be scheduled or manual"
    ;;
esac

case "$smoke_status" in
  pass|fail) ;;
  *)
    fail "--smoke-status must be pass or fail"
    ;;
esac

case "$deep_status" in
  pass|fail) ;;
  *)
    fail "--deep-status must be pass or fail"
    ;;
esac

case "$smoke_decision" in
  GO|NO-GO) ;;
  *)
    fail "--smoke-decision must be GO or NO-GO"
    ;;
esac

case "$deep_decision" in
  GO|NO-GO) ;;
  *)
    fail "--deep-decision must be GO or NO-GO"
    ;;
esac

case "$budget_status" in
  within|exceeded) ;;
  *)
    fail "--budget-status must be within or exceeded"
    ;;
esac

case "$ci_fast_gate" in
  PASS|FAIL) ;;
  *)
    fail "--ci-fast-gate must be PASS or FAIL"
    ;;
esac

smoke_elapsed_seconds="$(to_int "smoke_elapsed_seconds" "$smoke_elapsed_seconds")"
deep_elapsed_seconds="$(to_int "deep_elapsed_seconds" "$deep_elapsed_seconds")"
evidence_complete="$(to_bool "evidence_complete" "$evidence_complete_raw")"

mkdir -p "$(dirname "$output_file")"

python3 - "$output_file" "$event_name" "$cadence" "$smoke_status" "$smoke_decision" "$smoke_elapsed_seconds" "$deep_status" "$deep_decision" "$deep_elapsed_seconds" "$budget_status" "$evidence_complete" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys

(
    output_file,
    event_name,
    cadence,
    smoke_status,
    smoke_decision,
    smoke_elapsed,
    deep_status,
    deep_decision,
    deep_elapsed,
    budget_status,
    evidence_complete,
    ci_fast_gate,
) = sys.argv[1:]

decision_reasons: list[str] = []
if smoke_status != "pass":
    decision_reasons.append("smoke_lane_failed")
if smoke_decision != "GO":
    decision_reasons.append("smoke_decision_no_go")
if deep_status != "pass":
    decision_reasons.append("deep_lane_failed")
if deep_decision != "GO":
    decision_reasons.append("deep_decision_no_go")
if budget_status != "within":
    decision_reasons.append("runtime_budget_exceeded")
if evidence_complete != "true":
    decision_reasons.append("evidence_incomplete")
if ci_fast_gate != "PASS":
    decision_reasons.append("ci_fast_gate_failed")
if event_name not in {"schedule", "workflow_dispatch"}:
    decision_reasons.append("invalid_event")

final_decision = "GO" if not decision_reasons else "NO-GO"

payload = {
    "schema_version": "kamn.runtime.live-network-pilot-artifact-summary.v1",
    "event_name": event_name,
    "cadence": cadence,
    "smoke": {
        "status": smoke_status,
        "final_decision": smoke_decision,
        "elapsed_seconds": int(smoke_elapsed),
    },
    "deep": {
        "status": deep_status,
        "final_decision": deep_decision,
        "elapsed_seconds": int(deep_elapsed),
    },
    "budget_status": budget_status,
    "evidence_complete": evidence_complete == "true",
    "ci_fast_gate": ci_fast_gate,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
}

output_path = pathlib.Path(output_file)
output_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

reason_codes = "none" if not decision_reasons else ",".join(decision_reasons)
print("status=generated")
print(f"summary_file={output_path}")
print(f"final_decision={final_decision}")
print(f"failed_checks={reason_codes}")
PY
