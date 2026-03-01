# Issue 6281 - Runtime-guards policy-stack integration API and contract tests

## Objective
Add a crate-level policy-stack API for `kamn-runtime-guards` and an integration
test suite that verifies deterministic guard precedence across anti-spam, quota,
and fairness modules.

## Inputs/Outputs
- Inputs:
  - Existing guard modules in `crates/kamn-runtime-guards/src`:
    - `anti_spam`
    - `quota_policy`
    - `fairness_policy`
- Outputs:
  - New public API for evaluating guard stack in deterministic order.
  - New integration tests in `crates/kamn-runtime-guards/tests/` that exercise
    allow/reject precedence with real module interactions.

## Boundaries/Non-goals
- In scope:
  - Public policy-stack API and exports.
  - Integration tests for precedence behavior.
- Out of scope:
  - Changing per-module validation semantics.
  - Wiring new API into `kamn-core`.
  - New dependencies.

## Failure modes
- FM1: Guard evaluation order drifts (quota/fairness evaluated before anti-spam).
- FM2: Rejection precedence is ambiguous when multiple policies would reject.
- FM3: Integration path is untested, allowing future cross-module regressions.

## Acceptance criteria (testable booleans)
- AC1: `kamn-runtime-guards` exposes a public policy-stack API that evaluates
  `anti-spam -> quota -> fairness` in order.
- AC2: `crates/kamn-runtime-guards/tests/runtime_guard_policy_stack.rs` contains
  integration tests for:
  - allow path
  - anti-spam precedence
  - quota precedence after anti-spam allow
  - fairness rejection after anti-spam/quota allow
- AC3: `cargo test -p kamn-runtime-guards --test runtime_guard_policy_stack`
  passes.
- AC4: `cargo test -p kamn-runtime-guards` passes.

## Files to touch
- `crates/kamn-runtime-guards/src/lib.rs`
- `crates/kamn-runtime-guards/src/policy_stack.rs` (new)
- `crates/kamn-runtime-guards/tests/runtime_guard_policy_stack.rs` (new)

## Error semantics
- Invalid anti-spam input/config continues to return `AntiSpamError` directly.
- Policy rejections are explicit typed outcomes; no silent fallback.

## Test plan
- RED:
  - Add integration tests referencing new API and expected precedence.
  - Confirm compile/test failure before API implementation.
- GREEN:
  - Implement policy-stack API and exports.
  - Re-run targeted integration tests.
- REFACTOR:
  - Keep helper builders small and deterministic.
- INTEGRATION:
  - Run full crate test suite to verify no regressions.

## Deviations
- None.
