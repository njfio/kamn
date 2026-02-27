# Tasks: Issue #6011

## Ordered Tasks
- T1 (RED): Add production `expect()` surface policy test and verify it fails without fixtures.
- T2 (Implementation): Add baseline/threshold fixtures and test fixture schema validation.
- T3 (Implementation): Implement deterministic census + non-regression gate logic with reason codes.
- T4 (GREEN): Run targeted policy test and ensure pass with current baseline.
- T5 (Regression): Re-run adjacent `kamn-core` CI-contract tests to confirm no regressions.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T3, T4
- Regression: T5
- Conformance: T4
