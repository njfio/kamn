#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/deploy/run_deployment_slo_rollback_lane.sh \
    --output-json <path>
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_dr_evidence_bundle.sh"
SLO_CHECKER="$ROOT_DIR/scripts/deploy/check_release_slo_gates.sh"
RUNBOOK_DOC="$ROOT_DIR/docs/foundation/upgrade-rollback-runbook.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
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

if [[ -z "$output_json" ]]; then
  usage
  fail "--output-json is required"
fi

if [ ! -x "$GENERATOR" ]; then
  fail "expected deployment DR evidence generator to be executable"
fi

if [ ! -x "$SLO_CHECKER" ]; then
  fail "expected deployment SLO gate checker to be executable"
fi

if [ ! -f "$RUNBOOK_DOC" ]; then
  fail "expected upgrade rollback runbook doc to exist"
fi

max_seconds="${KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS:-180}"
skip_commands="${KAMN_DEPLOYMENT_SLO_ROLLBACK_SKIP_COMMANDS:-false}"
force_rollback_automation_missing="${KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_ROLLBACK_AUTOMATION_MISSING:-false}"
force_slo_gate_missing="${KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_SLO_GATE_MISSING:-false}"
force_docs_contract_missing="${KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_DOCS_CONTRACT_MISSING:-false}"
force_lane_failure="${KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_LANE_FAILURE:-false}"

if [[ ! "$max_seconds" =~ ^[0-9]+$ ]]; then
  fail "KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS must be a non-negative integer"
fi

for value_name in \
  skip_commands \
  force_rollback_automation_missing \
  force_slo_gate_missing \
  force_docs_contract_missing \
  force_lane_failure; do
  value="${!value_name}"
  if [[ "$value" != "true" && "$value" != "false" ]]; then
    fail "invalid boolean for ${value_name}: ${value}"
  fi
done

mkdir -p "$(dirname "$output_json")"
start_epoch="$(date +%s)"

commands=()
bundle_file="$TMP_DIR/deployment-slo-rollback-dr-evidence.json"
generator_output=""
policy_output=""
generator_exit_code=0
policy_exit_code=0
deployment_lane_passed=true

demo_drill_id="dr-rollback-contract-2026-02-09"
recovery_rto_seconds=240
recovery_rpo_seconds=90
max_rto_seconds=300
max_rpo_seconds=120
rollback_restored=true
evidence_complete=true
ci_fast_gate=PASS

if [[ "$force_lane_failure" == "true" ]]; then
  deployment_lane_passed=false
  generator_exit_code=1
  policy_exit_code=1
elif [[ "$skip_commands" != "true" ]]; then
  commands+=("bash scripts/deploy/generate_dr_evidence_bundle.sh --output-file ${bundle_file} --drill-id ${demo_drill_id} --recovery-rto-seconds ${recovery_rto_seconds} --recovery-rpo-seconds ${recovery_rpo_seconds} --max-rto-seconds ${max_rto_seconds} --max-rpo-seconds ${max_rpo_seconds} --rollback-restored ${rollback_restored} --evidence-complete ${evidence_complete} --ci-fast-gate ${ci_fast_gate}")
  set +e
  generator_output="$(
    bash "$GENERATOR" \
      --output-file "$bundle_file" \
      --drill-id "$demo_drill_id" \
      --recovery-rto-seconds "$recovery_rto_seconds" \
      --recovery-rpo-seconds "$recovery_rpo_seconds" \
      --max-rto-seconds "$max_rto_seconds" \
      --max-rpo-seconds "$max_rpo_seconds" \
      --rollback-restored "$rollback_restored" \
      --evidence-complete "$evidence_complete" \
      --ci-fast-gate "$ci_fast_gate" 2>&1
  )"
  generator_exit_code=$?
  set -e

  if [ "$generator_exit_code" -eq 0 ]; then
    commands+=("bash scripts/deploy/check_release_slo_gates.sh --bundle-file ${bundle_file}")
    set +e
    policy_output="$(bash "$SLO_CHECKER" --bundle-file "$bundle_file" 2>&1)"
    policy_exit_code=$?
    set -e
  else
    policy_exit_code=1
  fi

  if [ "$generator_exit_code" -ne 0 ] || [ "$policy_exit_code" -ne 0 ]; then
    deployment_lane_passed=false
  fi
fi

dr_bundle_final_decision="unknown"
if [[ "$skip_commands" != "true" && -n "$generator_output" ]]; then
  maybe_dr_decision="$(extract_value "$generator_output" "final_decision")"
  if [[ -n "$maybe_dr_decision" ]]; then
    dr_bundle_final_decision="$maybe_dr_decision"
  fi
fi

policy_final_decision="unknown"
if [[ "$skip_commands" != "true" && -n "$policy_output" ]]; then
  maybe_policy_decision="$(extract_value "$policy_output" "final_decision")"
  if [[ -n "$maybe_policy_decision" ]]; then
    policy_final_decision="$maybe_policy_decision"
  fi
fi

slo_gate_passed=true
if [[ "$skip_commands" != "true" ]]; then
  if [ "$deployment_lane_passed" != "true" ] || [ "$policy_final_decision" != "GO" ]; then
    slo_gate_passed=false
  fi
fi

