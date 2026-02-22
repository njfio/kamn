# Issue #5543 Plan - Service API Scope-Policy Fixture Exclusive Allow-Only/Deny-Only Route Coverage Metrics Exposure

## Approach
1. Extend fixture projection parser to derive allow-route and deny-route sets and compute exclusive set-difference counts.
2. Extend service-api snapshot with exclusive route-count fields.
3. Extend `/metrics` payload rendering with two new markers.
4. Extend existing endpoint metrics assertions across four lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/scope_fixture.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks and Mitigations
- Risk: value drift from canonical fixture semantics.
  - Mitigation: derive expected values directly from parsed fixture rows using set differences in each lane.
- Risk: collateral behavior drift.
  - Mitigation: run scoped service-api suite + extraction contract + fmt + clippy gates.

## Interfaces / Contracts
- New snapshot fields:
  - `scope_policy_fixture_unique_allow_only_route_count: usize`
  - `scope_policy_fixture_unique_deny_only_route_count: usize`
- New metrics markers:
  - `kamn_service_api_scope_policy_fixture_unique_allow_only_route_count`
  - `kamn_service_api_scope_policy_fixture_unique_deny_only_route_count`

## ADR
- Not required.
