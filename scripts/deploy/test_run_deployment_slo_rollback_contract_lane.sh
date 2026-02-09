#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/deploy/run_deployment_slo_rollback_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected deployment slo/rollback contract lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/deployment-slo-rollback-contract-report.json"
output="$(
  KAMN_DEPLOYMENT_SLO_ROLLBACK_CONTRACT_MAX_SECONDS=240 \
  bash "$SCRIPT" --output-file "$report_file"
)"

if ! printf '%s\n' "$output" | grep -q 'deployment slo/rollback contract lane tests passed.'; then
  echo "expected success output from deployment slo/rollback contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected deployment slo/rollback contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.deploy.slo-rollback-report.v1"' "$report_file"; then
  echo "expected deployment slo/rollback report schema marker in contract lane output" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$report_file"; then
  echo "expected GO final decision in deployment slo/rollback contract lane report" >&2
  exit 1
fi

if ! grep -q 'check_deployment_slo_rollback_policy.sh' "$SCRIPT"; then
  echo "expected deployment slo/rollback contract lane to execute policy checker" >&2
  exit 1
fi

echo "deployment slo/rollback contract lane script tests passed."
