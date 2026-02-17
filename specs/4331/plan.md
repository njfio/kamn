# Plan: Issue #4331

Status: Reviewed
Issue: #4331

## Approach

1. Tighten `daemon_shutdown_signal_tick` parsing to reject empty/non-numeric tick values.
2. Scope `should_use_os_signal_shutdown` to `daemon|full` runtime kinds even when explicit
   OS-signal flag is set.
3. Add regression tests for malformed tick rejection and non-daemon hook disablement.
4. Add release checklist section + docs contract test assertions for shutdown lifecycle reason
   taxonomy markers.

## Affected Modules

- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks / Mitigations

- Risk: tightening signal tick parsing may change classification for malformed legacy strings.
  - Mitigation: explicitly fail closed to existing deterministic reason code
    `full_supervisor_stop_missing_signal_tick`.

- Risk: runtime-mode gating change could alter behavior in unsupported modes.
  - Mitigation: gate only non-daemon/full modes and add targeted regression tests.

## Interface Contract

- No external API signatures changed.
- No protocol/wire-format changes.
- Deterministic contract validation behavior is hardened.

## Review Note (P1)

- P1 subtask; PR explicitly requests human review before merge.
