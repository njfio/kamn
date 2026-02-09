#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_governance_lifecycle_rollback_lane.sh"
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
