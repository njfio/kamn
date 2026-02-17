# Spec: Issue #4331

Status: Reviewed
Issue: #4331
Parent: #4326
Milestone: specs/milestones/r27-31-signal-safe-daemon-lifecycle-streaming-observability-and-runtime-decomposition-governance/index.md
Priority: P1

## Problem Statement

Signal lifecycle reason-projection contracts still allow malformed `signal@` tick payloads
(`signal@;` or `signal@abc`) to pass through contract validation as if they were valid ticks. Hook
selection also accepts explicit OS-signal mode for runtime kinds that should never use signal hooks.

## Scope

In scope:
- Harden shutdown signal tick parsing for supervisor stop contract classification.
- Ensure OS-signal hook selection is deterministic and runtime-mode scoped.
- Add release go/no-go checklist taxonomy markers for shutdown lifecycle reason mapping.
- Add unit/functional/integration/regression tests for hook-to-reason flow.

Out of scope:
- External signal broker integrations.
- New shutdown reason taxonomy families beyond deterministic mapping for existing outcomes.

## Acceptance Criteria

AC-1:
Given shutdown completion reasons in graceful/graceful-timeout forms, when signal tick is empty or
non-numeric, then contract classification fails closed with
`full_supervisor_stop_missing_signal_tick`.

AC-2:
Given explicit `--daemon-shutdown-os-signals` style controls, when runtime mode is not
`daemon|full`, then OS-signal hook selection remains disabled deterministically.

AC-3:
Given release checklist governance documentation, when go/no-go gates are reviewed, then shutdown
signal lifecycle reason taxonomy/version/value markers are present and test-validated.

## Conformance Cases

- C-01 (AC-1, Unit/Regression):
  - Test:
    `cargo test -p kamn-node main_tests::runtime_tests::regression_full_supervisor_stop_contract_classifier_rejects_empty_or_non_numeric_signal_tick -- --exact`
  - Expectation: malformed ticks fail closed with deterministic reason code.

- C-02 (AC-2, Unit/Regression):
  - Test:
    `cargo test -p kamn-node main_tests::runtime_tests::regression_shutdown_policy_rejects_os_signal_hooks_for_non_daemon_modes -- --exact`
  - Expectation: OS-signal hook selection disabled for non-daemon/full modes.

- C-03 (AC-3, Docs Contract):
  - Test:
    `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_shutdown_signal_lifecycle_reason_mapping_gate -- --exact`
  - Expectation: checklist contains required deterministic shutdown lifecycle markers.

- C-04 (AC-1/AC-2, Integration/Regression):
  - Test:
    `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_full_os_signal_timeout_stop_markers_project_shutdown_field_parity -- --exact`
  - Expectation: integrated full-runtime hook-to-reason timeout projection stays deterministic.

## Success Metrics / Observable Signals

- Malformed signal tick values cannot bypass stop-contract validation.
- OS-signal hook selection is runtime-mode deterministic.
- Release checklist includes explicit shutdown lifecycle reason taxonomy markers.
