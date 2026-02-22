# Issue #5539 Spec - Service API Scope-Policy Fixture Allow/Deny Route Coverage Metrics Exposure

- Status: Reviewed (agent-authored; human review requested in PR)
- Issue: #5539
- Parent: #3812
- Milestone: R50.35 Service API scope-policy fixture allow/deny route coverage metrics exposure

## Problem Statement
Service API `/metrics` publishes scope-policy fixture cardinality markers, but it does not expose allow/deny route-coverage markers for unique routes represented by allow vs deny fixture rows.

## Scope
In scope:
- Derive fixture unique allow-route and unique deny-route counts from canonical fixture rows.
- Emit allow/deny route-coverage markers in `/metrics`.
- Extend endpoint metrics tests to verify marker presence and canonical values.

Out of scope:
- Scope-policy fixture content changes.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_allow_route_count <n>`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_deny_route_count <n>`.
- AC-3: values derive from canonical fixture rows.
- AC-4: targeted endpoint lanes pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains fixture unique-allow-route-count marker.
- C-02 (AC-2): metrics response contains fixture unique-deny-route-count marker.
- C-03 (AC-3): tests parse canonical fixture rows and compare expected values with emitted markers.
- C-04 (AC-4): targeted endpoint lanes pass.

## Success Metrics / Observable Signals
- Runtime `/metrics` includes fixture allow/deny route-coverage markers with canonical values.
- Existing endpoint behavior remains stable under targeted regression checks.
