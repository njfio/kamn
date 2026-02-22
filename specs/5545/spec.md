# Issue #5545 Spec - Service API Scope-Policy Fixture Scope Overlap and Exclusive Coverage Metrics Exposure

- Status: Implemented
- Issue: #5545
- Parent: #3812
- Milestone: R50.38 Service API scope-policy fixture scope overlap and exclusive coverage metrics exposure

## Problem Statement
Service API `/metrics` does not expose overlap and exclusive scope coverage markers for scope-policy fixture rows.

## Scope
In scope:
- Derive unique overlap scope count, unique allow-only scope count, and unique deny-only scope count from canonical fixture rows.
- Emit markers in `/metrics`.
- Extend endpoint metric assertions across functional, HTTP integration, TLS integration, and unit observability lanes.

Out of scope:
- Fixture row changes.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_scope_count <n>`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_allow_only_scope_count <n>`.
- AC-3: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_deny_only_scope_count <n>`.
- AC-4: values derive from canonical scope-set arithmetic (intersection and set differences).
- AC-5: targeted endpoint lanes assert markers and pass.

## Conformance Cases
- C-01 (AC-1): metrics payload contains overlap-scope marker.
- C-02 (AC-2): metrics payload contains allow-only-scope marker.
- C-03 (AC-3): metrics payload contains deny-only-scope marker.
- C-04 (AC-4): tests compute expected values from fixture scope sets and match emitted values.
- C-05 (AC-5): four endpoint metric lanes pass with assertions.

## Success Metrics / Observable Signals
- Runtime `/metrics` exposes deterministic scope overlap/exclusive markers.
- Existing endpoint behavior remains stable under scoped regression run.
