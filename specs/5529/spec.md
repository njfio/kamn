# Issue #5529 Spec - Service API Scope-Policy Fixture Row-Count Metrics Exposure

- Status: Reviewed (agent-authored; human review requested in PR)
- Issue: #5529
- Parent: #3812
- Milestone: R50.30 Service API scope-policy fixture row-count metrics exposure

## Problem Statement
Service API `/metrics` currently publishes scope-policy fixture schema metadata but does not publish fixture row cardinalities (`total`, `allow`, `deny`), limiting runtime visibility into fixture shape drift.

## Scope
In scope:
- Derive scope-policy fixture row counts from canonical fixture content.
- Emit `total`, `allow`, and `deny` row-count markers in `/metrics`.
- Extend endpoint tests to verify marker presence and canonical count values.

Out of scope:
- Scope-policy fixture content changes.
- Scope-policy behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_row_count <n>`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_allow_row_count <n>`.
- AC-3: `/metrics` includes `kamn_service_api_scope_policy_fixture_deny_row_count <n>`.
- AC-4: values derive from canonical fixture content.
- AC-5: targeted endpoint lanes pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains fixture total row-count marker.
- C-02 (AC-2): metrics response contains fixture allow row-count marker.
- C-03 (AC-3): metrics response contains fixture deny row-count marker.
- C-04 (AC-4): tests compute counts from canonical fixture parsing and compare with metrics markers.
- C-05 (AC-5): targeted endpoint lanes pass.

## Success Metrics / Observable Signals
- Runtime `/metrics` exposes scope-policy fixture cardinality markers.
- Existing endpoint behavior remains stable under targeted regression checks.
