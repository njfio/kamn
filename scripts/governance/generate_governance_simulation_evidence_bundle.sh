#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/generate_governance_simulation_evidence_bundle.sh \
    --output-file <path> \
    --proposal-id <value> \
    --simulation-hash sha256:<64-hex> \
    --simulation-complete true|false \
    --veto-window-open true|false \
    --veto-recorded true|false \
    --timelock-expired true|false \
    --required-approvals <n> \
    --received-approvals <n> \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

normalize_bool() {
  local input="$1"
  case "$input" in
    true|false)
      printf '%s\n' "$input"
      ;;
    *)
      fail "boolean fields must be true or false"
      ;;
  esac
}

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be an integer"
  fi
}

output_file=""
proposal_id=""
simulation_hash=""
simulation_complete=""
veto_window_open=""
veto_recorded=""
timelock_expired=""
required_approvals=""
received_approvals=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --proposal-id)
      proposal_id="${2:-}"
      shift 2
      ;;
    --simulation-hash)
      simulation_hash="${2:-}"
      shift 2
      ;;
    --simulation-complete)
      simulation_complete="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --veto-window-open)
      veto_window_open="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --veto-recorded)
      veto_recorded="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --timelock-expired)
      timelock_expired="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --required-approvals)
      required_approvals="${2:-}"
      shift 2
      ;;
    --received-approvals)
      received_approvals="${2:-}"
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

if [[ -z "$output_file" || -z "$proposal_id" || -z "$simulation_hash" || -z "$simulation_complete" || -z "$veto_window_open" || -z "$veto_recorded" || -z "$timelock_expired" || -z "$required_approvals" || -z "$received_approvals" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all bundle arguments are required"
fi

for status in "$ci_fast_gate"; do
  if [[ "$status" != "PASS" && "$status" != "FAIL" ]]; then
    fail "ci-fast-gate must be PASS or FAIL"
  fi
done

require_int "required-approvals" "$required_approvals"
require_int "received-approvals" "$received_approvals"

if (( required_approvals < 1 )); then
  fail "required-approvals must be >= 1"
fi

if (( received_approvals < 0 )); then
  fail "received-approvals must be >= 0"
fi

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$proposal_id" "$simulation_hash" "$simulation_complete" "$veto_window_open" "$veto_recorded" "$timelock_expired" "$required_approvals" "$received_approvals" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys

(
    output_file,
    generated_at,
    proposal_id,
    simulation_hash,
    simulation_complete_raw,
    veto_window_open_raw,
    veto_recorded_raw,
    timelock_expired_raw,
    required_approvals_raw,
    received_approvals_raw,
    ci_fast_gate,
) = sys.argv[1:]

simulation_complete = simulation_complete_raw == "true"
veto_window_open = veto_window_open_raw == "true"
veto_recorded = veto_recorded_raw == "true"
timelock_expired = timelock_expired_raw == "true"
required_approvals = int(required_approvals_raw)
received_approvals = int(received_approvals_raw)

hash_valid = bool(re.match(r"^sha256:[0-9a-f]{64}$", simulation_hash))
approval_quorum_met = received_approvals >= required_approvals

is_go = (
    simulation_complete
    and hash_valid
    and not veto_window_open
    and not veto_recorded
    and timelock_expired
    and approval_quorum_met
    and ci_fast_gate == "PASS"
)
final_decision = "GO" if is_go else "NO-GO"

reason_codes = []
if not simulation_complete:
    reason_codes.append("simulation_missing")
if not hash_valid:
    reason_codes.append("simulation_hash_invalid")
if veto_window_open:
    reason_codes.append("veto_window_open")
if veto_recorded:
    reason_codes.append("veto_recorded")
if not timelock_expired:
    reason_codes.append("timelock_not_expired")
if not approval_quorum_met:
    reason_codes.append("approval_quorum_missing")
if ci_fast_gate != "PASS":
    reason_codes.append("ci_fast_gate_failed")

payload = {
    "schema_version": "kamn.governance.simulation-veto.v1",
    "generated_at": generated_at,
    "proposal_id": proposal_id,
    "simulation_hash": simulation_hash,
    "simulation_complete": simulation_complete,
    "veto_window_open": veto_window_open,
    "veto_recorded": veto_recorded,
    "timelock_expired": timelock_expired,
    "approvals": {
        "required": required_approvals,
        "received": received_approvals,
    },
    "ci_fast_gate": ci_fast_gate,
    "policy_checks": {
        "simulation_hash_valid": hash_valid,
        "approval_quorum_met": approval_quorum_met,
    },
    "reason_codes": reason_codes,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"

