# Issue #3774 Spec

- Title: Story: harden shared Kolme transport retry and reconnect resilience
- Status: Reviewed
- Type: story
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Shared Kolme transport client paths needed deterministic retry/reconnect behavior and fail-closed taxonomy contracts to avoid instability outside the primary runtime loop.

## Acceptance Criteria
- AC-1: Shared transport clients classify transient failures and apply deterministic bounded retry/backoff.
- AC-2: Notification reconnect loops apply bounded pacing and deterministic terminal taxonomy.
- AC-3: Contract tests fail closed on retry/reconnect marker drift and taxonomy mismatch.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- Task `#3778` closure: shared HTTP retry/backoff hardening.
- Task `#3779` closure: notifications reconnect pacing/taxonomy hardening.
- Task `#3780` closure: local-heavy transport resilience lane + CI exclusion/docs parity contracts.
- Story closure lineage artifacts in `specs/3774/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Protocol-level finality semantic changes.
- Unbounded retry loops.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | shared transport retry/backoff contract tests | deterministic bounded retry behavior present |
| C-02 | AC-2 | Functional | notifications reconnect pacing/taxonomy tests | deterministic reconnect pacing + terminal markers present |
| C-03 | AC-3 | Regression | policy/docs drift checks | retry/reconnect taxonomy mismatch fails closed |
| C-04 | AC-4 | Integration | local-heavy transport resilience lane + CI exclusion checks | transport lane/exclusion contracts remain green |
| C-05 | AC-4 | Regression | combined story verification bundle | all child-surface checks pass together |

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
- Child tasks `#3778`, `#3779`, and `#3780` are closed and reflected in story checklist.
- Retry/reconnect behavior and transport resilience CI/docs contracts remain deterministic and green.
