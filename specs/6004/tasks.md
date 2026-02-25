# Tasks: Issue #6004

## Ordered Tasks
- T1 (RED): Add failing tests for escaped/nested `tools/call` params and malformed JSON parse error handling.
- T2 (Implementation): Refactor protocol request parsing to serde-backed decode helpers.
- T3 (Implementation): Refactor tool-call dispatch payload extraction to typed params map.
- T4 (GREEN): Run targeted `kamn-mcp-server` protocol tests.
- T5 (Regression): Run crate-level clippy/tests for mcp-server to confirm no behavior drift.

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T1, T3, T4
- Integration: T4
- Regression: T5
- Conformance: T4
