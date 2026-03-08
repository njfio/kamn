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
- Real production-reachable `.expect(` calls outside skipped test-only scopes stop being counted.

## Acceptance criteria
- [ ] A regression test with a minimal inline `#[cfg(test)] mod tests` fixture fails on the current logic and passes after the fix.
- [ ] `functional_production_expect_surface_non_regression_gate` passes on current `main` without changing the production-expect baseline fixture.
- [ ] Real production `.expect(` occurrences outside test-only scopes are still counted by the census logic.
- [ ] The fix is limited to the census logic/tests in `production_expect_surface_policy`.

## Files to touch
- `crates/kamn-core/tests/production_expect_surface_policy.rs`
- `specs/6580-fix-production-expect-cfg-test-census.md`

## Error semantics
- The policy test must continue to fail with deterministic reason codes when real production `.expect(` debt exceeds the baseline.
- The regression test must fail loudly if the census reintroduces false positives from inline `#[cfg(test)]` blocks.

## Test plan
- Add a minimal regression test that demonstrates nested braces inside `#[cfg(test)] mod tests` must remain excluded.
- Run `cargo test -p kamn-core --test production_expect_surface_policy -- --nocapture`.
- Re-run the same test after refactor/lint verification.

## Deviations
- None.
