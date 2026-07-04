#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

signer_emulator_target_dir="${KAMN_SIGNER_EMULATOR_CONTRACT_TARGET_DIR:-$ROOT_DIR/target/signer-emulator-contract}"
mkdir -p "$signer_emulator_target_dir"

run_signer_emulator_cargo_test() {
  CARGO_TARGET_DIR="$signer_emulator_target_dir" \
    bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test "$@"
}

run_signer_emulator_cargo_test -p kamn-core --test signer_backend functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider -- --exact --nocapture
run_signer_emulator_cargo_test -p kamn-core --test signer_backend functional_router_uses_custom_provider_client_mapping_for_secure_provider -- --exact --nocapture
run_signer_emulator_cargo_test -p kamn-core --test signer_backend regression_provider_handshake_policy_block_rejects_without_fallback -- --exact --nocapture
run_signer_emulator_cargo_test -p kamn-core --test signer_backend regression_provider_client_backend_mismatch_is_rejected_without_fallback -- --exact --nocapture
run_signer_emulator_cargo_test -p kamn-core --test signer_backend integration_signature_profile_fixture_matrix_remains_consistent_with_transaction_guards -- --exact --nocapture
run_signer_emulator_cargo_test -p kamn-core --test signer_backend regression_secure_provider_backend_mismatch_is_rejected -- --exact --nocapture
run_signer_emulator_cargo_test -p kamn-core --test signer_backend performance_signer_emulator_contract_lane_stays_within_budget -- --exact --nocapture
run_signer_emulator_cargo_test -p kamn-core --test signer_backend_docs
bash scripts/signer/run_signer_policy_contract_lane.sh
bash scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh
bash scripts/signer/run_signer_incident_recovery_contract_lane.sh

if ! grep -Fq "## Signer Emulator Contract Lanes" docs/foundation/signer-backend-abstraction.md; then
  echo "expected signer emulator contract lane section in signer-backend-abstraction.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #619" docs/foundation/signer-backend-abstraction.md; then
  echo "expected regression marker for signer emulator contract lane in signer-backend-abstraction.md" >&2
  exit 1
fi

echo "signer emulator contract lane tests passed."
