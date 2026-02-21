# Issue #5533 Spec - Service API Scope-Policy Fixture Coverage Cardinality Metrics Exposure

- Status: Reviewed (agent-authored; human review requested in PR)
- Issue: #5533
- Parent: #3812
- Milestone: R50.32 Service API scope-policy fixture coverage cardinality metrics exposure

## Problem Statement
Service API `/metrics` publishes scope-policy fixture taxonomy and row markers, but it does not expose fixture coverage cardinality markers for unique routes and unique scopes represented by canonical fixture rows.

## Scope
In scope:
- Derive fixture unique route/scope counts from canonical fixture rows.
- Emit coverage cardinality markers in `/metrics`.
- Extend endpoint metrics tests to verify marker presence and canonical values.

Out of scope:
- Scope-policy fixture content changes.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_route_count <n>`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_scope_count <n>`.
- AC-3: values derive from canonical fixture rows.
- AC-4: targeted endpoint lanes pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains fixture unique-route-count marker.
- C-02 (AC-2): metrics response contains fixture unique-scope-count marker.
- C-03 (AC-3): tests parse canonical fixture rows and compare expected values with emitted markers.
- C-04 (AC-4): targeted endpoint lanes pass.

## Success Metrics / Observable Signals
- Runtime `/metrics` includes fixture coverage cardinality markers with canonical values.
- Existing endpoint behavior remains stable under targeted regression checks.
