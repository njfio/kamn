#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

signer_policy_target_dir="${KAMN_SIGNER_POLICY_CONTRACT_TARGET_DIR:-$ROOT_DIR/target/signer-policy-contract}"
mkdir -p "$signer_policy_target_dir"

run_signer_policy_cargo_test() {
  CARGO_TARGET_DIR="$signer_policy_target_dir" \
    bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test "$@"
}

run_signer_policy_cargo_test -p kamn-core --test signer_backend functional_privileged_roles_deny_fallback_when_provider_unavailable -- --exact --nocapture
run_signer_policy_cargo_test -p kamn-core --lib signer_backend::tests::policy_tests::router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes -- --exact
run_signer_policy_cargo_test -p kamn-core --test signer_backend regression_provider_handshake_policy_block_rejects_without_fallback -- --exact --nocapture
run_signer_policy_cargo_test -p kamn-core --test signer_backend regression_provider_client_backend_mismatch_is_rejected_without_fallback -- --exact --nocapture
run_signer_policy_cargo_test -p kamn-core --test threat_control_matrix_docs -- --test-threads=1 --nocapture

if ! grep -Fq "run_signer_policy_contract_lane.sh" docs/foundation/signer-backend-abstraction.md; then
  echo "expected signer backend abstraction doc to reference signer policy contract lane" >&2
  exit 1
fi

if ! grep -Fq "TM-007" docs/foundation/threat-control-matrix.md; then
  echo "expected threat control matrix to include signer fallback/handshake row" >&2
  exit 1
fi

if ! grep -Fq "Regression: #987" docs/foundation/threat-control-matrix.md; then
  echo "expected threat control matrix to include signer policy regression marker" >&2
  exit 1
fi

echo "signer policy contract lane tests passed."
