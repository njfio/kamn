#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_policy_contract_lane.sh"

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected signer policy contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "functional_privileged_roles_deny_fallback_when_provider_unavailable" "$POLICY_SCRIPT"; then
  echo "expected signer policy contract lane to include privileged-role fallback denial functional coverage" >&2
  exit 1
fi

if ! grep -q "router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes" "$POLICY_SCRIPT"; then
  echo "expected signer policy contract lane to include handshake decision-matrix unit coverage" >&2
  exit 1
fi

if ! grep -q "regression_provider_client_backend_mismatch_is_rejected_without_fallback" "$POLICY_SCRIPT"; then
  echo "expected signer policy contract lane to include provider-client mismatch regression coverage" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_SCRIPT")"
if ! printf '%s\n' "$policy_output" | grep -q "signer policy contract lane tests passed."; then
  echo "expected signer policy contract lane success marker" >&2
  exit 1
fi

echo "signer policy contract lane script tests passed."
