# 6620-add-runtime-guard-watchdog-coverage

## Objective
Add dedicated crate-level integration coverage for the public `kamn_runtime_guards::watchdog` boundary so watchdog alert classification and aggregate snapshot behavior remain pinned outside inline module tests.

## Inputs/Outputs
- Inputs:
  - public `WatchdogConfig`, `WatchdogNode`, and `WatchdogObservation` values covering mixed block and gossip sequences
  - invalid config and invalid observation values through the public API
- Outputs:
  - dedicated integration test surface at `crates/kamn-runtime-guards/tests/runtime_guard_watchdog.rs`
  - dedicated contract test at `crates/kamn-runtime-guards/tests/runtime_guard_watchdog_contract.rs`
  - refreshed `test_file_size_policy` baseline if the added test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-runtime-guards/src/watchdog.rs`
- No changes to `anti_spam`, `quota_policy`, `fairness_policy`, or `policy_stack`
- No CI or workflow changes
- No internal helper visibility changes solely for test access

## Failure modes
- Dedicated watchdog integration surface missing entirely
- Mixed block sequencing stops emitting invalid-parent critical alerts
- Low quorum stops emitting critical quorum alerts
- Low delivery ratio stops emitting warning censorship alerts
- Single-recipient gossip stops passing through without alerts
- Aggregate snapshot counters drift from processed alert sequence
- Invalid config stops failing closed
- Invalid observations stop failing closed
- Workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `runtime_guard_watchdog_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage emits critical `InvalidBlockParent` and `QuorumAnomaly` alerts through the public API on mixed block sequences
- [ ] integration coverage emits warning `CensorshipSignal` alerts through the public API on degraded gossip delivery
- [ ] integration coverage emits no alert for single-recipient gossip and preserves aggregate counters accordingly
- [ ] integration coverage proves `snapshot()` counters match total processed observations and emitted warning/critical counts across a mixed sequence
- [ ] integration coverage asserts invalid config and invalid observations fail closed through the public API
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_watchdog_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_watchdog -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6620-add-runtime-guard-watchdog-coverage.md`
- `crates/kamn-runtime-guards/tests/runtime_guard_watchdog.rs`
- `crates/kamn-runtime-guards/tests/runtime_guard_watchdog_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public watchdog fail-closed behavior only
- Invalid config and invalid observation paths must preserve current `WatchdogError` values
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `runtime_guard_watchdog.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering mixed block/gossip sequencing, counter snapshots, single-recipient pass-through, and invalid config/observation failures via the public API.
3. Run targeted watchdog contract and integration tests.
4. Run the full `kamn-runtime-guards` crate tests.
5. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.