rollback_automation_passed=true
if [[ "$skip_commands" != "true" ]]; then
  if [ "$deployment_lane_passed" != "true" ]; then
    rollback_automation_passed=false
  else
    set +e
    python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
dr = payload.get("dr_evidence", {})
if dr.get("rollback_restored") is not True:
    raise SystemExit(1)
if dr.get("evidence_complete") is not True:
    raise SystemExit(1)
PY
    rollback_status=$?
    set -e
    if [ "$rollback_status" -ne 0 ]; then
      rollback_automation_passed=false
    fi
  fi
fi

if [[ "$force_rollback_automation_missing" == "true" ]]; then
  rollback_automation_passed=false
fi

if [[ "$force_slo_gate_missing" == "true" ]]; then
  slo_gate_passed=false
fi

docs_contract_passed=true
required_doc_snippets=(
  "## Deployment SLO Evidence and Rollback Automation Contract"
  "run_deployment_slo_rollback_lane.sh"
  "check_deployment_slo_rollback_policy.sh"
  "run_deployment_slo_rollback_contract_lane.sh"
  "kamn.deploy.slo-rollback-report.v1"
  "KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS"
  "KAMN_DEPLOYMENT_SLO_ROLLBACK_CONTRACT_MAX_SECONDS"
  "Regression: #944"
)

for snippet in "${required_doc_snippets[@]}"; do
  if ! grep -Fq "$snippet" "$RUNBOOK_DOC"; then
    docs_contract_passed=false
    break
  fi
done

if [[ "$force_docs_contract_missing" == "true" ]]; then
  docs_contract_passed=false
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"

reason_codes=()
if [ "$deployment_lane_passed" != "true" ]; then
  reason_codes+=("deployment_lane_failed")
fi
if [ "$slo_gate_passed" != "true" ]; then
  reason_codes+=("slo_gate_missing")
fi
if [ "$rollback_automation_passed" != "true" ]; then
  reason_codes+=("rollback_automation_missing")
fi
if [ "$docs_contract_passed" != "true" ]; then
  reason_codes+=("docs_contract_missing")
fi
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  reason_codes+=("runtime_budget_exceeded")
fi

if [ "${#reason_codes[@]}" -gt 0 ]; then
  mapfile -t reason_codes < <(printf '%s\n' "${reason_codes[@]}" | sort -u)
fi

status="pass"
final_decision="GO"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  status="fail"
  final_decision="NO-GO"
fi
reason_key="deployment_slo_rollback_reason_codes:${final_decision}:v1"

reason_codes_csv="none"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  reason_codes_csv="$(printf '%s,' "${reason_codes[@]}" | sed 's/,$//')"
fi

python3 - "$output_json" "$status" "$final_decision" "$reason_key" "$elapsed_seconds" "$max_seconds" "$skip_commands" "$bundle_file" "$generator_exit_code" "$policy_exit_code" "$dr_bundle_final_decision" "$policy_final_decision" "$deployment_lane_passed" "$slo_gate_passed" "$rollback_automation_passed" "$docs_contract_passed" "$reason_codes_csv" "${commands[@]}" <<'PY'
import json
import pathlib
import sys

output_file = pathlib.Path(sys.argv[1])
status = sys.argv[2]
final_decision = sys.argv[3]
reason_key = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
skip_commands = sys.argv[7] == "true"
bundle_file = sys.argv[8]
generator_exit_code = int(sys.argv[9])
policy_exit_code = int(sys.argv[10])
dr_bundle_final_decision = sys.argv[11]
policy_final_decision = sys.argv[12]
deployment_lane_passed = sys.argv[13] == "true"
slo_gate_passed = sys.argv[14] == "true"
rollback_automation_passed = sys.argv[15] == "true"
docs_contract_passed = sys.argv[16] == "true"
reason_codes_csv = sys.argv[17]
commands = sys.argv[18:]

payload = {
    "schema_version": "kamn.deploy.slo-rollback-report.v1",
    "evidence_key": "deployment_slo_rollback:v1",
    "status": status,
    "final_decision": final_decision,
    "reason_key": reason_key,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "skip_commands": skip_commands,
    "dr_bundle_file": bundle_file,
    "generator_exit_code": generator_exit_code,
    "policy_exit_code": policy_exit_code,
    "dr_bundle_final_decision": dr_bundle_final_decision,
    "policy_final_decision": policy_final_decision,
    "deployment_lane_passed": deployment_lane_passed,
    "slo_gate_passed": slo_gate_passed,
    "rollback_automation_passed": rollback_automation_passed,
    "docs_contract_passed": docs_contract_passed,
    "command_count": len(commands),
    "commands": commands,
    "reason_codes": [] if reason_codes_csv == "none" else reason_codes_csv.split(","),
}
output_file.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

printf 'status=%s\n' "$status"
printf 'final_decision=%s\n' "$final_decision"
printf 'elapsed_seconds=%s\n' "$elapsed_seconds"
printf 'reason_codes=%s\n' "$reason_codes_csv"
printf 'reason_key=%s\n' "$reason_key"
printf 'report_file=%s\n' "$output_json"

if [ "$status" != "pass" ]; then
  fail "deployment slo/rollback lane failed closed: ${reason_codes_csv}"
fi

echo "deployment slo/rollback lane tests passed."
