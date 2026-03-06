## Objective
Add explicit coverage for `evaluate_runtime_guard_policy_stack()` propagating
`AntiSpamError::InvalidInput(..)` and pin that coverage with a small contract test.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-runtime-guards/src/policy_stack.rs`
  - `crates/kamn-runtime-guards/tests/runtime_guard_policy_stack.rs`
- Outputs:
  - a focused runtime-guards test covering anti-spam invalid-input error propagation
  - a contract test that requires the dedicated error-propagation test surface to exist

## Boundaries/Non-goals
- Do not change policy-stack evaluation ordering.
- Do not change anti-spam, quota, or fairness production behavior unless red-phase investigation
  proves propagation is currently broken.
- Do not add dependencies or modify CI/workflow surfaces.

## Failure modes
- `evaluate_runtime_guard_policy_stack()` swallows or remaps `AntiSpamError::InvalidInput(..)`.
- Coverage for the error-propagation path can disappear without a failing contract.
- Existing allow/reject paths regress while adding the new test coverage.

## Acceptance criteria
- [ ] A dedicated runtime-guards test asserts `AntiSpamError::InvalidInput(..)` propagates from
      `evaluate_runtime_guard_policy_stack()`.
- [ ] A contract test fails if that dedicated test surface is removed.
- [ ] Existing allow and reject policy-stack behavior remains unchanged.
- [ ] Focused runtime-guards tests pass locally.

## Files to touch
- `crates/kamn-runtime-guards/tests/runtime_guard_policy_stack.rs`
- `crates/kamn-runtime-guards/tests/runtime_guard_policy_stack_contract.rs`
- `specs/6518-policy-stack-error-propagation-coverage.md`

## Error semantics
- Invalid anti-spam inputs must remain fail-closed and typed as `AntiSpamError::InvalidInput(..)`.
- No swallowing, fallback, or remapping to `RuntimeGuardPolicyDecision::Reject` is allowed for this
  path.

## Test plan
- Red:
  - add a contract test that requires a named error-propagation regression in
    `runtime_guard_policy_stack.rs`
  - run the contract test and confirm it fails before the regression exists
- Green:
  - add the dedicated error-propagation regression
  - `cargo test -p kamn-runtime-guards --test runtime_guard_policy_stack_contract -- --nocapture`
  - `cargo test -p kamn-runtime-guards --test runtime_guard_policy_stack -- --nocapture`
- Refactor:
  - rerun the focused runtime-guards tests after cleanup

## Deviations
- Pending red-phase investigation may show the production behavior already exists and only the
  coverage artifact is missing. If so, no production code change is expected.

## Execution Evidence
- Pending.
