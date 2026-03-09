# 6636-add-deterministic-proposal-planner-integration-coverage

## Objective
Add dedicated crate-level integration coverage for the public deterministic proposal planner API in `runtime_peer_coordination.rs` so candidate validation, deterministic ordering, duplicate-id rejection, and stale-state fail-closed behavior remain pinned outside inline module coverage.

## Inputs/Outputs
- Inputs:
  - public `ProposalCandidate` construction
  - public `DeterministicProposalPlanner::plan(...)`
  - public `ProposalPlan` ordered output helpers
- Outputs:
  - dedicated integration test surface at `crates/kamn-core/tests/deterministic_proposal_planner_integration.rs`
  - dedicated contract test at `crates/kamn-core/tests/deterministic_proposal_planner_contract.rs`
  - refreshed `test_file_size_policy` baseline if the new test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-core/src/runtime_peer_coordination.rs`
- No runtime wiring or transport profile coverage in this issue
- No CI or workflow changes
- No visibility changes for internal helpers solely to support tests

## Failure modes
- dedicated deterministic-proposal-planner integration surface missing entirely
- valid planning stops returning deterministic nonce/sender/id ordering
- invalid candidate id, sender did, state hash, or nonce stop failing closed
- duplicate candidate ids stop failing closed
- stale state hash stops failing closed
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `deterministic_proposal_planner_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage asserts valid planning returns deterministic ordering by nonce, sender DID, and candidate id
- [ ] integration coverage asserts invalid candidate constructor inputs fail closed with deterministic public error variants
- [ ] integration coverage asserts duplicate candidate ids and stale state hashes fail closed through `DeterministicProposalPlanner::plan(...)`
- [ ] `cargo test -p kamn-core --test deterministic_proposal_planner_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test deterministic_proposal_planner_integration -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6636-add-deterministic-proposal-planner-integration-coverage.md`
- `crates/kamn-core/tests/deterministic_proposal_planner_integration.rs`
- `crates/kamn-core/tests/deterministic_proposal_planner_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public fail-closed behavior only
- Invalid constructor and planning paths must preserve current public enum variants
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `deterministic_proposal_planner_integration.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering one valid deterministic planning path and the public fail-closed constructor, duplicate-id, and stale-state paths.
3. Run targeted contract and integration tests.
4. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.

## Deviations
- The workspace `test_file_size_policy` inventory changed from `487` to `489`, so `fixtures/ci/test_file_size_policy_baseline.env` was refreshed during integration.

## Phase 6 Evidence
- `cargo test -p kamn-core --test deterministic_proposal_planner_contract -- --nocapture`
- `cargo test -p kamn-core --test deterministic_proposal_planner_integration -- --nocapture`
- `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
- `cargo clippy -p kamn-core --tests -- -D warnings`
