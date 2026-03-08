# 6580-fix-production-expect-cfg-test-census

## Objective
Fix `production_expect_surface_policy` so its production-source census excludes inline `#[cfg(test)]` blocks reliably, including cases where nested functions inside the test module open and close braces before the enclosing module ends.

## Inputs/Outputs
- Input: Rust source text from tracked `crates/**/src/**/*.rs` files under the existing census policy.
- Output: Deterministic `production_expect_count` used by `functional_production_expect_surface_non_regression_gate`.

## Boundaries/Non-goals
- No baseline bump in `fixtures/ci/production_expect_surface_baseline.env`.
- No threshold changes in `.ci/production_expect_surface_thresholds.env`.
- No changes to governance ratio or unrelated CI policy.
- No production-code behavior changes outside the test census implementation.

## Failure modes
- Inline `#[cfg(test)] mod tests { ... }` content is counted as production surface.
- Nested braces inside a skipped `#[cfg(test)]` item cause the skip to terminate before the enclosing item closes.
- Character literals containing `"` inside skipped test code incorrectly open a string state and leak test content back into the census.
- Real production-reachable `.expect(` calls outside skipped test-only scopes stop being counted.

## Acceptance criteria
- [x] A regression test with a minimal inline `#[cfg(test)] mod tests` fixture fails on the current logic and passes after the fix.
- [x] `functional_production_expect_surface_non_regression_gate` passes on current `main` without changing the production-expect baseline fixture.
- [x] Real production `.expect(` occurrences outside test-only scopes are still counted by the census logic.
- [x] The fix is limited to the census logic/tests in `production_expect_surface_policy`.

## Files to touch
- `crates/kamn-core/tests/production_expect_surface_policy.rs`
- `crates/kamn-core/tests/support/production_expect_surface_policy_support.rs`
- `specs/6580-fix-production-expect-cfg-test-census.md`

## Error semantics
- The policy test must continue to fail with deterministic reason codes when real production `.expect(` debt exceeds the baseline.
- The regression test must fail loudly if the census reintroduces false positives from inline `#[cfg(test)]` blocks.

## Test plan
- Add a minimal regression test that demonstrates nested braces inside `#[cfg(test)] mod tests` must remain excluded.
- Add a regression test that demonstrates `if ch == '"'` inside skipped test code does not open a string state.
- Run `cargo test -p kamn-core --test production_expect_surface_policy -- --nocapture`.
- Run `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`.
- Run `cargo clippy -p kamn-core --tests -- -D warnings`.

## Integration verification
- The integrated path is the same CI policy test target that failed in `Workspace Pre-Merge Gate (PR)`: `cargo test -p kamn-core --test production_expect_surface_policy -- --nocapture`.
- The branch-specific follow-up verification also includes `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` so the refactor stays under the workspace oversized-test budget without changing `fixtures/ci/test_file_size_policy_baseline.env`.

## Verification actuals
- Red: `cargo test -p kamn-core --test production_expect_surface_policy -- --nocapture`
  - `regression_nested_cfg_test_module_expect_calls_are_excluded_from_census` failed with `left: 2 right: 1`
- Green: `cargo test -p kamn-core --test production_expect_surface_policy -- --nocapture`
- Green: `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
- Green: `cargo clippy -p kamn-core --tests -- -D warnings`

## Deviations
- The green implementation initially pushed `crates/kamn-core/tests/production_expect_surface_policy.rs` over the soft test-file budget. The final refactor extracted scanner helpers into `crates/kamn-core/tests/support/production_expect_surface_policy_support.rs` so the policy fix ships without a baseline bump.
