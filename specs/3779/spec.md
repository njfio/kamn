# Issue #3779 Spec

- Title: Task: harden Kolme notifications reconnect pacing and terminal taxonomy
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Kolme notifications reconnect loops required deterministic pacing and explicit terminal taxonomy markers to avoid tight retry loops and drift-prone failure classification.

## Acceptance Criteria
- AC-1: Reconnect loops apply deterministic pacing/backoff behavior.
- AC-2: Terminal reconnect failures emit deterministic reason taxonomy/markers.
- AC-3: Contract tests fail closed on reconnect marker/taxonomy drift.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- Child subtask `#3793` (deterministic reconnect pacing schedule).
- Child subtask `#3792` (terminal taxonomy markers + docs drift contracts).
- Parent closure lineage artifacts in `specs/3779/{spec.md,plan.md,tasks.md}`.

Out of scope:
- New websocket protocol features.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | notifications reconnect exhaustion pacing tests | deterministic pacing markers and bounded behavior present |
| C-02 | AC-2 | Unit | notification policy reconnect terminal reason composition | deterministic reason_code/reason_taxonomy_version markers present |
| C-03 | AC-3 | Regression | docs + policy contract tests | reconnect taxonomy marker drift fails closed |
| C-04 | AC-4 | Integration | notifications consumer websocket reconnect tests | reconnect behavior remains green under integration scenarios |
| C-05 | AC-4 | Regression | combined reconnect verification bundle | all reconnect lane checks pass together |

## Test Mapping
- `cargo test -p kamn-kolme --test notification_policy_contracts`
- `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
- `cargo test -p kamn-node --test kolme_runtime_commit_docs`
- `cargo test -p kamn-core --test runtime_network_docs`

## Success Metrics
- Both child subtasks (`#3793`, `#3792`) are merged and reflected in parent checklist.
- Reconnect pacing + taxonomy contracts are passing across policy/runtime/docs surfaces.
