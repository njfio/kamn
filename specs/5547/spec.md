# Issue #5547 Spec - Service API Scope-Policy Fixture Method Overlap and Exclusive Coverage Metrics Exposure

- Status: Implemented
- Issue: #5547
- Parent: #3812
- Milestone: R50.39 Service API scope-policy fixture method overlap and exclusive coverage metrics exposure

## Problem Statement
Service API `/metrics` does not expose allow/deny method overlap and exclusive method coverage markers derived from canonical fixture rows.

## Scope
In scope:
- Derive overlap/allow-only/deny-only unique method counts from canonical fixture rows.
- Emit markers in `/metrics`.
- Extend endpoint metrics assertions across functional, HTTP integration, TLS integration, and unit observability lanes.

Out of scope:
- Fixture row edits.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_method_count <n>`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_allow_only_method_count <n>`.
- AC-3: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_deny_only_method_count <n>`.
- AC-4: values derive from canonical method-set operations.
- AC-5: targeted endpoint lanes assert markers and pass.

## Conformance Cases
- C-01 (AC-1): metrics payload contains overlap-method marker.
- C-02 (AC-2): metrics payload contains allow-only-method marker.
- C-03 (AC-3): metrics payload contains deny-only-method marker.
- C-04 (AC-4): tests compute expected values from parsed fixture method sets and match emitted markers.
- C-05 (AC-5): four endpoint lanes pass with assertions.

## Success Metrics / Observable Signals
- Runtime `/metrics` includes deterministic method overlap/exclusive markers.
- Existing endpoint behavior remains stable under scoped regression checks.
