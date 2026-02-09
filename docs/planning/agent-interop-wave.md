# Agent Identity and Protocol Interop Wave (Issue #887)

This plan tracks deterministic DID lifecycle and protocol interop contract lanes required for production-grade identity safety.

## DID Lifecycle Mutation Contract Lane (Issue #889)
- Lifecycle mutation API contract:
  - `DidLifecycleMutationRequest`
  - `DidLifecycleMutationAction::{Rotate, Revoke, Recover}`
  - `DidLifecycleMutationEvidence`
- Fast lane commands:
  - mutation suite label: `did_lifecycle_mutation_transactions`
  - `cargo test -p kamn-core --test did_registry_transactions -- functional_lifecycle_rotate_mutation_updates_document_and_emits_allowed_reason_code -- --exact`
  - `cargo test -p kamn-core --test did_registry_transactions -- integration_lifecycle_revoke_then_recover_restores_active_resolution -- --exact`
  - `cargo test -p kamn-core --test did_registry_transactions -- regression_lifecycle_replayed_or_unauthorized_mutation_fails_closed -- --exact`
  - `cargo test -p kamn-core --test did_registry_transactions -- performance_lifecycle_mutation_contract_lane_stays_within_budget -- --exact`
  - `bash scripts/did/run_did_registry_contract_lane.sh`
- Deterministic lifecycle mutation reason codes:
  - `did_lifecycle_mutation_allowed`
  - `did_lifecycle_mutation_nonce_invalid`
  - `did_lifecycle_mutation_nonce_replay`
  - `did_lifecycle_mutation_unauthorized_actor`
  - `did_lifecycle_mutation_invalid_transition`
- Contract-lane decision key:
  - `did_lifecycle_mutation_reason_codes:GO:v1`

Fail-closed regression marker:
- replayed nonce, unauthorized actor mutation, and invalid revoked/active lifecycle transitions are rejected (`Regression: #889`).

## CI Routing Contract
- DID lifecycle-related docs and script changes must remain on bounded DID contract scope:
  - `docs/foundation/did-registry-transactions.md`
  - `docs/planning/agent-interop-wave.md`
  - `scripts/did/run_did_registry_contract_lane.sh`
  - `scripts/did/test_run_did_registry_contract_lane.sh`
