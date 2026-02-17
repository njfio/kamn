# Plan — #4334

Status: Reviewed

## Approach

- Add targeted unit/functional regression tests in `observability_endpoint_tests.rs` that call checker hooks with tampered payloads.
- Assert expected reason code and deterministic taxonomy markers.
- Execute test lane to capture RED output before implementing checker logic.

## Risks and Mitigations

- Risk: tests depend on brittle string formatting.
  - Mitigation: assert required markers and explicit reason-code prefixes.

## Interfaces and Contracts

- Test contract expects reason codes:
  - `runtime_observability_policy_required_field_missing:<surface>.<field>`
  - `runtime_observability_policy_schema_drift:<surface>.schema_version`
