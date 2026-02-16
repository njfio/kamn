# Issue #4317 Plan

- Issue: `#4317`
- Status: `Completed`

## Approach
- Add red tests in `service_api_endpoint_tests` covering protocol-drift detection and invalid session-frame rejection across unit/functional/integration/regression/performance categories.
- Add small test-only websocket checker helpers in `service_api_endpoint` for deterministic protocol-drift and frame-validation reason classification.
- Add the required docs contract file `docs/service/api-contract.md` with invalid-frame handling matrix and reason taxonomy markers.
- Add docs guard test in `kamn-core` to fail closed if required docs markers drift.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/service/api-contract.md`
- `crates/kamn-core/tests/service_api_contract_docs.rs`

## Risks and Mitigations
- Risk: overlap/drift with existing websocket helper assertions in tests.
- Mitigation: keep checker functions deterministic and single-purpose; wire tests to explicit reason codes.
- Risk: docs path did not previously exist and could drift silently.
- Mitigation: add dedicated docs guard test for required matrix/taxonomy markers.

## Interface Contract
- Additive test-only API surface in `service_api_endpoint`; no external protocol wire-format changes.

## ADR
- Not required.
