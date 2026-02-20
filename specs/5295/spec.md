# Issue #5295 Spec

- Title: Task: add stateful Phase-6 scheduler runtime checkpoint contracts
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-6 has stateless scheduler-cycle contracts (`#5293`) but no stateful runtime contract that tracks last successful tick checkpoint, monotonic scheduler-clock validation, and cycle counters across repeated executions.

## Scope
In:
- Add stateful Phase-6 scheduler runtime and state snapshot types.
- Compose existing scheduler-cycle trigger/budget/execution contracts under runtime state updates.
- Enforce monotonic scheduler-clock fail-closed behavior.
- Add conformance tests and ops-doc marker assertions.

Out:
- Async daemon orchestration in `kamn-node`.
- Persisting runtime checkpoint state to external storage.
- Shell/python/workflow/template changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 240
- shell_to_rust_ratio_delta_estimate: -0.0003
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Runtime initializes with deterministic zeroed counters and no last-successful tick checkpoint.
- AC-2: Deferred cycle increments deferred counters and preserves last-successful checkpoint.
- AC-3: Applied cycle increments executed counters and updates last-successful checkpoint.
- AC-4: Fail-closed cycle increments fail counter and keeps last-successful checkpoint unchanged.
- AC-5: Monotonic clock regression fails closed with stable scheduler-signal reason code.
- AC-6: Unit/Functional/Integration/Regression coverage and verification commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | runtime constructed with valid scheduler policy + budget | zero counters; no successful tick; stable initialized marker |
| C-02 | AC-2 | Functional | deferred cycle (no due work, interval not elapsed) | deferred counter increments; successful tick remains `None` |
| C-03 | AC-3 | Integration | triggered cycle with due work + within budget | executed counter increments; successful tick set to `now_epoch_seconds` |
| C-04 | AC-4 | Regression | triggered cycle with preflight budget overflow | fail counter increments; successful tick unchanged |
| C-05 | AC-5 | Regression | cycle call where `now_epoch_seconds` regresses below previous observed now | fail-closed scheduler-signal invalid error |
| C-06 | AC-6 | Verification | fmt/clippy + targeted conformance + docs marker tests | all pass |

## Success Metrics
- Story `#5253` gains a runtime-facing checkpoint/counter contract for Phase-6 scheduler continuity.
- Shell surface remains unchanged while Rust runtime integration coverage expands.
