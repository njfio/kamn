#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_SCRIPT="$ROOT_DIR/scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_governance_lifecycle_rollback_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_governance_lifecycle_rollback_policy.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/governance/governance_lifecycle_rollback_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/governance_lifecycle_rollback_contract_lane.json"

if [ ! -x "$CONTRACT_SCRIPT" ]; then
  echo "expected governance lifecycle/rollback contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected governance lifecycle/rollback lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected governance lifecycle/rollback policy checker script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected governance lifecycle/rollback shared contract module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected governance lifecycle/rollback manifest to exist" >&2
  exit 1
fi

tmp_out="$(mktemp)"
TMP_DIR="$(mktemp -d)"
trap 'rm -f "$tmp_out"; rm -rf "$TMP_DIR"' EXIT

bash "$CONTRACT_SCRIPT" >"$tmp_out"
if ! grep -q "governance lifecycle/rollback contract lane tests passed." "$tmp_out"; then
  echo "expected governance lifecycle/rollback contract lane success marker" >&2
  exit 1
fi

if ! grep -q "run_manifest_lane.sh" "$CONTRACT_SCRIPT"; then
  echo "expected governance lifecycle/rollback contract lane wrapper to dispatch via manifest runner" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected governance lifecycle/rollback contract lane wrapper to resolve lifecycle manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q '"wrapper_name": "run_governance_lifecycle_rollback_contract_lane.sh"' "$MANIFEST"; then
  echo "expected governance lifecycle/rollback manifest wrapper_name metadata marker" >&2
  exit 1
fi
if ! grep -q '"phase": "contract"' "$MANIFEST"; then
  echo "expected governance lifecycle/rollback manifest phase metadata marker" >&2
  exit 1
fi
if ! grep -q "governance_lifecycle_rollback_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected governance lifecycle/rollback manifest to dispatch to shared module" >&2
  exit 1
fi
if ! grep -q "KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_CONTRACT_MAX_SECONDS" "$SHARED_CONTRACT"; then
  echo "expected governance lifecycle/rollback contract lane runtime guard env marker" >&2
  exit 1
fi
if ! grep -q "KAMN_GOVERNANCE_LIFECYCLE_FORCE_DOCS_CONTRACT_MISSING" "$SHARED_CONTRACT"; then
  echo "expected governance lifecycle/rollback contract lane forced docs-drift path" >&2
  exit 1
fi
if ! grep -q "reason_key mismatch" "$SHARED_CONTRACT"; then
  echo "expected governance lifecycle/rollback contract lane to enforce reason_key drift failures" >&2
  exit 1
fi
if ! grep -q "ci-local promotion budget boundary exceeded" "$SHARED_CONTRACT"; then
  echo "expected governance lifecycle/rollback contract lane to enforce ci-local promotion budget boundary failures" >&2
  exit 1
fi
if ! grep -q "kamn.governance.lifecycle-rollback-reason-taxonomy.v1" "$SHARED_CONTRACT"; then
  echo "expected governance lifecycle/rollback contract lane to enforce deterministic rollback reason taxonomy marker" >&2
  exit 1
fi

rollback_gate_drift_report="$TMP_DIR/governance-lifecycle-rollback-gate-drift.json"
KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS=true \
KAMN_GOVERNANCE_LIFECYCLE_FORCE_LANE_FAILURE=true \
  bash "$LANE_SCRIPT" --output-file "$rollback_gate_drift_report" >/dev/null
if ! grep -q '"rollback_gate_progress_stalled"' "$rollback_gate_drift_report"; then
  echo "expected rollback_gate_progress_stalled reason marker in rollback gate drift report" >&2
  exit 1
fi

runbook_parity_drift_report="$TMP_DIR/governance-lifecycle-rollback-runbook-parity-drift.json"
KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS=true \
KAMN_GOVERNANCE_LIFECYCLE_FORCE_DOCS_CONTRACT_MISSING=true \
  bash "$LANE_SCRIPT" --output-file "$runbook_parity_drift_report" >/dev/null
if ! grep -q '"runbook_marker_parity_bypass_detected"' "$runbook_parity_drift_report"; then
  echo "expected runbook_marker_parity_bypass_detected reason marker in runbook parity drift report" >&2
  exit 1
fi

if ! grep -q "ci_local_promotion_budget_boundary_status=verified" "$tmp_out"; then
  echo "expected governance lifecycle/rollback contract lane ci-local promotion budget boundary marker" >&2
  exit 1
fi
if ! grep -q "reason_taxonomy_version=kamn.governance.lifecycle-rollback-reason-taxonomy.v1" "$tmp_out"; then
  echo "expected governance lifecycle/rollback contract lane reason taxonomy version marker" >&2
  exit 1
fi
if ! grep -q "reason_taxonomy_codes_csv=docs_contract_missing,governance_lifecycle_lane_failed,lifecycle_contract_missing,rollback_contract_missing,rollback_gate_progress_stalled,runbook_marker_parity_bypass_detected,runtime_budget_exceeded" "$tmp_out"; then
  echo "expected governance lifecycle/rollback contract lane reason taxonomy codes marker" >&2
  exit 1
fi

set +e
oversized_budget_output="$(
  KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_CONTRACT_MAX_SECONDS=241 \
    bash "$CONTRACT_SCRIPT" 2>&1
)"
oversized_budget_code=$?
set -e
if [ "$oversized_budget_code" -eq 0 ]; then
  echo "expected governance lifecycle/rollback contract lane to fail when ci-local promotion budget boundary is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$oversized_budget_output" | grep -q "ci-local promotion budget boundary exceeded"; then
  echo "expected deterministic ci-local promotion budget boundary rejection marker for governance lifecycle/rollback contract lane" >&2
  exit 1
fi

echo "governance lifecycle/rollback contract lane script tests passed."
