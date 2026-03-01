# Issue 6271 - Stabilize runtime full OS-signal timeout stop-marker parity test

## Objective
Remove intermittent assertion drift in the full-runtime OS-signal timeout stop-marker parity regression test by aligning expected completion-reason semantics with observed runtime behavior under bounded timeout controls.

## Inputs/Outputs
- Inputs:
  - `kamn-node` full runtime test args with:
    - `--daemon-shutdown-os-signals`
    - `--daemon-shutdown-drain-ticks 5`
    - `--daemon-shutdown-timeout-ticks 1`
  - deterministic test trigger: `Sigint` at tick `5`
  - captured runtime logs and returned execution report
- Outputs:
  - stable assertion contract for `daemon_completion_reason` parity between report and stop-complete log marker
  - deterministic timeout-marker field expectations

## Boundaries/Non-goals
- In scope:
  - `crates/kamn-node/src/main_tests/runtime_tests/full_supervisor_os_signal_shutdown_tests.rs`
  - minimal runtime-test helper adjustments required for deterministic marker assertion
- Out of scope:
  - broad runtime orchestrator redesign
  - unrelated shutdown policy behavior changes
  - non-test refactors outside the failing contract path

## Failure modes
- FM1: `daemon_completion_reason` expectation is hard-coded to a stale/alternate reason class and mismatches report/log parity.
- FM2: timeout marker fields are asserted before reason stabilization and can drift under cross-suite timing.
- FM3: repeated full-suite runs intermittently fail due nondeterministic reason-class expectation rather than true behavior drift.

## Acceptance criteria (testable booleans)
- AC1: `main_tests::runtime_tests::regression_runtime_full_os_signal_timeout_stop_markers_project_shutdown_field_parity` passes repeatedly in isolation.
- AC2: The test asserts marker parity against runtime report semantics without stale hard-coded reason assumptions.
- AC3: Timeout marker field assertions remain intact (`shutdown_drain_status`, `shutdown_snapshot_flush_status`, and shutdown parity fields).
- AC4: The test passes when exercised via full `make test` lane in this branch.

## Files to touch
- `crates/kamn-node/src/main_tests/runtime_tests/full_supervisor_os_signal_shutdown_tests.rs`

## Error semantics
- Contract tests fail loudly when stop-marker fields diverge from report projection.
- Assertions use deterministic, explicit expectation rules for timeout reason classes.
- No silent fallback or skipped assertion for completion-reason mismatches.

## Test plan
- RED:
  - run the targeted flaky test repeatedly and capture drift.
  - add/adjust assertion coverage to expose current mismatch deterministically.
- GREEN:
  - implement minimal assertion/fixture adjustment to align with runtime contract.
  - rerun targeted test repeatedly.
- REFACTOR:
  - extract concise helper assertions if needed to reduce duplication and clarify parity rules.
- INTEGRATION:
  - run `make test` and verify no recurrence of this parity failure.
