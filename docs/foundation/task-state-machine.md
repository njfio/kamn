# Task State Machine and Transition Validator (Issue #126)

This document defines the first implementation slice for deterministic task
state transitions and legal transition validation.

## Task States
- `Submitted`
- `Accepted`
- `Delegated`
- `InProgress`
- `Blocked`
- `Completed` (terminal)
- `Failed` (terminal)
- `Cancelled` (terminal)

## Supported Transitions
- `Submitted -> Accepted | Cancelled`
- `Accepted -> Delegated | InProgress | Cancelled`
- `Delegated -> InProgress | Cancelled`
- `InProgress -> Blocked | Completed | Failed | Cancelled`
- `Blocked -> InProgress | Failed | Cancelled`

Any transition outside this map is rejected.

## APIs
- `TaskLifecycle::new(task_id)` initializes task in `Submitted`.
- `transition(TaskTransition)` validates and applies a transition.
- `state()` returns current state.
- `history()` returns immutable state history sequence.

## Validation Rules
- Empty task IDs are rejected.
- Terminal states reject follow-up transitions (`TerminalState`).
- Illegal transitions return `InvalidTransition { from, transition }`.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test task_state_machine
cargo test -p kamn-core
```

## Notes
This slice uses deterministic in-memory validation with explicit typed errors
for low-cost, high-signal CI coverage.
