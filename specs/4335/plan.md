# Plan — #4335

Status: Reviewed

## Approach

- Introduce endpoint-surface classifier and required-field matrix in `observability_endpoint.rs`.
- Add `validate_observability_endpoint_payload_contract` checker returning deterministic reason strings.
- Route endpoint responses through a fail-closed builder if checker returns violation.
- Preserve current success payload shape and status for valid contracts.

## Risks and Mitigations

- Risk: unintentional behavior change for healthy responses.
  - Mitigation: integration tests assert unchanged happy-path response status/body markers.
- Risk: unstable reason code outputs.
  - Mitigation: reason strings built from fixed constants and deterministic formatting.

## Interfaces and Contracts

- Taxonomy version: `kamn.runtime.observability-endpoint-reason-taxonomy.v1`.
- Fail-closed envelope schema: `kamn.runtime.observability.endpoint-fail-closed.v1`.
- Deterministic reason formats:
  - `runtime_observability_policy_required_field_missing:<surface>.<field>`
  - `runtime_observability_policy_schema_drift:<surface>.schema_version`
