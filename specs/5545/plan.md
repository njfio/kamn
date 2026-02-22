# Issue #5545 Plan - Service API Scope-Policy Fixture Scope Overlap and Exclusive Coverage Metrics Exposure

## Approach
1. Extend fixture projection parser with scope-set arithmetic:
   - overlap scopes (`allow ∩ deny`)
   - allow-only scopes (`allow - deny`)
   - deny-only scopes (`deny - allow`)
2. Extend service-api snapshot model and builder with the three fields.
3. Extend `/metrics` payload rendering with three new markers.
4. Extend four endpoint metric lanes with canonical expected values and marker assertions.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/scope_fixture.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks and Mitigations
- Risk: semantic drift between fixture and emitted values.
  - Mitigation: expected values are derived from parsed canonical fixture rows in each lane.
- Risk: unintended behavior regression.
  - Mitigation: scoped endpoint suite + extraction contract + fmt/clippy.

## Interfaces / Contracts
- New snapshot fields:
  - `scope_policy_fixture_unique_allow_deny_overlap_scope_count: usize`
  - `scope_policy_fixture_unique_allow_only_scope_count: usize`
  - `scope_policy_fixture_unique_deny_only_scope_count: usize`
- New metrics markers:
  - `kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_scope_count`
  - `kamn_service_api_scope_policy_fixture_unique_allow_only_scope_count`
  - `kamn_service_api_scope_policy_fixture_unique_deny_only_scope_count`

## ADR
- Not required.
