# Objective

Split `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs` into bounded production and test modules while preserving the existing runtime-orchestration entrypoint and daemon test behavior.

# Inputs/Outputs

Inputs:
- Existing daemon runtime orchestration in `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- Existing daemon runtime test surface in `crates/kamn-node/src/main_tests/daemon_tests.rs`
- Existing runtime extraction contracts in `crates/kamn-node/tests/main_module_extraction_contract.rs`

Outputs:
- A small `daemon_phase.rs` root that declares bounded submodules and keeps only the minimal public/test-facing surface
- Extracted production modules for daemon projections, live-postgres selector bundle helpers, service-api relay P2P forwarding, and daemon relay tick/shutdown helpers
- Extracted sibling daemon-phase test modules replacing the inline `#[cfg(test)] mod tests`
- Contract coverage that fails if the daemon-phase root re-accumulates the extracted seams

# Boundaries/Non-goals

Boundaries:
- Keep `runtime_orchestration.rs` as the real entrypoint and preserve its current imports/re-exports
- Preserve current daemon runtime semantics, error messages, telemetry counters, and test-only helper surface
- Limit the issue to splitting `daemon_phase.rs` and its inline tests; do not redesign runtime policy or full-supervisor logic

Non-goals:
- Changing daemon runtime behavior or public CLI contracts
- Changing shell/python/workflow surface
- Refactoring unrelated `kamn-node` test files outside the daemon-phase split contract needed for this extraction

# Failure modes

- The root `daemon_phase.rs` still contains inline phase-6 projection, live-postgres selector bundle, service-api relay P2P, or tick-loop bodies
- The root `daemon_phase.rs` still contains an inline `#[cfg(test)] mod tests`
- Extracted daemon-phase files exceed the active Rust size policy
- `runtime_orchestration.rs` stops re-exporting the existing daemon test hooks used by `main_tests::daemon_tests`
- Daemon runtime regression tests stop exercising the real runtime path after extraction

# Acceptance criteria

- [ ] `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs` is reduced to a bounded root module that delegates production logic to submodules
- [ ] `daemon_phase.rs` no longer contains inline implementations for phase-6/convergence projection helpers, live-postgres selector bundle helpers, service-api relay P2P helpers, or the daemon relay tick loop
- [ ] `daemon_phase.rs` no longer contains an inline `#[cfg(test)] mod tests`
- [ ] New extracted production/test files created by this issue stay within the active size policy on the touched write set
- [ ] `crates/kamn-node/tests/main_module_extraction_contract.rs` or a dedicated daemon extraction contract fails if the extracted seams are re-inlined into the root
- [ ] Existing daemon runtime contract/integration tests that exercise the re-exported test helpers and relay tick loop still pass through the real `runtime_orchestration` wiring

# Files to touch

- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- New files under `crates/kamn-node/src/runtime_orchestration/daemon_phase/`
- New files under `crates/kamn-node/src/runtime_orchestration/daemon_phase_tests/` or sibling bounded test files adjacent to the daemon-phase module
- `crates/kamn-node/tests/main_module_extraction_contract.rs` and/or a new daemon-phase extraction contract
- `crates/kamn-node/src/runtime_orchestration.rs` only if module declarations or test re-exports need to move to preserve current callers

# Error semantics

- Preserve existing hard-fail `ConfigError::RuntimeDaemonLifecycle` and string error behavior exactly at current boundaries
- Do not introduce silent fallbacks, swallowed errors, or relaxed validation
- Entrypoint behavior remains unchanged: `runtime_orchestration::execute` still delegates to `execute_daemon_runtime` and propagates failures unchanged

# Test plan

1. Add a red extraction contract that asserts:
   - bounded daemon-phase submodule declarations exist
   - the daemon root no longer contains the extracted production seams
   - the daemon root no longer contains inline tests
   - staged root/file-size limits for the touched daemon-phase files hold
2. Turn the contract green by extracting:
   - phase-6/convergence projection helpers
   - live-postgres selector bundle helpers
   - service-api relay P2P config/context/forwarding helpers
   - relay tick loop/sleep helpers
   - inline daemon-phase tests into bounded sibling files
3. Run focused runtime verification:
   - `cargo test -p kamn-node daemon_phase`
   - `cargo test -p kamn-node daemon_tests -- --nocapture`
   - `cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture`
   - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json <tmpfile>`
