#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions retry_classification_is_deterministic_for_duplicate_submission
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions functional_chain_submission_adapter_returns_typed_submitted_outcome
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions integration_chain_submission_adapter_deduplicates_retry_outcomes
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions regression_chain_submission_adapter_exposes_rejected_outcome_without_panicking
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions integration_register_retry_and_finality_boundary_is_idempotent
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions regression_register_finality_rejects_stale_or_conflicting_updates
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions unit_lifecycle_mutation_nonce_guards_emit_deterministic_reason_codes
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions functional_lifecycle_rotate_mutation_updates_document_and_emits_allowed_reason_code
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions integration_lifecycle_revoke_then_recover_restores_active_resolution
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions regression_lifecycle_replayed_or_unauthorized_mutation_fails_closed
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions performance_lifecycle_mutation_contract_lane_stays_within_budget
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions_docs
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test agent_interop_wave_docs
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test key_lifecycle_audit_trails_docs
bash scripts/did/run_lifecycle_operator_binding_contract_lane.sh --skip-tests
bash scripts/did/run_service_endpoint_canonicalization_contract_lane.sh --skip-tests
bash scripts/did/run_multikey_algorithm_policy_contract_lane.sh --skip-tests

if ! grep -Fq "register_with_retry_guard" docs/foundation/did-registry-transactions.md; then
  echo "expected retry guard contract in did-registry-transactions.md" >&2
  exit 1
fi

if ! grep -Fq "submit_registration_via_chain_adapter" docs/foundation/did-registry-transactions.md; then
  echo "expected chain adapter contract in did-registry-transactions.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #678" docs/foundation/did-registry-transactions.md; then
  echo "expected regression marker for DID finality guards in did-registry-transactions.md" >&2
  exit 1
fi

if ! grep -Fq "apply_lifecycle_mutation" docs/foundation/did-registry-transactions.md; then
  echo "expected lifecycle mutation contract in did-registry-transactions.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #889" docs/foundation/did-registry-transactions.md; then
  echo "expected lifecycle mutation regression marker in did-registry-transactions.md" >&2
  exit 1
fi

if ! grep -Fq "did_lifecycle_mutation_reason_codes:GO:v1" docs/planning/agent-interop-wave.md; then
  echo "expected lifecycle mutation reason-key contract in agent-interop-wave planning doc" >&2
  exit 1
fi

if ! grep -Fq "did_lifecycle_operator_binding_reason_codes:GO:v1" docs/planning/agent-interop-wave.md; then
  echo "expected lifecycle operator-binding reason-key contract in agent-interop-wave planning doc" >&2
  exit 1
fi

if ! grep -Fq "Regression: #890" docs/planning/agent-interop-wave.md; then
  echo "expected lifecycle operator-binding regression marker in agent-interop-wave planning doc" >&2
  exit 1
fi

if ! grep -Fq "run_service_endpoint_canonicalization_contract_lane.sh" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected service endpoint canonicalization contract lane command in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "run_service_endpoint_canonicalization_matrix.py" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected service endpoint canonicalization matrix runner command in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "did_service_endpoint_canonicalization_reason_codes:GO:v1" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected service endpoint canonicalization GO reason-key marker in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "Regression: #1000" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected service endpoint canonicalization regression marker in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "run_multikey_algorithm_policy_contract_lane.sh" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected multikey algorithm policy contract lane command in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "run_multikey_algorithm_migration_matrix.py" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected multikey algorithm migration matrix runner command in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "did_multikey_algorithm_policy_reason_codes:GO:v1" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected multikey algorithm policy GO reason-key marker in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "Regression: #1001" docs/foundation/did-core-conformance-kamn-method.md; then
  echo "expected multikey algorithm policy regression marker in did core conformance doc" >&2
  exit 1
fi

if ! grep -Fq "generate_lifecycle_operator_binding_evidence_bundle.sh" docs/foundation/key-lifecycle-audit-trails.md; then
  echo "expected lifecycle operator-binding evidence generator command in key-lifecycle-audit-trails.md" >&2
  exit 1
fi

if ! grep -Fq "check_lifecycle_operator_binding_policy.sh" docs/foundation/key-lifecycle-audit-trails.md; then
  echo "expected lifecycle operator-binding policy checker command in key-lifecycle-audit-trails.md" >&2
  exit 1
fi

if ! grep -Fq "run_lifecycle_operator_binding_contract_lane.sh" docs/foundation/key-lifecycle-audit-trails.md; then
  echo "expected lifecycle operator-binding contract lane command in key-lifecycle-audit-trails.md" >&2
  exit 1
fi

if ! grep -Fq "did_lifecycle_operator_binding_reason_codes:GO:v1" docs/foundation/key-lifecycle-audit-trails.md; then
  echo "expected lifecycle operator-binding reason-key contract in key-lifecycle-audit-trails.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #890" docs/foundation/key-lifecycle-audit-trails.md; then
  echo "expected lifecycle operator-binding regression marker in key-lifecycle-audit-trails.md" >&2
  exit 1
fi

echo "did registry contract lane tests passed."
