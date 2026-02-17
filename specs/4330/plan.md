# Plan: Issue #4330

Status: Reviewed
Issue: #4330

## Approach

1. Add daemon-shutdown integration test for SIGINT-first OS-signal capture path.
2. Add full-runtime regression test for OS-signal timeout shutdown-phase marker parity.
3. Update ops configuration docs to point to signal-failure runbook matrix.
4. Run focused `kamn-node` selectors for new tests and impacted docs contract checks.

## Affected Modules

- `crates/kamn-node/src/daemon_shutdown.rs`
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `docs/ops/configuration.md`

## Risks / Mitigations

- Risk: OS-signal tests can be timing-sensitive.
  - Mitigation: keep bounded tick windows, deterministic trigger delays, and existing signal test
    runtime lock usage.

- Risk: timeout marker assertions could become brittle.
  - Mitigation: assert deterministic prefix + required parity fields instead of fragile full-string
    equality for dynamic signal ticks.

## Interface Contract

- No API signature or protocol changes.
- Test and documentation contract coverage only.

## Review Note (P1)

- This is a P1 subtask; PR requests human review before merge.
