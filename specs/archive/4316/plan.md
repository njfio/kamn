# Issue #4316 Plan

- Issue: `#4316`
- Status: `Completed`

## Approach
- Add a shared lifecycle-limiter rejection projection mapping in `service_api_endpoint` keyed by reason code.
- Use the projection mapping for concurrency-limit, ingress-rate-limit, and sender anti-spam limiter rejection responses in middleware.
- Expose test-only projection helpers for conformance testing in `service_api_endpoint_tests`.
- Add lifecycle rejection taxonomy docs and a docs parity test in `kamn-core`.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/service/api-contract.md`
- `crates/kamn-core/tests/service_api_lifecycle_contract_docs.rs`

## Risks and Mitigations
- Risk: changing middleware projection could unintentionally alter existing status/outcome behavior.
- Mitigation: map reason codes to existing status/error/outcome tuples and enforce with integration/regression tests.
- Risk: docs marker drift for lifecycle taxonomy.
- Mitigation: dedicated docs test with explicit marker assertions.

## Interface Contract
- Internal additive projection mapping API; no public wire-format changes.

## ADR
- Not required.
