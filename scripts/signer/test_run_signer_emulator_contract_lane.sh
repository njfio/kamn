#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_emulator_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_provider_deep_lane.sh"
DEEP_IMPL_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_provider_deep_lane_impl.sh"
POLICY_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_policy_contract_lane.sh"
LIFECYCLE_SCRIPT="$ROOT_DIR/scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh"
INCIDENT_CONTRACT_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_contract_lane.sh"
INCIDENT_DEEP_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/signer/signer_emulator_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/signer_signer_emulator_contract_lane.json"
DEEP_MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/signer_signer_provider_deep_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected signer emulator fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected signer provider deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$DEEP_IMPL_SCRIPT" ]; then
  echo "expected signer provider deep-lane implementation to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected signer policy contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LIFECYCLE_SCRIPT" ]; then
  echo "expected signer secure-provider key-lifecycle contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$INCIDENT_CONTRACT_SCRIPT" ]; then
  echo "expected signer incident recovery contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$INCIDENT_DEEP_SCRIPT" ]; then
  echo "expected signer incident recovery deep lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected signer emulator shared contract-lane module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "signer emulator contract lane tests passed." "$TMP_OUT"; then
  echo "expected signer emulator contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected signer emulator contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected signer emulator contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected signer emulator wrapper to resolve signer emulator manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "signer_emulator_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected signer emulator manifest to dispatch shared contract module" >&2
  exit 1
fi

if [ ! -L "$DEEP_SCRIPT" ]; then
  echo "expected signer provider deep-lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$DEEP_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected signer provider deep-lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_MANIFEST_FILE" ]; then
  echo "expected signer provider deep-lane wrapper to resolve signer provider deep manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "run_signer_provider_deep_lane_impl.sh" "$DEEP_MANIFEST_FILE"; then
  echo "expected signer provider deep-lane manifest to dispatch implementation module" >&2
  exit 1
fi

if ! grep -q "functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to include provider handshake fallback functional coverage" >&2
  exit 1
fi

if ! grep -q "functional_router_uses_custom_provider_client_mapping_for_secure_provider" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to include provider client mapping functional coverage" >&2
  exit 1
fi

if ! grep -q "regression_provider_handshake_policy_block_rejects_without_fallback" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to include provider handshake policy-block regression coverage" >&2
  exit 1
fi

if ! grep -q "regression_provider_client_backend_mismatch_is_rejected_without_fallback" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to include provider client backend mismatch regression coverage" >&2
  exit 1
fi

if ! grep -q "integration_signature_profile_fixture_matrix_remains_consistent_with_transaction_guards" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to include signature profile compatibility matrix integration coverage" >&2
  exit 1
fi

if ! grep -q "run_signer_policy_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to invoke signer policy contract lane" >&2
  exit 1
fi

if ! grep -q "run_secure_provider_key_lifecycle_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to invoke secure-provider key-lifecycle contract lane" >&2
  exit 1
fi

if ! grep -q "run_signer_incident_recovery_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected signer emulator shared contract module to invoke signer incident recovery contract lane" >&2
  exit 1
fi

if ! grep -Fq "performance_signer_emulator_bulk_signing_deep_lane -- --ignored" "$DEEP_IMPL_SCRIPT"; then
  echo "expected deep-lane script to execute ignored signer provider stress test" >&2
  exit 1
fi

if ! grep -Fq "run_signer_incident_recovery_deep_lane.sh" "$DEEP_IMPL_SCRIPT"; then
  echo "expected signer provider deep lane script to execute signer incident recovery deep lane" >&2
  exit 1
fi

if ! grep -Fq "KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled" "$DEEP_IMPL_SCRIPT"; then
  echo "expected signer provider deep lane script to set scheduled cadence guard for incident recovery deep lane" >&2
  exit 1
fi

echo "signer emulator contract lane script tests passed."
