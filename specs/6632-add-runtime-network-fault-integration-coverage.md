# 6632-add-runtime-network-fault-integration-coverage

## Objective
Add dedicated crate-level integration coverage for the public `runtime_network_fault` simulation API so deterministic lifecycle, queue, backpressure, and watchdog behavior remain pinned outside inline module coverage.

## Inputs/Outputs
- Inputs:
  - public `NetworkFaultSimulationInput` values
  - public `DeterministicNetworkFaultSimulator` and `simulate_daemon_network_fault(...)`
- Outputs:
  - dedicated integration test surface at `crates/kamn-core/tests/runtime_network_fault_integration.rs`
  - dedicated contract test at `crates/kamn-core/tests/runtime_network_fault_contract.rs`
  - refreshed `test_file_size_policy` baseline if the new test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-core/src/runtime_network_fault.rs`
- No runtime wiring changes outside the public simulation entrypoints
- No CI or workflow changes
- No visibility changes for internal helpers solely to support tests

## Failure modes
- dedicated runtime-network-fault integration surface missing entirely
- valid public simulation no longer returns deterministic lifecycle, queue, backpressure, and watchdog outputs
- invalid sample id or invalid peer id stop failing closed
- invalid queue capacity stops failing closed
- invalid watchdog input stops failing closed through the public constructor path
- daemon helper stops delegating to the same public simulation output
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `runtime_network_fault_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage asserts a valid public simulation returns deterministic lifecycle state, queue counts, and watchdog markers
- [ ] integration coverage asserts invalid constructor inputs fail closed with deterministic public error variants and reason codes
- [ ] integration coverage asserts `simulate_daemon_network_fault(...)` matches the direct simulator output for the same input
- [ ] `cargo test -p kamn-core --test runtime_network_fault_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test runtime_network_fault_integration -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6632-add-runtime-network-fault-integration-coverage.md`
- `crates/kamn-core/tests/runtime_network_fault_integration.rs`
- `crates/kamn-core/tests/runtime_network_fault_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public fail-closed behavior only
- Invalid constructor and simulation inputs must preserve current public enum variants and `reason_code()` markers where exposed
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `runtime_network_fault_integration.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering one valid simulation path, constructor fail-closed paths, and daemon-helper parity through the public API.
3. Run targeted contract and integration tests.
4. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.

## Deviations
- The workspace `test_file_size_policy` inventory changed from `483` to `485`, so `fixtures/ci/test_file_size_policy_baseline.env` was refreshed during integration.

## Phase 6 Evidence
- `cargo test -p kamn-core --test runtime_network_fault_contract -- --nocapture`
- `cargo test -p kamn-core --test runtime_network_fault_integration -- --nocapture`
- `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
- `cargo clippy -p kamn-core --tests -- -D warnings`
