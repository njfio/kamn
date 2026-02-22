# Issue #5547 Plan - Service API Scope-Policy Fixture Method Overlap and Exclusive Coverage Metrics Exposure

## Approach
1. Extend fixture projection parser with allow/deny method set arithmetic to derive overlap/allow-only/deny-only counts.
2. Extend service-api snapshot fields and builder wiring.
3. Extend `/metrics` payload rendering with three markers.
4. Extend four endpoint metric lanes with expected method-set values and assertions.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/scope_fixture.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks and Mitigations
- Risk: set arithmetic drift.
  - Mitigation: each lane computes expected marker values directly from canonical fixture rows.
- Risk: unintended behavior drift.
  - Mitigation: run scoped endpoint suite + extraction contract + fmt/clippy.

## Interfaces / Contracts
- New snapshot fields:
  - `scope_policy_fixture_unique_allow_deny_overlap_method_count: usize`
  - `scope_policy_fixture_unique_allow_only_method_count: usize`
  - `scope_policy_fixture_unique_deny_only_method_count: usize`
- New metrics markers:
  - `kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_method_count`
  - `kamn_service_api_scope_policy_fixture_unique_allow_only_method_count`
  - `kamn_service_api_scope_policy_fixture_unique_deny_only_method_count`

## ADR
- Not required.
