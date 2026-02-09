# Task State Machine and Transition Validator (Issue #126 / #472 / #573)

This document defines the first implementation slice for deterministic task
state transitions and legal transition validation.

## Task States
- `Submitted`
- `Accepted`
- `Delegated`
- `InProgress`
- `InputRequired`
- `Blocked`
- `Completed` (terminal)
- `Failed` (terminal)
- `Cancelled` (terminal)

## Supported Transitions
- `Submitted -> Accepted | Cancelled`
- `Accepted -> Delegated | InProgress | Cancelled`
- `Delegated -> InProgress | Cancelled`
- `InProgress -> InputRequired | Blocked | Completed | Failed | Cancelled`
- `InputRequired -> InProgress | Failed | Cancelled`
- `Blocked -> InProgress | Failed | Cancelled`

Any transition outside this map is rejected.

## Dependency-Aware Transition Gates
- For standalone tasks, lifecycle transition rules above apply directly.
- For swarm DAG tasks, `TaskOperationEngine::start_work` adds a dependency gate:
  - all declared dependencies must already be in `Completed` state.
  - unsatisfied dependencies are rejected with `DependencyNotSatisfied`.
  - cyclic DAG registration is rejected before lifecycle transitions begin (`Regression: #472`).
- Snapshot restore invariants:
  - lifecycle history must replay deterministically from `Submitted`.
  - restore rejects dependency-state tampering where execution states appear before dependency completion (`Regression: #502`).

## APIs
- `TaskLifecycle::new(task_id)` initializes task in `Submitted`.
- `transition(TaskTransition)` validates and applies a transition.
- `state()` returns current state.
- `history()` returns immutable state history sequence.

## Validation Rules
- Empty task IDs are rejected.
- Terminal states reject follow-up transitions (`TerminalState`).
- Illegal transitions return `InvalidTransition { from, transition }`.

## Transition Evidence and Reason-Code Contract
- `TaskLifecycle::transition_with_evidence(TaskTransition)` emits deterministic evidence for authorized transitions:
  - `TaskTransitionEvidence { from, transition, to, reason_code }`
  - allowed transition reason code: `task_transition_allowed`
- Rejected transition reason codes are deterministic via `TaskLifecycleError::reason_code()`:
  - `task_id_empty`
  - `task_history_invalid`
  - `task_transition_invalid_edge`
  - `task_transition_terminal_state`
- Regression policy:
  - transition reason-code drift and illegal transition acceptance fail closed (`Regression: #903`).

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test task_state_machine
cargo test -p kamn-core --test task_escrow_transition_contracts
cargo test -p kamn-core --test task_state_machine_docs
cargo test -p kamn-core --test swarm_task_dag
cargo test -p kamn-core --test task_operation_snapshot
cargo test -p kamn-core
```

## Notes
This slice uses deterministic in-memory validation with explicit typed errors
for low-cost, high-signal CI coverage.
