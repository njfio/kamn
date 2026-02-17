# Spec: Issue #4330

Status: Reviewed
Issue: #4330
Parent: #4326
Milestone: specs/milestones/r27-31-signal-safe-daemon-lifecycle-streaming-observability-and-runtime-decomposition-governance/index.md
Priority: P1

## Problem Statement

Shutdown-signal coverage currently under-specifies SIGINT-triggered graceful path handling and
OS-signal timeout phase-marker parity checks in full runtime supervisor stop output.

## Scope

In scope:
- Add RED-first tests for SIGINT/SIGTERM capture coverage gaps in daemon shutdown signal tests.
- Add RED-first tests for timeout shutdown-phase marker stability in full runtime OS-signal path.
- Update operations configuration docs with explicit signal-failure matrix references.

Out of scope:
- New orchestrator integrations.
- New shutdown reason taxonomy families.

## Acceptance Criteria

AC-1:
Given daemon runtime OS-signal mode on Unix, when SIGINT is the first received shutdown signal,
then graceful-shutdown completion remains deterministic and bounded.

AC-2:
Given full runtime OS-signal mode with drain budget exceeding timeout budget, when shutdown is
triggered by OS signal, then supervisor stop markers project timeout phase fields with deterministic
reason shape.

AC-3:
Given operations configuration docs, when operators configure shutdown controls, then signal-failure
matrix references are explicit and consistent with runbook contracts.

## Conformance Cases

- C-01 (AC-1, Integration/Regression):
  - Test:
    `cargo test -p kamn-node daemon_shutdown::tests::integration_daemon_completion_with_os_signals_applies_sigint_graceful_shutdown -- --exact`
  - Expectation: SIGINT-triggered graceful shutdown path succeeds and remains bounded.

- C-02 (AC-2, Integration/Regression):
  - Test:
    `cargo test -p kamn-node regression_runtime_full_os_signal_timeout_stop_markers_project_shutdown_field_parity -- --exact`
  - Expectation: timeout-phase supervisor stop markers include deterministic parity fields.

- C-03 (AC-3, Docs Contract):
  - Test:
    `cargo test -p kamn-core --test service_api_ops_configuration_docs`
  - Expectation: ops configuration docs contract suite remains green after guidance update.

## Success Metrics / Observable Signals

- SIGINT-first shutdown path has explicit regression test coverage.
- OS-signal timeout phase projection has explicit regression test coverage.
- `docs/ops/configuration.md` references signal-failure matrix guidance.
