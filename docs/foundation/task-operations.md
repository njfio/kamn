# Task Operations Command Surface (Issue #128)

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

## Validation and Safety Rules
- Task IDs must be unique.
- DIDs must parse as `kamn:did:agent:*`.
- Unauthorized actors are rejected with explicit required-role context.
- Underlying illegal or terminal lifecycle transitions bubble as typed lifecycle errors.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test task_operations
cargo test -p kamn-core
```

## Notes
This slice wires operation commands directly to the deterministic task lifecycle
validator to keep CI fast and behavior auditable.
