# Issue #3628 Spec

- Title: `Story: decompose signer monolith into modular signing adapter and policy layers`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
`crates/kamn-node/src/signer.rs` accumulated adapter, policy, orchestration, and runtime linkage responsibilities in one file, making review and change isolation unsafe.

## Scope
In:
- Decompose signer responsibilities into module boundaries (`signer_adapter`, `signer_policy`, and supporting extraction seams).
- Preserve signer behavior parity for profile normalization, key source resolution, nonce handling, and managed backend flows.
- Add deterministic drift guards for signer ownership and reason taxonomy.

Out:
- New signing algorithms.
- Protocol or wire format redesign.

## Acceptance Criteria
- AC-1: signer module boundaries are explicit and auditable.
- AC-2: signer behavior remains parity-stable for existing runtime flows.
- AC-3: deterministic policy reason taxonomy and adapter ownership drift checks are enforced.
- AC-4: signer migration/compatibility lanes remain green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `cargo test -p kamn-node --test signer_adapter_boundary_contract` | adapter boundary contract passes |
| C-02 | AC-1/AC-3 | Functional | `cargo test -p kamn-node --test signer_extraction_budget_contract` | extraction budget/ownership guards pass |
| C-03 | AC-3 | Functional | `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract` | reason taxonomy drift checks pass |
| C-04 | AC-2/AC-4 | Integration | `bash scripts/kolme/test_run_signature_parity_contract_lane.sh` | signature parity lane passes |
| C-05 | AC-2/AC-4 | Integration | `bash scripts/kolme/test_run_nonce_broadcast_parity_contract_lane.sh` | nonce parity lane passes |
| C-06 | AC-2/AC-4 | Integration | `bash scripts/kolme/test_run_managed_signer_startup_live_validation_contract_lane.sh` | managed signer startup parity lane passes |

## Test Mapping
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `crates/kamn-node/tests/signer_extraction_budget_contract.rs`
- `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- `scripts/kolme/test_run_signature_parity_contract_lane.sh`
- `scripts/kolme/test_run_nonce_broadcast_parity_contract_lane.sh`
- `scripts/kolme/test_run_managed_signer_startup_live_validation_contract_lane.sh`

## Success Metrics
- Signer decomposition contracts and parity lanes pass deterministically.
- Signer boundary and taxonomy drift are fail-closed.
