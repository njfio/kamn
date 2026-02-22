# Issue #5541 Spec - Service API Scope-Policy Fixture Allow/Deny Overlap Route Coverage Metrics Exposure

- Status: Implemented
- Issue: #5541
- Parent: #3812
- Milestone: R50.36 Service API scope-policy fixture allow-deny overlap route coverage metrics exposure

## Problem Statement
Service API `/metrics` exposes fixture allow-route and deny-route cardinality markers, but it does not expose overlap route coverage for unique `(method,path)` routes that occur in both allow and deny fixture rows.

## Scope
In scope:
- Derive a fixture unique allow/deny overlap route count from canonical fixture rows.
- Emit overlap route coverage marker in `/metrics`.
- Extend endpoint metrics tests to verify marker presence and canonical value.

Out of scope:
- Scope-policy fixture content changes.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count <n>`.
- AC-2: `<n>` equals the unique `(method,path)` intersection count between allow rows and deny rows.
- AC-3: Functional + HTTP integration + TLS integration + unit observability endpoint lanes assert the marker.
- AC-4: Existing route/auth behavior remains unchanged.

## Conformance Cases
- C-01 (AC-1): metrics response contains overlap route coverage marker.
- C-02 (AC-2): tests compute expected overlap count from canonical fixture rows and match emitted value.
- C-03 (AC-3): four endpoint lanes assert overlap marker presence/value and pass.
- C-04 (AC-4): scoped endpoint suite passes without behavior regression.

## Success Metrics / Observable Signals
- Runtime `/metrics` publishes deterministic overlap route coverage marker derived from canonical fixture rows.
- Existing service API endpoint tests remain green.
