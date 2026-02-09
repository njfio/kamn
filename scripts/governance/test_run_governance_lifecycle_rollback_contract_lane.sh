#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_SCRIPT="$ROOT_DIR/scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_governance_lifecycle_rollback_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_governance_lifecycle_rollback_policy.sh"

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

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$CONTRACT_SCRIPT" >"$tmp_out"
if ! grep -q "governance lifecycle/rollback contract lane tests passed." "$tmp_out"; then
  echo "expected governance lifecycle/rollback contract lane success marker" >&2
  exit 1
fi

if ! grep -q "KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_CONTRACT_MAX_SECONDS" "$CONTRACT_SCRIPT"; then
  echo "expected governance lifecycle/rollback contract lane runtime guard env marker" >&2
  exit 1
fi
if ! grep -q "KAMN_GOVERNANCE_LIFECYCLE_FORCE_DOCS_CONTRACT_MISSING" "$CONTRACT_SCRIPT"; then
  echo "expected governance lifecycle/rollback contract lane forced docs-drift path" >&2
  exit 1
fi
if ! grep -q "reason_key mismatch" "$CONTRACT_SCRIPT"; then
  echo "expected governance lifecycle/rollback contract lane to enforce reason_key drift failures" >&2
  exit 1
fi

echo "governance lifecycle/rollback contract lane script tests passed."
