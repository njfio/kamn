# Spec: Issue #6133 - Startup-cached node log configuration

Status: Accepted
Issue: #6133
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`emit_log_event` currently calls `resolve_log_config_from_env()` on every log emission. This repeatedly reads process environment variables in hot logging paths and was flagged in R59 (`S-10`).

## Scope
In scope:
- Cache node log config in process state and use cached config for log emission.
- Initialize/refresh cache at startup execution entrypoints.
- Add regression coverage proving emission no longer re-reads env each call.

Out of scope:
- Changes to log event schema/format.
- Cross-crate logging redesign.

## Acceptance Criteria
- AC-1: `emit_log_event` uses cached log config and does not call env resolution on each emission.
- AC-2: Startup paths initialize log config from env and preserve existing invalid-config fail-closed behavior.
- AC-3: Regression tests demonstrate that mutating env after initialization does not change emission behavior for that initialized runtime.

## Conformance Cases
- C-01 (AC-1): `emit_log_event` reads config from cache and emits using cached level/format.
- C-02 (AC-2): Runtime execute path initializes config from env and still errors on invalid log env values.
- C-03 (AC-3): After initialization with `info/text`, changing env to `error/json` does not suppress subsequent `log_info` emission.

## Success Metrics
- `cargo test -p kamn-node logging::tests::`
- `cargo test -p kamn-node regression_invalid_log_level_config_fails_closed`
- `cargo clippy -p kamn-node --tests -- -D warnings`
