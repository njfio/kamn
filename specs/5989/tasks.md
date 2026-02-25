# Tasks: Issue #5989

## Ordered Tasks
- T1 (Regression): Establish common harness and map existing constants/reason-codes into parameters.
- T2 (Implementation): Convert both duplicated scripts into thin wrappers around the common harness.
- T3 (Verification): Run both wrapper scripts and confirm behavior parity.
- T4 (Surface): Measure shell LOC delta for touched scripts and record in PR.

## Tier Mapping
- Unit: T1
- Functional: T2, T3
- Regression: T1, T3
- Performance: T4
- Conformance: T3, T4
