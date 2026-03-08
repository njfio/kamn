# 6618-add-runtime-guard-quota-policy-coverage

## Objective
Add dedicated crate-level integration coverage for the public `kamn_runtime_guards::quota_policy` boundary so quota-policy behavior remains pinned outside inline module tests.

## Inputs/Outputs
- Inputs:
  - `QuotaPolicyInput` values covering all allowed scope classes and fail-closed invalid cases
  - public quota-policy helper functions for taxonomy markers
- Outputs:
  - dedicated integration test surface at `crates/kamn-runtime-guards/tests/runtime_guard_quota_policy.rs`
  - dedicated contract test at `crates/kamn-runtime-guards/tests/runtime_guard_quota_policy_contract.rs`
  - refreshed `test_file_size_policy` baseline if a new test target changes the workspace inventory

## Boundaries/Non-goals
- No production behavior change in `crates/kamn-runtime-guards/src/quota_policy.rs`
- No changes to `policy_stack`, `anti_spam`, `watchdog`, or any other runtime guard module
- No CI or workflow changes
- No mutation of quota-policy input values inside evaluation logic

## Failure modes
- Integration surface missing entirely
- Public helper markers drift from expected deterministic values
- Unknown scope stops failing closed
- Zero window stops failing closed
- Zero limit stops failing closed
- `observed_count > limit` stops failing closed
- boundary case `observed_count == limit` stops allowing
- evaluation mutates caller-provided input
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `runtime_guard_quota_policy_contract` fails when the dedicated integration surface or its coverage markers disappear
- [ ] integration coverage exercises all allowed scope classes through the public API and receives `Allow`
- [ ] integration coverage receives deterministic reject reasons for unknown scope, zero window, zero limit, and exceeded limit
- [ ] integration coverage receives `Allow` for `observed_count == limit`
- [ ] integration coverage proves `evaluate_quota_policy` does not mutate `QuotaPolicyInput`
- [ ] integration coverage pins quota-policy helper marker strings through the public API
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_quota_policy_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_quota_policy -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6618-add-runtime-guard-quota-policy-coverage.md`
- `crates/kamn-runtime-guards/tests/runtime_guard_quota_policy.rs`
- `crates/kamn-runtime-guards/tests/runtime_guard_quota_policy_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public fail-closed contract only
- Reject decisions must preserve the current `QuotaPolicyViolationReason` values
- No new error types or production error translation changes are introduced

## Test plan
1. Add a contract test that references the dedicated integration surface and expected marker test names before the integration file exists to force a red failure.
2. Add the integration surface with public-boundary coverage for allowed scopes, invalid scope/window/limit/exceeded cases, boundary allow, input immutability, and deterministic helper markers.
3. Run targeted contract and integration tests.
4. Run the full `kamn-runtime-guards` crate tests.
5. Run `test_file_size_policy` and refresh the baseline only if the new test targets change inventory counts.

## Phase 6 notes
- No production quota-policy behavior changes were required.
- Integration for this issue is the dedicated crate-level test surface plus the contract pin that prevents silent removal of that surface.
- Adding the two new test targets increased the workspace `test_file_total` baseline from `469` to `471`.
