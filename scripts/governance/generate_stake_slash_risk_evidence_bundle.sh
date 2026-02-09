#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/generate_stake_slash_risk_evidence_bundle.sh \
    --output-file <path> \
    --proposal-id <value> \
    --simulation-hash sha256:<64-hex> \
    --stake-at-risk-bps <n> \
    --max-stake-at-risk-bps <n> \
    --slash-probability-bps <n> \
    --max-slash-probability-bps <n> \
    --validator-churn-bps <n> \
    --max-validator-churn-bps <n> \
    --quorum-safety-margin-bps <n> \
    --min-quorum-safety-margin-bps <n> \
    --evidence-complete true|false \
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
stake_at_risk_bps=""
max_stake_at_risk_bps=""
slash_probability_bps=""
max_slash_probability_bps=""
validator_churn_bps=""
max_validator_churn_bps=""
quorum_safety_margin_bps=""
min_quorum_safety_margin_bps=""
evidence_complete=""
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
    --stake-at-risk-bps)
      stake_at_risk_bps="${2:-}"
      shift 2
      ;;
    --max-stake-at-risk-bps)
      max_stake_at_risk_bps="${2:-}"
      shift 2
      ;;
    --slash-probability-bps)
      slash_probability_bps="${2:-}"
      shift 2
      ;;
    --max-slash-probability-bps)
      max_slash_probability_bps="${2:-}"
      shift 2
      ;;
    --validator-churn-bps)
      validator_churn_bps="${2:-}"
      shift 2
      ;;
    --max-validator-churn-bps)
      max_validator_churn_bps="${2:-}"
      shift 2
      ;;
    --quorum-safety-margin-bps)
      quorum_safety_margin_bps="${2:-}"
      shift 2
      ;;
    --min-quorum-safety-margin-bps)
      min_quorum_safety_margin_bps="${2:-}"
      shift 2
      ;;
    --evidence-complete)
      evidence_complete="$(normalize_bool "${2:-}")"
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

if [[ -z "$output_file" || -z "$proposal_id" || -z "$simulation_hash" || -z "$stake_at_risk_bps" || -z "$max_stake_at_risk_bps" || -z "$slash_probability_bps" || -z "$max_slash_probability_bps" || -z "$validator_churn_bps" || -z "$max_validator_churn_bps" || -z "$quorum_safety_margin_bps" || -z "$min_quorum_safety_margin_bps" || -z "$evidence_complete" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all bundle arguments are required"
fi

for value in \
  "$stake_at_risk_bps" \
  "$max_stake_at_risk_bps" \
  "$slash_probability_bps" \
  "$max_slash_probability_bps" \
  "$validator_churn_bps" \
  "$max_validator_churn_bps" \
  "$quorum_safety_margin_bps" \
  "$min_quorum_safety_margin_bps"; do
  require_int "bps field" "$value"
done

if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  fail "ci-fast-gate must be PASS or FAIL"
fi

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$proposal_id" "$simulation_hash" "$stake_at_risk_bps" "$max_stake_at_risk_bps" "$slash_probability_bps" "$max_slash_probability_bps" "$validator_churn_bps" "$max_validator_churn_bps" "$quorum_safety_margin_bps" "$min_quorum_safety_margin_bps" "$evidence_complete" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys

(
    output_file,
    generated_at,
    proposal_id,
    simulation_hash,
    stake_at_risk_bps_raw,
    max_stake_at_risk_bps_raw,
    slash_probability_bps_raw,
    max_slash_probability_bps_raw,
    validator_churn_bps_raw,
    max_validator_churn_bps_raw,
    quorum_safety_margin_bps_raw,
    min_quorum_safety_margin_bps_raw,
    evidence_complete_raw,
    ci_fast_gate,
) = sys.argv[1:]

stake_at_risk_bps = int(stake_at_risk_bps_raw)
max_stake_at_risk_bps = int(max_stake_at_risk_bps_raw)
slash_probability_bps = int(slash_probability_bps_raw)
max_slash_probability_bps = int(max_slash_probability_bps_raw)
validator_churn_bps = int(validator_churn_bps_raw)
max_validator_churn_bps = int(max_validator_churn_bps_raw)
quorum_safety_margin_bps = int(quorum_safety_margin_bps_raw)
min_quorum_safety_margin_bps = int(min_quorum_safety_margin_bps_raw)
evidence_complete = evidence_complete_raw == "true"

hash_valid = bool(re.match(r"^sha256:[0-9a-f]{64}$", simulation_hash))
stake_risk_within_limit = stake_at_risk_bps <= max_stake_at_risk_bps
slash_probability_within_limit = slash_probability_bps <= max_slash_probability_bps
validator_churn_within_limit = validator_churn_bps <= max_validator_churn_bps
quorum_margin_within_limit = quorum_safety_margin_bps >= min_quorum_safety_margin_bps

is_go = (
    hash_valid
    and stake_risk_within_limit
    and slash_probability_within_limit
    and validator_churn_within_limit
    and quorum_margin_within_limit
    and evidence_complete
    and ci_fast_gate == "PASS"
)
final_decision = "GO" if is_go else "NO-GO"

reason_codes = []
if not hash_valid:
    reason_codes.append("simulation_hash_invalid")
if not stake_risk_within_limit:
    reason_codes.append("stake_at_risk_threshold_breach")
if not slash_probability_within_limit:
    reason_codes.append("slash_probability_threshold_breach")
if not validator_churn_within_limit:
    reason_codes.append("validator_churn_threshold_breach")
if not quorum_margin_within_limit:
    reason_codes.append("quorum_safety_margin_breach")
if not evidence_complete:
    reason_codes.append("evidence_incomplete")
if ci_fast_gate != "PASS":
    reason_codes.append("ci_fast_gate_failed")

payload = {
    "schema_version": "kamn.governance.stake-slash-risk.v1",
    "generated_at": generated_at,
    "proposal_id": proposal_id,
    "simulation_hash": simulation_hash,
    "risk_metrics_bps": {
        "stake_at_risk": stake_at_risk_bps,
        "slash_probability": slash_probability_bps,
        "validator_churn": validator_churn_bps,
        "quorum_safety_margin": quorum_safety_margin_bps,
    },
    "risk_thresholds_bps": {
        "max_stake_at_risk": max_stake_at_risk_bps,
        "max_slash_probability": max_slash_probability_bps,
        "max_validator_churn": max_validator_churn_bps,
        "min_quorum_safety_margin": min_quorum_safety_margin_bps,
    },
    "evidence_complete": evidence_complete,
    "ci_fast_gate": ci_fast_gate,
    "policy_checks": {
        "simulation_hash_valid": hash_valid,
        "stake_risk_within_limit": stake_risk_within_limit,
        "slash_probability_within_limit": slash_probability_within_limit,
        "validator_churn_within_limit": validator_churn_within_limit,
        "quorum_margin_within_limit": quorum_margin_within_limit,
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

