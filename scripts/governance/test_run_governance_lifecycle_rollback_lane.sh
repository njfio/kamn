#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_governance_lifecycle_rollback_lane.sh"
LANE_IMPL="$ROOT_DIR/scripts/governance/run_governance_lifecycle_rollback_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/governance_lifecycle_rollback_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected governance lifecycle/rollback lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_IMPL" ]; then
  echo "expected governance lifecycle/rollback lane implementation to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected governance lifecycle/rollback lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected governance lifecycle/rollback lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected governance lifecycle/rollback lane wrapper to resolve governance manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q 'run_governance_lifecycle_rollback_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected governance lifecycle/rollback lane manifest to dispatch implementation module" >&2
  exit 1
fi

go_report="$TMP_DIR/governance-lifecycle-rollback-go.json"
go_output="$(
  KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS=true \
    bash "$LANE_SCRIPT" --output-file "$go_report"
)"
if [ "$(extract_value "$go_output" "status")" != "ok" ]; then
  echo "expected governance lifecycle/rollback GO path status=ok" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "final_decision")" != "GO" ]; then
  echo "expected governance lifecycle/rollback GO path final_decision=GO" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "reason_key")" != "governance_lifecycle_rollback_reason_codes:GO:v1" ]; then
  echo "expected governance lifecycle/rollback GO path reason_key marker" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "reason_taxonomy_version")" != "kamn.governance.lifecycle-rollback-reason-taxonomy.v1" ]; then
  echo "expected governance lifecycle/rollback GO path reason taxonomy version marker" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "reason_taxonomy_codes_csv")" != "docs_contract_missing,governance_lifecycle_lane_failed,lifecycle_contract_missing,rollback_contract_missing,rollback_gate_progress_stalled,runbook_marker_parity_bypass_detected,runtime_budget_exceeded" ]; then
  echo "expected governance lifecycle/rollback GO path reason taxonomy codes marker" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.governance.lifecycle-rollback-report.v1"' "$go_report"; then
  echo "expected governance lifecycle/rollback report schema marker" >&2
  exit 1
fi

no_go_report="$TMP_DIR/governance-lifecycle-rollback-no-go.json"
no_go_output="$(
  KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS=true \
  KAMN_GOVERNANCE_LIFECYCLE_FORCE_ROLLBACK_MISSING=true \
    bash "$LANE_SCRIPT" --output-file "$no_go_report"
)"
if [ "$(extract_value "$no_go_output" "final_decision")" != "NO-GO" ]; then
  echo "expected governance lifecycle/rollback forced rollback-missing path final_decision=NO-GO" >&2
  exit 1
fi
if [ "$(extract_value "$no_go_output" "reason_key")" != "governance_lifecycle_rollback_reason_codes:NO-GO:v1" ]; then
  echo "expected governance lifecycle/rollback forced rollback-missing reason_key marker" >&2
  exit 1
fi
if ! grep -q '"rollback_contract_missing"' "$no_go_report"; then
  echo "expected governance lifecycle/rollback forced rollback-missing reason" >&2
  exit 1
fi

echo "governance lifecycle/rollback lane script tests passed."
