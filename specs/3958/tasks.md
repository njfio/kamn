# Issue #3958 Tasks

- Issue: #3958
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing checker/docs assertions for quorum drift and signer-disagreement go/no-go marker fields.
- [x] T2 (Green): implement additive checker output markers derived from deterministic reason-code subsets.
- [x] T3 (Green): document markers in CI strategy and add docs contract assertion coverage.
- [x] T4 (Regression): run policy checker contract tests, docs contract tests, `cargo fmt --check`, and scoped lint/test verification.
- [ ] T5 (Verify): update issue/parent progress markers and close workflow metadata.

## Tier Mapping
- Unit: N/A (script contract verification is functional/integration scoped).
- Functional: checker GO fixture marker outputs.
- Integration: policy checker shell contract test lane.
- Regression: drift/disagreement NO-GO fixture marker assertions.
- Performance: N/A (existing checker/runtime budget unchanged).
