# Issue #5101 Plan

- Issue: #5101
- Status: Implemented

## Approach
1. Add additive M9 projection input/output contracts for runtime backpressure evaluation.
2. Build projection path from M9 recipient queue snapshot into `RuntimeBackpressureInput`.
3. Evaluate projection via `DeterministicBackpressureController` and return runtime decision plus M9 reason markers.
4. Add explicit M9 fail-closed error mapping for invalid projection input.
5. Add RED tests for action mapping and invalid input, then implement and run full regression/guardrails.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Bridge could alter existing dispatch queue behavior.
  - Error taxonomy could leak runtime error details inconsistently.
- Mitigations:
  - Keep existing dispatch APIs unchanged; add additive bridge only.
  - Map runtime input/validation errors into deterministic M9 projection error variants.
  - Preserve existing M9 tests and run full regression suite.

## Interface Contract
- Additive M9 projection contracts and method.
- No protocol/wire format changes.
- No new dependencies.

## ADR
- Not required for this scoped integration task.
