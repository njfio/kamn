# Task Operations Command Surface (Issue #128 / #472)

This document defines the first implementation slice for task operation command
handling across `submit`, `accept`, `delegate`, `block`, `complete`, `fail`,
and `cancel`.

## Core Types
- `TaskOperationEngine`: deterministic in-memory task operations handler.
- `TaskOperationRecord`: task-level operation context:
  - `task_id`
  - `requester`
  - `assignee`
  - `description`
  - `lifecycle` (`TaskLifecycle`)
- `TaskOperationNoticeKind`:
  - `Submitted`
  - `Accepted`
  - `Delegated`
  - `Started`
  - `Blocked`
  - `Completed`
  - `Failed`
  - `Cancelled`
- `SwarmTaskDraft`: swarm DAG registration payload:
  - `task_id`
  - `requester`
  - `description`
  - `dependencies`

## Command Behavior
- `submit(task_id, requester, description)`:
  - creates a new task record in `Submitted`.
  - emits `Submitted` notice.
- `accept(task_id, actor)`:
  - transitions lifecycle using `Accept`.
  - binds `assignee = actor`.
  - emits `Accepted` notice.
- `delegate(task_id, actor, delegatee)`:
  - requires current assignee actor.
  - transitions via `Delegate`.
  - updates assignee to delegatee.
  - emits `Delegated` notice.
- `start_work(task_id, actor)`:
  - assignee-only.
  - transitions via `StartWork`.
  - emits `Started` notice.
- `block(task_id, actor, reason)`:
  - assignee-only with non-empty reason.
  - transitions via `Block`.
  - emits `Blocked` notice.
- `complete(task_id, actor)`:
  - assignee-only.
  - transitions via `Complete`.
  - emits `Completed` notice.
- `fail(task_id, actor, reason)`:
  - assignee-only with non-empty reason.
  - transitions via `Fail`.
  - emits `Failed` notice.
- `cancel(task_id, actor)`:
  - requester or current assignee.
  - transitions via `Cancel`.
  - emits `Cancelled` notice.
- `submit_swarm_tasks(drafts)`:
  - registers a bounded DAG-linked task set in a single deterministic pass.
  - rejects duplicate task IDs, duplicate dependency edges, unknown dependency references, and cyclic graphs.
  - initializes dependency metadata used by readiness checks.
- `ready_tasks()`:
  - returns deterministic ready-task IDs (sorted) where lifecycle state is `Accepted` or `Delegated` and all dependencies are `Completed`.
- `export_snapshot()`:
  - returns deterministic snapshot payload with schema version, task records, notices, lifecycle history, and dependency metadata.
- `restore_snapshot(snapshot)`:
  - validates schema version, lifecycle history, dependency references, and cycle safety before mutating engine state.
  - rejects tampered restore payloads where dependency-complete invariants are violated (`Regression: #502`).

## Validation and Safety Rules
- Task IDs must be unique.
- DIDs must parse as `kamn:did:agent:*`.
- Unauthorized actors are rejected with explicit required-role context.
- Underlying illegal or terminal lifecycle transitions bubble as typed lifecycle errors.
- Swarm dependency rules:
  - dependency IDs must reference registered tasks.
  - cyclic dependency graphs are rejected with `CyclicDependency`.
  - `start_work` is blocked when any dependency is not `Completed` (`DependencyNotSatisfied`).
  - replayed completion attempts remain rejected by terminal-state lifecycle guards (`Regression: #472`).
- Snapshot recovery rules:
  - schema version mismatch is rejected.
  - lifecycle history shape must be replayable from `Submitted`.
  - dependency references must resolve and remain acyclic.
  - tasks restored in execution states (`InProgress`/`Blocked`/`Completed`/`Failed`) require dependencies already `Completed` (`Regression: #502`).

## Bounded Graph Benchmark
- A bounded graph benchmark keeps CI cost low while validating DAG guard performance characteristics.
- The benchmark covers a 128-task linear DAG registration path and enforces a generous local CI budget.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test task_operations
cargo test -p kamn-core --test swarm_task_dag
cargo test -p kamn-core --test task_operation_snapshot
cargo test -p kamn-core
```

## Notes
This slice wires operation commands directly to the deterministic task lifecycle
validator to keep CI fast and behavior auditable.
