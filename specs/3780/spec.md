# Issue #3780 Spec

- Title: Task: add local-heavy transport resilience lane with CI exclusion contracts
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Transport resilience local-heavy validation needed deterministic lane artifacts and policy enforcement while remaining excluded from PR fast CI scope.

## Acceptance Criteria
- AC-1: Local-heavy transport resilience lane produces deterministic retry/reconnect evidence and policy outputs.
- AC-2: CI policy fails closed if transport resilience local-heavy run-mode commands leak into fast PR workflow surfaces.
- AC-3: Strategy/docs command-surface declarations remain synchronized with policy/checker marker taxonomy.
- AC-4: Unit/Functional/Integration/Regression evidence for lane + exclusion contracts is passing.

## Scope
In scope:
- Child subtask `#3794` delivery (lane/policy marker contract propagation).
- Child subtask `#3795` delivery (CI-fast exclusion + docs parity contract tightening).
- Task closure lineage artifacts in `specs/3780/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Running local-heavy transport lane unconditionally in `ci-fast-gate`.
- New transport protocol/runtime behavior.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | transport fault-matrix lane dry-run contract tests | deterministic marker/taxonomy outputs present |
| C-02 | AC-1 | Regression | transport policy checker tamper tests | missing/mismatched markers fail closed |
| C-03 | AC-2 | Functional | CI exclusion policy script over workflow + ci-tools surfaces | run-mode leakage into fast scope fails closed |
| C-04 | AC-3 | Regression | strategy/docs contract tests | transport fault-matrix marker/taxonomy declarations remain synchronized |
| C-05 | AC-4 | Integration | combined lane + exclusion verification bundle | all closure tests pass together |

## Test Mapping
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `bash scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`
- `cargo test -p kamn-node --test kolme_devnet_ops_docs`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_live_transport_fault_matrix_ci_exclusion_policy_contract_markers -- --exact`

## Success Metrics
- Both child subtasks are merged and parent checklist reflects completion.
- Deterministic transport resilience lane markers and CI exclusion/docs parity checks are green.
