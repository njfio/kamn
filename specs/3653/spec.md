# Issue #3653 Spec

- Title: `Subtask: extract signer_adapter for key-source and crypto paths`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer key-source and cryptographic operations needed dedicated adapter ownership to reduce monolith coupling.

## Scope
In:
- Extract key-source and crypto paths into signer adapter ownership.
- Preserve deterministic signing behavior and parity.
- Add boundary drift guards.

Out:
- Policy/quorum extraction.

## Acceptance Criteria
- AC-1: signer adapter owns key-source and crypto paths.
- AC-2: boundary drift contracts enforce adapter ownership.
- AC-3: signing behavior remains parity-stable.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Functional | `cargo test -p kamn-node --test signer_adapter_boundary_contract` | adapter boundary contract passes |
| C-02 | AC-2 | Functional | `cargo test -p kamn-node --test signer_extraction_budget_contract` | extraction ownership guards pass |
| C-03 | AC-3 | Integration | `bash scripts/kolme/test_run_signature_parity_matrix.sh` | signature parity matrix lane passes |
| C-04 | AC-3 | Regression | `bash scripts/kolme/test_check_signature_parity_policy.sh` | parity policy checks pass |

## Test Mapping
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `crates/kamn-node/tests/signer_extraction_budget_contract.rs`
- `scripts/kolme/test_run_signature_parity_matrix.sh`
- `scripts/kolme/test_check_signature_parity_policy.sh`

## Success Metrics
- Adapter ownership and parity checks are deterministic and green.
