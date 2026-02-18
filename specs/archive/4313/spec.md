# Issue #4313 Spec

- Title: `Task: implement networked peer transport adapter integrity checks with deterministic retry-timeout governance`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4310`

## Problem Statement
Networked peer transport promotion requires deterministic integrity and retry-timeout reason governance so drift and tamper paths fail closed with stable reason outputs.

## Scope
In:
- Peer adapter integrity conformance tests for sender drift and retry-timeout classification.
- Deterministic peer adapter reason projection + multi-process validation hook APIs.
- Release and planning docs markers parity-guarded by docs tests.

Out:
- Gossip protocol redesign.
- New transport protocol/wire format.

## Acceptance Criteria
- AC-1: peer adapter checks validate integrity fields deterministically and fail closed on drift.
- AC-2: retry/timeout behavior emits stable reason codes and classification before/after budget boundaries.
- AC-3: deterministic multi-process peer validation hooks remain ordered and local-heavy scoped.
- AC-4: docs marker contracts for peer transport integrity/reason projection remain parity-guarded by tests.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout unit_peer_transport_rejects_sender_integrity_drift_with_reason_code -- --exact` | sender-integrity drift fails closed with deterministic reason |
| C-02 | AC-2 | Functional | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout functional_retry_timeout_fault_class_emits_timeout_reason_code -- --exact` | retry timeout maps to deterministic retry reason |
| C-03 | AC-2 | Integration | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout integration_retry_fault_matrix_classification_is_stable -- --exact` | retry fault matrix classification remains stable |
| C-04 | AC-2 | Regression | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout regression_retry_timeout_pre_budget_attempt_remains_timeout_classified -- --exact` | pre-budget attempts stay timeout-classified |
| C-05 | AC-2 | Performance | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout performance_retry_timeout_classification_stays_within_local_budget -- --exact` | retry classification loop remains bounded |
| C-06 | AC-2 | Unit | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection unit_retry_timeout_reason_projection_is_deterministic -- --exact` | retry projection reason code/class markers are deterministic |
| C-07 | AC-3 | Unit | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection unit_multi_process_validation_hooks_are_stable_and_ordered -- --exact` | multi-process hook ordering/taxonomy markers are deterministic |
| C-08 | AC-3 | Integration | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection integration_projection_and_hooks_reason_output_integrity_contract -- --exact` | reason projection and hooks compose deterministically |
| C-09 | AC-2 | Regression | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection regression_retry_timeout_budget_boundary_projection_stays_stable -- --exact` | budget-boundary reason projection remains stable |
| C-10 | AC-4 | Docs | `cargo test -p kamn-core --test kolme_devnet_ops_docs plan_contains_runtime_transport_retry_reconnect_failure_taxonomy -- --exact` | planning doc retains peer integrity/retry taxonomy markers |
| C-11 | AC-4 | Docs | `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_peer_adapter_reason_projection_multi_process_gate -- --exact` | release checklist retains peer adapter reason-projection gate markers |

## Test Mapping
- `crates/kamn-core/src/p2p_transport.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/p2p_peer_integrity_drift_timeout.rs`
- `crates/kamn-core/tests/p2p_peer_adapter_reason_projection.rs`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`

## Success Metrics
- Sender-integrity drift fails closed with deterministic reason output.
- Retry-timeout and retry-budget boundary reason projection is deterministic.
- Multi-process validation hooks remain ordered and local-heavy scoped.
- Docs parity tests prevent peer transport governance marker drift.
