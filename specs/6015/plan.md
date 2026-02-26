# Plan: Issue #6015

## Approach
1. Refactor census path selection into explicit production-scope filter.
2. Add `.expect(` counter that skips `#[cfg(test)]` items in source files.
3. Add a focused regression test for cfg(test) exclusion logic.
4. Recompute and update baseline fixture values from corrected census.
5. Run targeted policy test to verify green.

## Affected Modules
- `crates/kamn-core/tests/production_expect_surface_policy.rs`
- `fixtures/ci/production_expect_surface_baseline.env`

## Risks / Mitigations
- Risk: Over-filtering could hide true production `expect()` usage.
  Mitigation: use explicit, documented filename/path rules and unit regression coverage.
- Risk: Parser edge cases around attributes/braces.
  Mitigation: fail-safe deterministic line-based parser plus regression test fixture.

## Interfaces / Contracts
- No runtime/API changes.
- CI policy scope is corrected for audit accuracy.
