# Plan: Issue #6011

## Approach
1. Add baseline + threshold fixtures under `fixtures/ci/` and `.ci/` with schema markers.
2. Add a new contract test in `crates/kamn-core/tests/` that:
   - collects tracked Rust source files in `crates/*/src/`,
   - excludes known test-only source file patterns,
   - counts `.expect(` occurrences,
   - enforces non-regression against baseline + threshold.
3. Validate fixtures, compute live count, and fail closed with deterministic reason codes when policy is violated.
4. Re-run targeted gate test and update baseline to current measured value.

## Affected Modules
- `crates/kamn-core/tests/production_expect_surface_policy.rs` (new)
- `fixtures/ci/production_expect_surface_baseline.env` (new)
- `.ci/production_expect_surface_thresholds.env` (new)

## Risks / Mitigations
- Risk: false positives from test code colocated in `src`.
  Mitigation: exclude common test-only filename patterns in scope rules and document this contract explicitly.
- Risk: noisy baseline churn.
  Mitigation: strict delta threshold (`0`) and explicit fixture refresh workflow.

## Interfaces / Contracts
- No runtime API changes.
- New CI contract surface for panic-risk budgeting.
