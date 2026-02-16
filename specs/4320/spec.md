# Issue #4320 Spec

- Title: `Subtask: implement peer adapter reason projection and deterministic multi-process validation hooks`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4313`

## Problem Statement
Peer transport validation evidence must expose stable reason projection outputs and deterministic multi-process validation hooks so local-heavy contract lanes remain repeatable and fail-closed.

## Scope
In:
- Add deterministic peer adapter reason projection mapping for retry/timeout and fail-closed classes.
- Add deterministic multi-process peer validation hook definitions for process-isolated convergence lanes.
- Add tests proving reason projection determinism and hook repeatability.
- Update release go/no-go checklist with peer reason taxonomy references for multi-process validation.

Out:
- Distributed orchestrator redesign.
- New transport protocol implementation.

## Acceptance Criteria
- AC-1: reason projection remains deterministic across retry/timeout classifications and fail-closed markers.
- AC-2: deterministic multi-process validation hooks are exposed for repeatable local-heavy peer validation lanes.
- AC-3: integration tests validate reason output integrity across projection + hook contracts.
- AC-4: `docs/foundation/release-gonogo-checklist.md` includes peer reason taxonomy + multi-process validation references.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection unit_retry_timeout_reason_projection_is_deterministic -- --exact` | retry/timeout class maps to stable projected reason/classification markers |
| C-02 | AC-1 | Functional | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection functional_reason_projection_maps_fail_closed_transport_error -- --exact` | fail-closed transport error maps to deterministic projection |
| C-03 | AC-2 | Unit | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection unit_multi_process_validation_hooks_are_stable_and_ordered -- --exact` | hook list is deterministic and local-heavy aware |
| C-04 | AC-3 | Integration | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection integration_projection_and_hooks_reason_output_integrity_contract -- --exact` | projection outputs and hook reason contract remain internally consistent |
| C-05 | AC-3 | Regression | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection regression_retry_timeout_budget_boundary_projection_stays_stable -- --exact` | retry-budget boundary projects stable class/reason outputs |
| C-06 | AC-3 | Performance | `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection performance_reason_projection_loop_stays_within_local_budget -- --exact` | projection loop remains bounded |
| C-07 | AC-4 | Docs | `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_peer_adapter_reason_projection_multi_process_gate -- --exact` | release checklist retains required peer reason taxonomy and multi-process markers |

## Test Mapping
- `crates/kamn-core/tests/p2p_peer_adapter_reason_projection.rs` (new)
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `docs/foundation/release-gonogo-checklist.md`

## Success Metrics
- Peer adapter reason projection API yields deterministic outputs for timeout/retry/fail-closed classes.
- Multi-process hook list is deterministic and verifiable in tests.
- Release checklist includes peer reason taxonomy references aligned with contract tests.
