#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/deploy/run_deployment_slo_rollback_contract_lane.sh \
    [--output-file <path>]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/deploy/run_deployment_slo_rollback_lane.sh"
CHECKER="$ROOT_DIR/scripts/deploy/check_deployment_slo_rollback_policy.sh"
RUNBOOK_DOC="$ROOT_DIR/docs/foundation/upgrade-rollback-runbook.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

output_file=""

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

if [ ! -x "$LANE_SCRIPT" ]; then
  fail "expected deployment slo/rollback lane script to be executable"
fi

if [ ! -x "$CHECKER" ]; then
  fail "expected deployment slo/rollback policy checker to be executable"
fi

if [ ! -f "$RUNBOOK_DOC" ]; then
  fail "expected upgrade rollback runbook doc to exist"
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/deployment-slo-rollback-contract-report.json"
fi

max_contract_seconds="${KAMN_DEPLOYMENT_SLO_ROLLBACK_CONTRACT_MAX_SECONDS:-240}"
if [[ ! "$max_contract_seconds" =~ ^[1-9][0-9]*$ ]]; then
  fail "KAMN_DEPLOYMENT_SLO_ROLLBACK_CONTRACT_MAX_SECONDS must be a positive integer"
fi

start_epoch="$(date +%s)"

go_output="$(
  KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS="$max_contract_seconds" \
  bash "$LANE_SCRIPT" --output-json "$output_file"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  fail "expected deployment slo/rollback lane GO run to report pass status"
fi
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  fail "expected deployment slo/rollback lane GO run to report GO decision"
fi
if ! printf '%s\n' "$go_output" | grep -q '^reason_key=deployment_slo_rollback_reason_codes:GO:v1$'; then
  fail "expected deployment slo/rollback lane GO run to emit deterministic GO reason key"
fi

go_policy_output="$(bash "$CHECKER" --report-file "$output_file")"
if ! printf '%s\n' "$go_policy_output" | grep -q '^status=ok$'; then
  fail "expected deployment slo/rollback policy checker status marker for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^final_decision=GO$'; then
  fail "expected deployment slo/rollback policy checker GO decision for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^failed_checks=none$'; then
  fail "expected deployment slo/rollback policy checker no failed checks for GO report"
fi

rollback_no_go_report="$TMP_DIR/deployment-slo-rollback-no-go.json"
set +e
rollback_no_go_output="$(
  KAMN_DEPLOYMENT_SLO_ROLLBACK_SKIP_COMMANDS=true \
  KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_ROLLBACK_AUTOMATION_MISSING=true \
  bash "$LANE_SCRIPT" --output-json "$rollback_no_go_report" 2>&1
)"
rollback_no_go_code=$?
set -e

if [ "$rollback_no_go_code" -eq 0 ]; then
  fail "expected forced rollback automation missing deployment slo/rollback lane run to fail closed"
fi

if ! printf '%s\n' "$rollback_no_go_output" | grep -q 'rollback_automation_missing'; then
  fail "expected forced rollback automation missing lane run to emit rollback_automation_missing reason code"
fi

rollback_no_go_policy_output="$(bash "$CHECKER" --report-file "$rollback_no_go_report")"
if ! printf '%s\n' "$rollback_no_go_policy_output" | grep -q '^final_decision=NO-GO$'; then
  fail "expected deployment slo/rollback policy checker NO-GO decision for rollback-missing report"
fi
if ! printf '%s\n' "$rollback_no_go_policy_output" | grep -q 'rollback_automation_missing'; then
  fail "expected deployment slo/rollback policy checker failed checks to include rollback_automation_missing"
fi

if ! grep -q 'run_deployment_slo_rollback_lane.sh' "$RUNBOOK_DOC"; then
  fail "expected upgrade rollback runbook to reference deployment slo/rollback lane command"
fi
if ! grep -q 'check_deployment_slo_rollback_policy.sh' "$RUNBOOK_DOC"; then
  fail "expected upgrade rollback runbook to reference deployment slo/rollback policy checker command"
fi
if ! grep -q 'run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_deployment_slo_rollback_contract_lane.json --phase contract' "$RUNBOOK_DOC"; then
  fail "expected upgrade rollback runbook to reference manifest-backed deployment slo/rollback contract lane command"
fi
if ! grep -q 'kamn.deploy.slo-rollback-report.v1' "$RUNBOOK_DOC"; then
  fail "expected upgrade rollback runbook to reference deployment slo/rollback schema marker"
fi
if ! grep -q 'Regression: #944' "$RUNBOOK_DOC"; then
  fail "expected upgrade rollback runbook to include Regression: #944 marker"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_contract_seconds" ]; then
  fail "deployment slo/rollback contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'report_file=%s\n' "$output_file"
printf 'final_decision=GO\n'
echo "deployment slo/rollback contract lane tests passed."
