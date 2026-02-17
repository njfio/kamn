# Plan — #4257 Finality Taxonomy/Runbook Red Tests

Status: Reviewed

## Approach

- Extend policy checker script tests with taxonomy drift and runbook divergence tamper fixtures.
- Extend contract lane tests with new marker assertions and runbook parity failure checks.
- Ensure repeated tamper invocations produce stable fail-closed reason outputs.

## Risks

- Test brittleness against unrelated doc rewording.
  Mitigation: assert deterministic marker tokens only.
