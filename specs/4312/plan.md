# Issue #4312 Plan

- Issue: `#4312`
- Status: `Completed`

## Approach
- Integrate websocket protocol drift/session violation conformance tests into `service_api_endpoint_tests`.
- Add deterministic protocol-session reason projection and docs-contract validation helpers in `service_api_endpoint`.
- Add/update service API contract and release checklist protocol/session marker docs plus parity tests.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/service/api-contract.md`
- `crates/kamn-core/tests/service_api_contract_docs.rs`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations
- Risk: reason-class mapping can drift across websocket/payload/auth categories.
- Mitigation: deterministic projection helper with integration/regression coverage on class outputs.
- Risk: docs markers can drift from checker expectations.
- Mitigation: docs-contract validator and docs parity tests in `kamn-core`.

## Interface Contract
- Additive service API helper surfaces for protocol-session reason taxonomy and docs-contract validation.

## ADR
- Not required.
