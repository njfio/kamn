# Tasks: Issue #5907 - Nonce Overflow Fail-Closed in kamn-agent-lib

1. T1 (RED): add failing tests for `NonceTracker` overflow and `AgentLib` overflow propagation.
2. T2 (GREEN): implement overflow-aware nonce advancement and deterministic error mapping.
3. T3 (REFACTOR): keep nonce API surface minimal while preserving monotonic behavior below overflow.
4. T4 (VERIFY): run fmt, strict clippy, and targeted `kamn-agent-lib` tests.
5. T5 (REGRESSION): ensure duplicate nonce saturation behavior cannot reappear unnoticed.
