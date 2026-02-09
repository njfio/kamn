#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/governance/generate_governance_simulation_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_governance_simulation_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BUNDLE_FILE="$TMP_DIR/governance-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --proposal-id "gov-proposal-contract-001" \
    --simulation-hash "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --simulation-complete true \
    --veto-window-open false \
    --veto-recorded false \
    --timelock-expired true \
    --required-approvals 2 \
    --received-approvals 2 \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected governance contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected governance contract lane policy decision to be GO" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  echo "expected governance contract lane to report no failed checks" >&2
  exit 1
fi

echo "governance simulation contract lane tests passed."

