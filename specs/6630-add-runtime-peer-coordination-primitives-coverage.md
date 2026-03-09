# 6630-add-runtime-peer-coordination-primitives-coverage

## Objective
Add dedicated crate-level integration coverage for the public `PeerLifecycle` and `BoundedRuntimeQueue` primitives in `runtime_peer_coordination.rs` so lifecycle transitions and queue semantics remain pinned outside inline module coverage.

## Inputs/Outputs
- Inputs:
  - public `PeerLifecycle`, `PeerLifecycleEvent`, `PeerLifecycleState`, and `BoundedRuntimeQueue` values
- Outputs:
  - dedicated integration test surface at `crates/kamn-core/tests/runtime_peer_coordination_primitives.rs`
  - dedicated contract test at `crates/kamn-core/tests/runtime_peer_coordination_primitives_contract.rs`
  - refreshed `test_file_size_policy` baseline if the new test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-core/src/runtime_peer_coordination.rs`
- No authenticated frame, proposal planner, or runtime wiring coverage in this issue
- No CI or workflow changes
- No visibility changes for internal helpers solely to support tests

## Failure modes
- Dedicated coordination-primitives integration surface missing entirely
- valid lifecycle transitions stop producing expected states
- invalid lifecycle transitions stop failing closed with deterministic reason codes
- queue FIFO behavior regresses
- queue overflow stops failing closed
- zero-capacity queue construction stops failing closed
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `runtime_peer_coordination_primitives_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage asserts a valid peer lifecycle sequence reaches `Connecting`, `Active`, `Degraded`, and back to `Disconnected`
- [ ] integration coverage asserts invalid lifecycle transitions fail closed and preserve `reason_code()` markers
- [ ] integration coverage asserts queue FIFO ordering via enqueue/dequeue
- [ ] integration coverage asserts overflow and invalid-capacity queue creation fail closed with deterministic error variants and reason codes
- [ ] `cargo test -p kamn-core --test runtime_peer_coordination_primitives_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test runtime_peer_coordination_primitives -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6630-add-runtime-peer-coordination-primitives-coverage.md`
- `crates/kamn-core/tests/runtime_peer_coordination_primitives.rs`
- `crates/kamn-core/tests/runtime_peer_coordination_primitives_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public peer-lifecycle and queue fail-closed behavior only
- Invalid transition and queue error paths must preserve current public enum values and reason codes
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `runtime_peer_coordination_primitives.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering lifecycle progression, invalid transitions, FIFO queue behavior, overflow rejection, and invalid-capacity rejection via the public API.
3. Run targeted contract and integration tests.
4. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.

## Deviations
- The workspace `test_file_size_policy` inventory changed from `481` to `483`, so `fixtures/ci/test_file_size_policy_baseline.env` was refreshed during integration.

## Phase 6 Evidence
- `cargo test -p kamn-core --test runtime_peer_coordination_primitives_contract -- --nocapture`
- `cargo test -p kamn-core --test runtime_peer_coordination_primitives -- --nocapture`
- `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
- `cargo clippy -p kamn-core --tests -- -D warnings`
