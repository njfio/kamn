# Issue #5541 Plan - Service API Scope-Policy Fixture Allow/Deny Overlap Route Coverage Metrics Exposure

## Approach
1. Extend fixture projection parser to track unique allow routes and deny routes, then derive overlap route count via deterministic set intersection cardinality.
2. Extend service-api snapshot model and builder to carry the overlap route count.
3. Extend `/metrics` renderer to emit `kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count`.
4. Extend existing endpoint metric lane assertions in functional, HTTP integration, TLS integration, and unit observability tests.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/scope_fixture.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks and Mitigations
- Risk: Counter derivation drift from canonical fixture semantics.
  - Mitigation: derive expected overlap count in tests directly from parsed fixture rows and compare with emitted marker.
- Risk: accidental endpoint behavior change beyond observability marker.
  - Mitigation: run scoped service API endpoint suite and module extraction line-budget conformance test.

## Interfaces / Contracts
- Add snapshot field: `scope_policy_fixture_unique_allow_deny_overlap_route_count: usize`.
- Add metrics marker: `kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count <n>`.

## ADR
- Not required: no architecture or dependency decision change.
