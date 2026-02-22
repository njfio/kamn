# Issue #5543 Spec - Service API Scope-Policy Fixture Exclusive Allow-Only/Deny-Only Route Coverage Metrics Exposure

- Status: Implemented
- Issue: #5543
- Parent: #3812
- Milestone: R50.37 Service API scope-policy fixture exclusive allow-only and deny-only route coverage metrics exposure

## Problem Statement
Service API `/metrics` lacks explicit exclusive route-coverage markers for routes present only in allow rows and only in deny rows of the canonical scope-policy fixture.

## Scope
In scope:
- Derive unique allow-only route and unique deny-only route counts from canonical fixture rows.
- Emit exclusive coverage markers in `/metrics`.
- Extend endpoint metric tests in functional, HTTP integration, TLS integration, and unit observability lanes.

Out of scope:
- Fixture row content changes.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_allow_only_route_count <n>`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_deny_only_route_count <n>`.
- AC-3: values are canonical set differences (allow-only and deny-only).
- AC-4: targeted endpoint lanes assert markers and pass.

## Conformance Cases
- C-01 (AC-1): metrics payload includes allow-only marker.
- C-02 (AC-2): metrics payload includes deny-only marker.
- C-03 (AC-3): tests compute canonical set-difference values from fixture rows and match emitted values.
- C-04 (AC-4): four endpoint lanes pass with marker assertions.

## Success Metrics / Observable Signals
- Runtime `/metrics` exposes deterministic exclusive route-coverage markers.
- Existing endpoint behavior remains stable under scoped regression checks.
