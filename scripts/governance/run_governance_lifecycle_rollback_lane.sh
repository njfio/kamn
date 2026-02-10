#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GOVERNANCE_DOC="$ROOT_DIR/docs/foundation/governance-proposal-vote-execution.md"
ROLLBACK_DOC="$ROOT_DIR/docs/foundation/upgrade-rollback-runbook.md"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/run_governance_lifecycle_rollback_lane.sh \
    [--output-file <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file="$ROOT_DIR/governance-lifecycle-rollback-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
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

max_runtime_seconds="${KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_MAX_SECONDS:-180}"
if [[ ! "$max_runtime_seconds" =~ ^[0-9]+$ ]]; then
  fail "KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_MAX_SECONDS must be an integer >= 0"
fi

skip_commands="${KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS:-false}"
case "$skip_commands" in
  true|false) ;;
  *)
    fail "KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS must be true or false"
    ;;
esac

lifecycle_contract_present=true
rollback_contract_present=true
docs_contract_present=true
lane_failed=false
commands=()

if [[ ! -f "$ROOT_DIR/crates/kamn-core/tests/governance_workflow.rs" ]]; then
  lifecycle_contract_present=false
fi
if [[ ! -f "$ROOT_DIR/crates/kamn-core/tests/upgrade_orchestration.rs" ]]; then
  rollback_contract_present=false
fi

if [[ "${KAMN_GOVERNANCE_LIFECYCLE_FORCE_LIFECYCLE_MISSING:-false}" == "true" ]]; then
  lifecycle_contract_present=false
fi
if [[ "${KAMN_GOVERNANCE_LIFECYCLE_FORCE_ROLLBACK_MISSING:-false}" == "true" ]]; then
  rollback_contract_present=false
fi

start_epoch="$(date +%s)"

if [[ "$skip_commands" != "true" ]]; then
  if [[ "$lifecycle_contract_present" == "true" ]]; then
    commands+=("cargo test -p kamn-core --test governance_workflow governance_workflow_functional_submit_vote_execute_flow")
    if ! cargo test -p kamn-core --test governance_workflow governance_workflow_functional_submit_vote_execute_flow >/dev/null; then
      lane_failed=true
    fi
    commands+=("cargo test -p kamn-core --test governance_workflow governance_workflow_regression_rejects_late_votes_after_deadline")
    if ! cargo test -p kamn-core --test governance_workflow governance_workflow_regression_rejects_late_votes_after_deadline >/dev/null; then
      lane_failed=true
    fi
  fi

  if [[ "$rollback_contract_present" == "true" ]]; then
    commands+=("cargo test -p kamn-core --test upgrade_orchestration upgrade_orchestration_functional_activate_then_rollback_restores_version_and_audits_event")
    if ! cargo test -p kamn-core --test upgrade_orchestration upgrade_orchestration_functional_activate_then_rollback_restores_version_and_audits_event >/dev/null; then
      lane_failed=true
    fi
    commands+=("cargo test -p kamn-core --test upgrade_orchestration upgrade_orchestration_regression_rejects_rollback_before_activation")
    if ! cargo test -p kamn-core --test upgrade_orchestration upgrade_orchestration_regression_rejects_rollback_before_activation >/dev/null; then
      lane_failed=true
    fi
  fi
fi

if [[ "${KAMN_GOVERNANCE_LIFECYCLE_FORCE_LANE_FAILURE:-false}" == "true" ]]; then
  lane_failed=true
fi

required_doc_markers=(
  "governance_lifecycle_rollback_policy_contract.py"
  "run_governance_lifecycle_rollback_lane.sh"
  "check_governance_lifecycle_rollback_policy.sh"
  "run_governance_lifecycle_rollback_contract_lane.sh"
  "kamn.governance.lifecycle-rollback-report.v1"
  "governance_lifecycle_rollback_reason_codes:GO:v1"
  "governance_lifecycle_rollback_reason_codes:NO-GO:v1"
  'illegal lifecycle transitions and rollback integrity drift must fail closed (`Regression: #910`).'
)

for marker in "${required_doc_markers[@]}"; do
  if ! grep -Fq "$marker" "$GOVERNANCE_DOC"; then
    docs_contract_present=false
  fi
  if ! grep -Fq "$marker" "$ROLLBACK_DOC"; then
    docs_contract_present=false
  fi
done

if [[ "${KAMN_GOVERNANCE_LIFECYCLE_FORCE_DOCS_CONTRACT_MISSING:-false}" == "true" ]]; then
  docs_contract_present=false
fi

runtime_seconds="$(( $(date +%s) - start_epoch ))"
runtime_budget_ok=true
if [ "$runtime_seconds" -gt "$max_runtime_seconds" ]; then
  runtime_budget_ok=false
fi

decision_reasons=()
if [[ "$lane_failed" == "true" ]]; then
  decision_reasons+=("governance_lifecycle_lane_failed")
fi
if [[ "$lifecycle_contract_present" != "true" ]]; then
  decision_reasons+=("lifecycle_contract_missing")
fi
if [[ "$rollback_contract_present" != "true" ]]; then
  decision_reasons+=("rollback_contract_missing")
fi
if [[ "$docs_contract_present" != "true" ]]; then
  decision_reasons+=("docs_contract_missing")
fi
if [[ "$runtime_budget_ok" != "true" ]]; then
  decision_reasons+=("runtime_budget_exceeded")
fi

final_decision="GO"
if [ "${#decision_reasons[@]}" -gt 0 ]; then
  final_decision="NO-GO"
fi
reason_key="governance_lifecycle_rollback_reason_codes:${final_decision}:v1"

mkdir -p "$(dirname "$output_file")"
decision_reasons_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${decision_reasons[@]}")"
commands_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${commands[@]}")"

python3 - "$output_file" "$max_runtime_seconds" "$runtime_seconds" "$decision_reasons_json" "$commands_json" "$lane_failed" "$lifecycle_contract_present" "$rollback_contract_present" "$docs_contract_present" "$runtime_budget_ok" "$final_decision" "$reason_key" <<'PY'
import json
import pathlib
import sys
from datetime import datetime, timezone

output_file = pathlib.Path(sys.argv[1])
max_runtime_seconds = int(sys.argv[2])
runtime_seconds = int(sys.argv[3])
decision_reasons = json.loads(sys.argv[4])
commands = json.loads(sys.argv[5])
lane_failed = sys.argv[6] == "true"
lifecycle_contract_present = sys.argv[7] == "true"
rollback_contract_present = sys.argv[8] == "true"
docs_contract_present = sys.argv[9] == "true"
runtime_budget_ok = sys.argv[10] == "true"
final_decision = sys.argv[11]
reason_key = sys.argv[12]

payload = {
    "schema_version": "kamn.governance.lifecycle-rollback-report.v1",
    "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "max_runtime_seconds": max_runtime_seconds,
    "runtime_seconds": runtime_seconds,
    "checks": {
        "lane_failed": lane_failed,
        "lifecycle_contract_present": lifecycle_contract_present,
        "rollback_contract_present": rollback_contract_present,
        "docs_contract_present": docs_contract_present,
        "runtime_budget_ok": runtime_budget_ok,
    },
    "commands": commands,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
    "reason_key": reason_key,
}
output_file.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

printf 'status=ok\n'
printf 'output_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
printf 'reason_key=%s\n' "$reason_key"
printf 'runtime_seconds=%s\n' "$runtime_seconds"
printf 'max_runtime_seconds=%s\n' "$max_runtime_seconds"
