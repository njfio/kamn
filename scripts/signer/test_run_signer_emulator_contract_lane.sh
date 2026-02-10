#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_emulator_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_provider_deep_lane.sh"
POLICY_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_policy_contract_lane.sh"
LIFECYCLE_SCRIPT="$ROOT_DIR/scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh"
INCIDENT_CONTRACT_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_contract_lane.sh"
INCIDENT_DEEP_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected signer emulator fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected signer provider deep-lane runner to be executable" >&2
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

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "signer emulator contract lane tests passed." "$TMP_OUT"; then
  echo "expected signer emulator contract lane success marker" >&2
  exit 1
fi

if ! grep -q "functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to include provider handshake fallback functional coverage" >&2
  exit 1
fi

if ! grep -q "functional_router_uses_custom_provider_client_mapping_for_secure_provider" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to include provider client mapping functional coverage" >&2
  exit 1
fi

if ! grep -q "regression_provider_handshake_policy_block_rejects_without_fallback" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to include provider handshake policy-block regression coverage" >&2
  exit 1
fi

if ! grep -q "regression_provider_client_backend_mismatch_is_rejected_without_fallback" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to include provider client backend mismatch regression coverage" >&2
  exit 1
fi

if ! grep -q "integration_signature_profile_fixture_matrix_remains_consistent_with_transaction_guards" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to include signature profile compatibility matrix integration coverage" >&2
  exit 1
fi

if ! grep -q "run_signer_policy_contract_lane.sh" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to invoke signer policy contract lane" >&2
  exit 1
fi

if ! grep -q "run_secure_provider_key_lifecycle_contract_lane.sh" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to invoke secure-provider key-lifecycle contract lane" >&2
  exit 1
fi

if ! grep -q "run_signer_incident_recovery_contract_lane.sh" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to invoke signer incident recovery contract lane" >&2
  exit 1
fi

if ! grep -Fq "performance_signer_emulator_bulk_signing_deep_lane -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute ignored signer provider stress test" >&2
  exit 1
fi

if ! grep -Fq "run_signer_incident_recovery_deep_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected signer provider deep lane script to execute signer incident recovery deep lane" >&2
  exit 1
fi

if ! grep -Fq "KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled" "$DEEP_SCRIPT"; then
  echo "expected signer provider deep lane script to set scheduled cadence guard for incident recovery deep lane" >&2
  exit 1
fi

echo "signer emulator contract lane script tests passed."
