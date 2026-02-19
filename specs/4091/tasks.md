# Issue #4091 Tasks

- Issue: #4091
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Ordered Tasks
- T1 (RED): add failing checker contract tests for fail-closed decisions and taxonomy marker expectations.
- T2 (GREEN): implement quota checker + taxonomy helpers in `kamn-core`.
- T3 (Regression): add CI strategy marker docs and docs-contract assertions.
- T4 (Verify): run targeted checker/fixture/docs tests plus fmt/clippy and mark spec `Implemented`.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | checker reason classification and taxonomy helper stability |
| Functional | fail-closed checker behavior across pass/fail quota inputs |
| Integration | checker + fixture-aligned case composition |
| Regression | taxonomy/docs marker drift assertions |
