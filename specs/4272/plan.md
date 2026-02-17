# Plan — #4272 Red Drift/Divergence Tests

Status: Reviewed

## Approach

1. Add runbook-override test fixtures in service api axum contract lane test.
2. Introduce one fixture for taxonomy value drift and one for marker divergence.
3. Assert deterministic reason outputs for each failure path.

## Risks

- False positives from unrelated doc formatting changes.
  - Mitigation: mutate only deterministic marker lines in fixtures.
