# Spec: Issue 6189 - Atomic Service API State Writes

- Issue: #6189
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P0
- Area: backend

## Problem Statement

`ServiceApiMessageStore::persist` currently writes state with `fs::write(path, payload)`.
That pattern can truncate and partially rewrite the destination file non-atomically, leaving
state corruption risk if the process crashes during write.

## Scope

In scope:
1. Persist state snapshots via atomic write-then-rename in the same directory.
2. Ensure best-effort durability semantics by syncing file and parent directory.
3. Preserve existing public behavior and error propagation shape.

Out of scope:
1. WAL introduction.
2. Snapshot schema changes.
3. Replay guard persistence work.

## Acceptance Criteria

### AC-1 Atomic Replacement
Given a state file path,
When persistence succeeds,
Then write is performed via a temporary file and atomic rename over the target path.

### AC-2 Cleanup + Fail-Closed
Given write/flush/rename failures,
When persistence fails,
Then function returns `Err(String)` and does not leave temporary artifacts behind.

### AC-3 Regression Safety
Given existing message/channel/task persistence flows,
When persistence implementation is swapped,
Then existing service API state behavior remains unchanged.

## Conformance Cases

- C-01 (AC-1, Unit): atomic write helper replaces destination content and leaves no temp files.
- C-02 (AC-2, Unit): helper failure path removes temp file and returns deterministic write error.
- C-03 (AC-3, Integration): existing service-api message-store persistence tests remain green.

## Success Signals

1. `persist` no longer calls `fs::write(path, payload)` directly.
2. New atomic-write unit tests pass.
3. Existing service-api persistence integration lanes pass unchanged.
