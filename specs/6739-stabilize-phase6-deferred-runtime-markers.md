# Objective

Stabilize the `kamn-node` daemon Phase 6 deferred-runtime marker projection so `functional_runtime_daemon_projects_phase6_deferred_runtime_markers_when_shutdown_signals_are_present` passes both in isolation and inside the full `daemon_tests` suite.

# Inputs/Outputs

- Inputs:
  - Current `main` at `280bd32e26e0490a5801fc3be8c7f96b718602a4`
  - Repro where the target test passes in isolation but fails in the full `daemon_tests` run
  - Existing selector-bundle/runtime contract helpers under `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests/`
- Outputs:
  - Stable Phase 6 deferred marker assertions with deterministic capture order
  - A regression test or helper contract documenting the suite interaction that was fixed
  - Full `cargo test -p kamn-node daemon_tests -- --nocapture` green on the issue branch

# Boundaries/Non-goals

- Touch only the daemon runtime contract test surface and any narrowly scoped shared support required for deterministic capture
- Do not weaken or delete the failing test
- Do not refactor unrelated daemon runtime code or production behavior
- Do not fold `#6738` observability import regression work into this issue

# Failure modes

- Shared log capture or JSON rendering helpers leak state between tests in the full suite
- The test selects the wrong completion log line when multiple daemon runs emit similar markers
- Assertions depend on non-deterministic ordering in captured logs
- The test uses helper state that is valid in isolation but unstable under suite concurrency/order

# Acceptance criteria

- [ ] `cargo test -p kamn-node daemon_tests -- --nocapture` passes on the issue branch
- [ ] `functional_runtime_daemon_projects_phase6_deferred_runtime_markers_when_shutdown_signals_are_present` passes in isolation and in the full suite
- [ ] The root cause is documented in this spec with the exact suite interaction that was fixed
- [ ] A regression helper/test guards the stabilized log-selection or capture behavior

# Files to touch

- `specs/6739-stabilize-phase6-deferred-runtime-markers.md`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests/selector_bundle_contract_tests.rs`
- optional narrow helper/support files under `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests/`

# Error semantics

- Failing assertions remain hard failures with explicit missing-marker or wrong-log-line context
- No silent retries or swallowed capture errors
- If deterministic capture cannot be established, the test must fail with the precise missing selection reason

# Test plan

1. Reproduce the failure in the full `daemon_tests` suite and the pass in isolation
2. Add a red regression around the log-line selection/capture behavior if current helpers are ambiguous
3. Apply the minimum deterministic capture fix
4. Re-run the isolated failing test
5. Re-run `cargo test -p kamn-node daemon_tests -- --nocapture`
6. Re-run touched-Rust size policy on the issue branch

# Root cause and outcome

- Root cause: the deferred Phase 6 projection test selected the first `node.runtime.daemon.execute.complete` log line in the captured buffer and reused the default `kamn-devnet` execution identity. In the full suite, other daemon tests emit completion lines with the same event and default execution identity, so the test could bind to the wrong completion record.
- Fix: the test now runs under a dedicated chain id (`phase6-deferred-contract`) and resolves the completion log through an execution-id-aware helper instead of first-match selection.
- Result: the regression selector test passes, the deferred Phase 6 test passes in isolation, and the full `daemon_tests` target is green on the clean final branch.
