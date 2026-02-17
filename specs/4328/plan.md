# Plan — #4328

Status: Reviewed

## Approach

- Add a deterministic payload contract checker in `observability_endpoint.rs` keyed by endpoint surface.
- Validate rendered payloads before endpoint response return; if validation fails, return fail-closed response envelope with stable taxonomy markers.
- Extend tests to encode RED drift/missing-field cases and fail-closed reason-code behavior.
- Update docs and docs-contract tests to keep policy text synchronized with checker taxonomy.

## Affected Areas

- `crates/kamn-node/src/observability_endpoint.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs`
- `docs/observability/schema.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/observability_schema_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: checker integration changes endpoint response behavior.
  - Mitigation: keep happy-path outputs unchanged; only fail-closed on explicit contract violation.
- Risk: docs tests become stale.
  - Mitigation: add exact deterministic markers and keep taxonomy constants stable.

## Interfaces and Contracts

- New deterministic checker reason taxonomy version constant.
- Reason formats:
  - `runtime_observability_policy_required_field_missing:<surface>.<field>`
  - `runtime_observability_policy_schema_drift:<surface>.schema_version`
- Fail-closed envelope carries schema version, status, final decision, reason taxonomy version, reason code.
