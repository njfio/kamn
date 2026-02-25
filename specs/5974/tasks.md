# Tasks: Issue #5974

## Ordered Tasks
- T1 (Conformance): Add failing tests for cryptographic-only production transport auth.
- T2 (Implementation): Replace deterministic signing in production request constructors.
- T3 (Implementation): Wire verification path and compatibility gating.
- T4 (Regression): Add tamper/replay/wrong-key integration regressions.
- T5 (Verification): Run scoped sdk/agent/node checks and map AC -> tests.

## Tier Mapping
- Unit: T2, T3
- Functional: T2, T3
- Integration: T4
- Regression: T4
- Conformance: T1, T5
