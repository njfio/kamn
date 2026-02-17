# Tasks: Issue #4331

Status: Reviewed
Issue: #4331

## Ordered Tasks

T1 (RED)
- Add failing-first regression tests for:
  - malformed `signal@` tick values in stop-contract classifier.
  - explicit OS-signal hooks on non-daemon/full runtime modes.
  - missing release checklist shutdown lifecycle reason-taxonomy markers.

T2 (GREEN)
- Harden signal tick parsing and runtime-mode hook selection.
- Update release checklist with deterministic shutdown lifecycle reason mapping gate markers.

T3 (VERIFY)
- Run:
  - `cargo test -p kamn-node main_tests::runtime_tests::regression_full_supervisor_stop_contract_classifier_rejects_empty_or_non_numeric_signal_tick -- --exact`
  - `cargo test -p kamn-node main_tests::runtime_tests::regression_shutdown_policy_rejects_os_signal_hooks_for_non_daemon_modes -- --exact`
  - `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_full_os_signal_timeout_stop_markers_project_shutdown_field_parity -- --exact`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_shutdown_signal_lifecycle_reason_mapping_gate -- --exact`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - New regression tests and docs-contract assertion introduced to fail on malformed signal tick
    acceptance, unsupported runtime hook enablement, or missing checklist markers.

- GREEN command/output:
  - All verification commands in T3 pass after implementation.

- Regression summary:
  - Existing timeout hook-to-reason integration selector remains green.
