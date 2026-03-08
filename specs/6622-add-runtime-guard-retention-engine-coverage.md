# 6622-add-runtime-guard-retention-engine-coverage

## Objective
Add dedicated crate-level integration coverage for the public `kamn_runtime_guards::retention_engine` boundary so retention evaluation, resurface blocking, and status projection remain pinned outside inline module tests.

## Inputs/Outputs
- Inputs:
  - public `RetentionEnginePolicy`, `RetentionPolicyEngine`, `RetentionRecord`, and `RetentionPolicyCheckerInput` values covering valid and fail-closed cases
  - mixed-domain retention records and repeated evaluation cycles
- Outputs:
  - dedicated integration test surface at `crates/kamn-runtime-guards/tests/runtime_guard_retention_engine.rs`
  - dedicated contract test at `crates/kamn-runtime-guards/tests/runtime_guard_retention_engine_contract.rs`
  - refreshed `test_file_size_policy` baseline if the new test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-runtime-guards/src/retention_engine.rs`
- No changes to anti-spam, quota, fairness, watchdog, or policy-stack modules
- No CI or workflow changes
- No visibility changes for internal helpers solely to support tests

## Failure modes
- Dedicated retention-engine integration surface missing entirely
- public checker stops failing closed for unknown domain, zero window, or expired record age
- domain override status projection stops using the correct retention class or expiration timestamp
- expired records stop surfacing in deterministic sorted order
- resurfaced expired records stop failing closed on later evaluations
- invalid class or empty record id stop failing closed
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `runtime_guard_retention_engine_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage asserts public checker decisions for unknown domain, zero window, expired age, and boundary allow cases
- [ ] integration coverage asserts `status_for()` uses default and per-domain override classes and computes deterministic expiration timestamps
- [ ] integration coverage asserts `evaluate()` returns deterministic expired ids across a mixed-domain batch
- [ ] integration coverage asserts resurfaced expired records fail closed on later evaluations
- [ ] integration coverage asserts invalid class and empty record id fail closed through the public API
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_retention_engine_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_retention_engine -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6622-add-runtime-guard-retention-engine-coverage.md`
- `crates/kamn-runtime-guards/tests/runtime_guard_retention_engine.rs`
- `crates/kamn-runtime-guards/tests/runtime_guard_retention_engine_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public retention-engine fail-closed behavior only
- Invalid retention class, empty record id, and resurfaced expired record paths must preserve current `RetentionPolicyError` values
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `runtime_guard_retention_engine.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering checker decisions, status projection, deterministic batch expiration, resurfaced-record blocking, and invalid input/config failures via the public API.
3. Run targeted retention-engine contract and integration tests.
4. Run the full `kamn-runtime-guards` crate tests.
5. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.

## Phase 6 notes
- No production retention-engine behavior changes were required.
- Integration for this issue is the dedicated crate-level retention-engine surface plus the contract pin that prevents silent removal of that surface.
- Adding the two new test targets increased the workspace `test_file_total` baseline from `473` to `475`.
