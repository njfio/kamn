# Plan — #4396

Status: Reviewed

## Approach

- Extend `scripts/runtime/test_validate_persistence_adapters_live.sh` with new required marker assertions.
- Add JSON contract assertions for taxonomy version, reason-code CSV, tamper/freshness and completeness markers.
- Add deterministic negative-path checks by tampering generated report values and expecting fail-closed mismatch.

## Risks and Mitigations

- Risk: test brittleness from non-deterministic error output.
  - Mitigation: assert exact deterministic mismatch reason markers.

## Validation

- Run target test script until RED failures reproduce.
