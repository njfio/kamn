# Plan: Issue #6133

## Approach
1. Add a process-local cached log-config store in `crates/kamn-node/src/logging.rs`.
2. Add explicit initialization API (`initialize_log_config_from_env`) and switch `emit_log_event` to cached reads.
3. Call initialization at runtime startup path(s) before log-heavy execution.
4. Add RED regression test in `logging.rs` proving env mutation after initialization does not alter emission behavior.
5. Run scoped fmt/clippy/tests for `kamn-node`.

## Affected Modules
- `crates/kamn-node/src/logging.rs`
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/runtime_orchestration.rs`

## Risks
- Risk: introducing global mutable cache can affect test ordering.
  - Mitigation: startup path always initializes from env; tests that mutate env use existing env locks.
- Risk: startup error semantics drift.
  - Mitigation: preserve `ConfigError::InvalidLogConfig` return path via init call.

## Interfaces/Contracts
- New internal API: `initialize_log_config_from_env()` for startup initialization.
- Existing external behavior preserved, except log emission now uses cached config.
