# Tasks: Issue #6140

- T1 (RED): Add failing structural conformance test for dispatcher decomposition contract.
- T2 (GREEN): Refactor `handle_service_api_http_route` to delegate into method-specific helpers.
- T3 (REGRESSION): Add regression test proving helper delegation markers and method-router contract.
- T4 (VERIFY): Run scoped `kamn-node` service API tests + formatting/lint checks.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T4
- Regression: T3, T4
- Conformance: T1, T3, T4
- Integration: T4 (existing service API behavior suite)
