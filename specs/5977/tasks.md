# Tasks: Issue #5977

## Ordered Tasks
- T1 (Conformance): Add failing tests expecting cryptographic transport signatures.
- T2 (Implementation): Replace SDK deterministic signing path in production send APIs.
- T3 (Implementation): Replace agent-lib deterministic signing path.
- T4 (Regression): Add tamper/replay/wrong-key/malformed auth integration matrix.
- T5 (Verification): Run scoped fmt/clippy/tests and map AC coverage.

## Tier Mapping
- Unit: T2, T3
- Functional: T2, T3
- Integration: T4
- Regression: T4
- Conformance: T1, T5
