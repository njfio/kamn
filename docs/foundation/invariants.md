# Invariant Catalog and Error Taxonomy (Issue #77)

This document defines the canonical transaction invariant catalog and error taxonomy used by baseline KAMN validation scaffolding.

## Catalog
| Invariant ID | Domain | Description | Failure Codes | PRD References | Owner |
|---|---|---|---|---|---|
| `INV-TX-001` | transactions | Transaction envelope fields must be present and non-empty | `INV-TX-001-EMPTY-FIELD` | `2.2.3`, `4.1` | #78 |
| `INV-TX-002` | transactions | Nonce must be positive and per-sender sequential | `INV-TX-002-INVALID-NONCE`, `INV-TX-002-NONCE-SEQUENCE` | `2.2.3`, `4.2` | #78 |
| `INV-TX-003` | transactions | Signature must match deterministic signing rules | `INV-TX-003-INVALID-SIGNATURE` | `1.3`, `2.2.2`, `4.1` | #78 |
| `INV-TX-004` | transactions | State hash must match the expected chain/app state hash | `INV-TX-004-STATE-HASH-MISMATCH` | `2.2.3`, `13.1` | #78 |
| `INV-TX-005` | transactions | Transaction IDs must be globally unique in observed history | `INV-TX-005-DUPLICATE-TX-ID` | `2.2.3`, `4.2` | #78 |
| `INV-TX-006` | state-transitions | Only validated transactions may be committed | `INV-TX-006-UNVALIDATED-COMMIT` | `2.2.3`, `13.1` | #78 |

## Mapping Conventions
- Guard-layer errors (`TransactionGuardError`) map deterministically to one invariant ID and one canonical failure code.
- Smoke-layer errors are classified into taxonomy only when the source is a guard violation.
- Non-guard operational errors (for example empty mempool) are intentionally out-of-taxonomy.

## Current Integration Surfaces
- `classify_transaction_guard_error(...)`
- `classify_smoke_error(...)`
- `validate_catalog(...)`

## Runtime Invariant Harness Coverage (Issue #897)
- Property-based lifecycle invariant lane:
  - `bash scripts/runtime/run_lifecycle_property_contract_lane.sh`
  - `bash scripts/runtime/run_lifecycle_property_contract_lane.sh --output-json /tmp/lifecycle-property-contract-report.json`
- Lifecycle property report schema:
  - `kamn.runtime.lifecycle-property-contract-report.v1`
- Lifecycle property replay artifact key:
  - `lifecycle_property_replay:v1`
- Fuzz/mutation fail-closed lane:
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh`
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh --output-json /tmp/input-mutation-contract-report.json`
- Input mutation report schema:
  - `kamn.runtime.input-mutation-contract-report.v1`
- Input mutation replay artifact key:
  - `input_mutation_replay:v1`
- ZK witness mutation fast lane:
  - `bash scripts/runtime/run_zk_witness_mutation_contract_lane.sh`
- ZK witness mutation deep lane (scheduled):
  - `bash scripts/runtime/run_zk_witness_mutation_deep_lane.sh`
  - route via `KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP=true` when running `run_input_mutation_contract_lane.sh`.
- Concurrency state-mutation lane:
  - `bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh`
- Combined bounded lane with evidence output:
  - `bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json`
- Combined lane policy checker:
  - `bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json`
- Report schema:
  - `kamn.runtime.invariant-fuzz-concurrency-contract-report.v1`
- Property lane runtime budget env:
  - `KAMN_RUNTIME_LIFECYCLE_PROPERTY_MAX_SECONDS` (default `120`)
- Input mutation lane runtime budget env:
  - `KAMN_RUNTIME_INPUT_MUTATION_MAX_SECONDS` (default `120`)

## Dispute/Refund Property and Concurrency Contracts (Issue #904)
- Property + replay trace contract lane:
  - `cargo test -p kamn-core --test dispute_refund_transition_contracts functional_property_dispute_refund_sequences_preserve_contracts -- --exact`
  - `cargo test -p kamn-core --test dispute_refund_transition_contracts integration_dispute_refund_replay_traces_are_deterministic -- --exact`
- Concurrency replay determinism contract lane:
  - `cargo test -p kamn-core --test concurrency_state_mutation functional_escrow_dispute_refund_concurrency_replay_fixture_preserves_terminal_snapshot -- --exact`
  - `cargo test -p kamn-core --test concurrency_state_mutation integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds -- --exact`
- Fast CI lane scripts:
  - `bash scripts/runtime/run_lifecycle_property_contract_lane.sh`
  - `bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh`
- Fail-closed regression marker:
  - dispute/refund replay mutation drift is rejected (`Regression: #904`).

## ZK Witness Mutation Contracts (Issue #994)
- Fast smoke/property lane:
  - `cargo test -p kamn-core --test zk_witness_fuzz_smoke fuzz_smoke_zk_witness_mutation_lane_is_panic_free_and_deterministic -- --exact`
  - `cargo test -p kamn-core --test zk_witness_fuzz_smoke functional_zk_witness_mutation_suite_covers_malformed_missing_and_tampered_classes -- --exact`
- Deep scheduled lane:
  - `cargo test -p kamn-core --test zk_witness_fuzz_smoke performance_zk_witness_mutation_deep_lane_stress -- --ignored`
  - `bash scripts/runtime/run_zk_witness_mutation_deep_lane.sh`
- Fail-closed regression marker:
  - selector/envelope mutation reason signatures remain stable (`Regression: #994`).

## Validation
Run from repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
