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
- Fuzz/mutation fail-closed lane:
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh`
- Concurrency state-mutation lane:
  - `bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh`
- Combined bounded lane with evidence output:
  - `bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json`
- Combined lane policy checker:
  - `bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json`
- Report schema:
  - `kamn.runtime.invariant-fuzz-concurrency-contract-report.v1`

## Validation
Run from repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
