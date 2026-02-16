# Issue #4318 Plan

- Issue: `#4318`
- Status: `In Progress`

## Approach
- Add deterministic protocol/session reason projection types + helpers near `service_api_endpoint` reason-code constants.
- Add a docs-contract marker checker in `service_api_endpoint` that validates release-checklist parity and fails closed with deterministic reason code.
- Extend `service_api_endpoint_tests` with conformance-category tests for projection/docs parity.
- Update release checklist with protocol/session reason taxonomy section and guard it in docs tests.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations
- Risk: classification drift between reason codes and projection classes.
- Mitigation: explicit reason-class mapping + regression test for websocket reason class stability.
- Risk: docs markers drift unnoticed.
- Mitigation: docs-contract checker + release checklist docs test assertions.

## Interface Contract
- Additive internal API in `kamn-node::service_api_endpoint`; no protocol wire-format changes.

## ADR
- Not required.
