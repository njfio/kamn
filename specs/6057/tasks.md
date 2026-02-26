# Tasks: Issue #6057

## Ordered Tasks
- T1 (RED): Add failing SDK service tests for endpoint-path and header-value injection rejection.
- T2 (GREEN): Add parse-time endpoint base-path validation and satisfy new tests.
- T3 (GREEN): Ensure route segment and auth validation contracts remain deterministic.
- T4 (VERIFY): Run targeted `kamn-sdk` format/lint/test commands and collect results.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T1, T2
- Conformance: T1, T4
- Integration: N/A (module-local validation hardening)
- Regression: T3, T4
- Performance: N/A (validation-only)
