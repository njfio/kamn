#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_policy_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/signer/signer_policy_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/signer_signer_policy_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected signer policy contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected signer policy shared contract-lane module to be executable" >&2
  exit 1
fi

if ! grep -q "functional_privileged_roles_deny_fallback_when_provider_unavailable" "$SHARED_CONTRACT"; then
  echo "expected signer policy shared contract module to include privileged-role fallback denial functional coverage" >&2
  exit 1
fi

if ! grep -q "router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes" "$SHARED_CONTRACT"; then
  echo "expected signer policy shared contract module to include handshake decision-matrix unit coverage" >&2
  exit 1
fi

if ! grep -q "regression_provider_client_backend_mismatch_is_rejected_without_fallback" "$SHARED_CONTRACT"; then
  echo "expected signer policy shared contract module to include provider-client mismatch regression coverage" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_SCRIPT")"
if ! printf '%s\n' "$policy_output" | grep -q "signer policy contract lane tests passed."; then
  echo "expected signer policy contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$POLICY_SCRIPT" ]; then
  echo "expected signer policy contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$POLICY_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected signer policy contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$POLICY_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected signer policy wrapper to resolve signer policy manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "signer_policy_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected signer policy manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "signer policy contract lane script tests passed."
