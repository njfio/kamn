# Issue #5535 Spec - Service API Scope-Policy Fixture Shape-Cardinality Metrics Exposure

- Status: Implemented (agent-authored; human review requested in PR)
- Issue: #5535
- Parent: #3812
- Milestone: R50.33 Service API scope-policy fixture shape-cardinality metrics exposure

## Problem Statement
Service API `/metrics` publishes scope-policy fixture row/coverage metadata but does not expose fixture shape-cardinality markers for unique methods and unique expected outcomes represented by canonical fixture rows.

## Scope
In scope:
- Derive fixture unique method/expected-outcome counts from canonical fixture rows.
- Emit shape-cardinality markers in `/metrics`.
- Extend endpoint metrics tests to verify marker presence and canonical values.

Out of scope:
- Scope-policy fixture content changes.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_method_count <n>`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_unique_expected_outcome_count <n>`.
- AC-3: values derive from canonical fixture rows.
- AC-4: targeted endpoint lanes pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains fixture unique-method-count marker.
- C-02 (AC-2): metrics response contains fixture unique-expected-outcome-count marker.
- C-03 (AC-3): tests parse canonical fixture rows and compare expected values with emitted markers.
- C-04 (AC-4): targeted endpoint lanes pass.

## Success Metrics / Observable Signals
- Runtime `/metrics` includes fixture shape-cardinality markers with canonical values.
- Existing endpoint behavior remains stable under targeted regression checks.
