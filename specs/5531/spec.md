# Issue #5531 Spec - Service API Scope-Policy Fixture Metadata Parity Metrics Exposure

- Status: Reviewed (agent-authored; human review requested in PR)
- Issue: #5531
- Parent: #3812
- Milestone: R50.31 Service API scope-policy fixture metadata parity metrics exposure

## Problem Statement
Service API `/metrics` currently publishes scope-policy taxonomy markers and fixture schema markers, but it does not expose fixture metadata parity markers for fixture-declared reason taxonomy/version coupling. This limits runtime observability for fixture metadata drift.

## Scope
In scope:
- Derive scope-policy fixture metadata parity values from canonical fixture metadata.
- Emit fixture metadata parity markers in `/metrics`.
- Extend endpoint metrics tests to verify marker presence and canonical values.

Out of scope:
- Scope-policy fixture content changes.
- Scope-policy authorization behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_reason_taxonomy_info{version="<v>"} 1`.
- AC-2: `/metrics` includes `kamn_service_api_scope_policy_fixture_reason_code_count <n>`.
- AC-3: values derive from canonical fixture metadata (`scope_policy_reason_taxonomy_version`, `scope_policy_reason_codes_csv`).
- AC-4: targeted endpoint lanes pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains fixture reason taxonomy info marker.
- C-02 (AC-2): metrics response contains fixture reason code-count marker.
- C-03 (AC-3): tests parse canonical fixture metadata and compare expected values with emitted markers.
- C-04 (AC-4): targeted endpoint lanes pass.

## Success Metrics / Observable Signals
- Runtime `/metrics` includes fixture metadata parity markers with canonical values.
- Existing endpoint behavior remains stable under targeted regression checks.
