# Issue #4319 Spec

- Title: `Subtask: add red tests for networked peer integrity drift and retry-timeout misclassification`
- Status: `Reviewed`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4313`

## Problem Statement
Networked peer transport safety can regress if sender-integrity drift is accepted or retry-timeout faults are misclassified into unstable reason-code classes.

## Scope
In:
- Add deterministic tests that fail when sender-integrity drift is accepted by peer transport.
- Add deterministic tests that fail when retry-timeout classification drifts.
- Update `docs/planning/kolme-devnet-ops.md` with peer-integrity drift test markers and reason-code expectations.

Out:
- Transport protocol redesign.
- Runtime retry algorithm changes.

## Acceptance Criteria
- AC-1: peer transport test fixtures fail-closed for sender-integrity drift with deterministic reason code.
- AC-2: retry-timeout fault classification remains deterministic and does not drift into non-timeout reason classes before budget exhaustion.
- AC-3: regression selectors preserve deterministic peer rejection behavior for drift/misclassification paths.
- AC-4: `docs/planning/kolme-devnet-ops.md` documents peer-integrity drift + timeout-classification contract markers.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout unit_peer_transport_rejects_sender_integrity_drift_with_reason_code -- --exact` | drift sender is rejected with `p2p_transport_unknown_sender_peer` |
| C-02 | AC-2 | Functional | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout functional_retry_timeout_fault_class_emits_timeout_reason_code -- --exact` | dial-timeout retry retains timeout-class reason code |
| C-03 | AC-2 | Integration | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout integration_retry_fault_matrix_classification_is_stable -- --exact` | mixed fault matrix yields deterministic reason-code decisions |
| C-04 | AC-3 | Regression | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout regression_retry_timeout_pre_budget_attempt_remains_timeout_classified -- --exact` | pre-budget dial-timeout attempts are not misclassified as exhausted |
| C-05 | AC-3 | Performance | `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout performance_retry_timeout_classification_stays_within_local_budget -- --exact` | classification loop remains bounded while preserving reason determinism |
| C-06 | AC-4 | Docs | `cargo test -p kamn-core --test kolme_devnet_ops_docs plan_contains_runtime_transport_retry_reconnect_failure_taxonomy -- --exact` | docs preserve taxonomy section |

## Test Mapping
- `crates/kamn-core/tests/p2p_peer_integrity_drift_timeout.rs` (new)
- `docs/planning/kolme-devnet-ops.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Success Metrics
- Drift/misclassification regressions are caught by deterministic tests.
- No retry-timeout reason-code instability in covered matrix paths.
- Docs include explicit peer-integrity drift and timeout-classification contract markers.
