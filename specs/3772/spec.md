# Issue #3772 Spec

- Title: Epic: observability and transport resilience hardening
- Status: Reviewed
- Type: epic
- Priority: P0
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Production-readiness required two major hardening tracks: standardized tracing/observability serving contracts and deterministic shared transport retry/reconnect resilience with bounded CI/local-heavy governance.

## Acceptance Criteria
- AC-1: Runtime/service observability surfaces use deterministic structured tracing/serving contracts.
- AC-2: Shared transport clients and notification reconnect loops enforce deterministic retry/reconnect behavior with fail-closed taxonomy markers.
- AC-3: CI fast-lane boundaries remain bounded while local-heavy validation evidence remains available and contract-guarded.
- AC-4: Unit/Functional/Integration/Regression/Performance evidence is present across child story chains.

## Scope
In scope:
- Story `#3773` closure (tracing + observability serving contracts).
- Story `#3774` closure (shared transport retry/reconnect resilience and CI/local-heavy transport contracts).
- Epic closure lineage artifacts in `specs/3772/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Multi-region observability backend deployment.
- Protocol economics/consensus redesign.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Integration | observability route parity/secure-serving contract suites | deterministic route/security markers remain green |
| C-02 | AC-2 | Unit/Functional | transport retry/reconnect policy/runtime suites | deterministic retry/reconnect pacing and taxonomy markers remain green |
| C-03 | AC-3 | Regression | CI exclusion policy + docs parity checks | fast-lane leakage and marker-drift checks fail closed |
| C-04 | AC-4 | Integration | combined child-story verification bundles | cross-story evidence remains green |
| C-05 | AC-4 | Performance | bounded CI/local-heavy governance checks | no fast-lane cost boundary regressions |

## Test Mapping
- `cargo test -p kamn-kolme --test notification_policy_contracts`
- `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
- `cargo test -p kamn-node --test kolme_runtime_commit_docs`
- `cargo test -p kamn-core --test runtime_network_docs`
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `bash scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_live_transport_fault_matrix_ci_exclusion_policy_contract_markers -- --exact`

## Success Metrics
- Child stories `#3773` and `#3774` are closed and reflected in epic checklist.
- Combined observability + transport resilience contract surfaces remain green.
