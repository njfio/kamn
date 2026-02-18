# Issue #3889 Spec

- Title: Task: enforce activation go-no-go budget and documentation parity contracts
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Activation closure needs deterministic gating that combines readiness markers, budget status, and docs synchronization.

## Scope
In:
- Add go-no-go marker checks and budget policy validation.
- Add docs-contract and milestone summary parity checks.

Out:
- Additional interoperability scenarios.

## Acceptance Criteria
- AC-1:  Activation gate fails closed on readiness marker or budget violations.
- AC-2:  Docs parity and summary checks fail on marker drift.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing.
- AC-4:  Performance budgets remain bounded.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration | `bash scripts/runtime/test_run_go_no_go_gate_lane.sh` | Activation go/no-go gate fails closed on readiness-marker drift and runtime budget violations with deterministic reason codes. |
| C-02 | AC-2 | Regression/Integration | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`; `cargo test -p kamn-core --test ci_strategy_docs doc_contains_live_gonogo_boundary_reason_taxonomy_markers -- --exact`; `cargo test -p kamn-core --test kolme_devnet_ops_docs` | Docs parity and milestone-summary marker checks fail closed on drift and remain synchronized across strategy + plan docs. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | `bash scripts/runtime/test_run_go_no_go_gate_lane.sh`; `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`; `cargo test -p kamn-core --test ci_strategy_docs doc_contains_live_gonogo_boundary_reason_taxonomy_markers -- --exact`; `cargo test -p kamn-core --test kolme_devnet_ops_docs` | Required test categories are present and passing for the activation closure surface. |
| C-04 | AC-4 | Functional/Performance | `bash scripts/runtime/test_run_go_no_go_gate_lane.sh` | Go/no-go runtime budget thresholds remain enforced and bounded with deterministic fail-closed behavior when exceeded. |

## Test Mapping
- `scripts/runtime/test_run_go_no_go_gate_lane.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
