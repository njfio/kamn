# Issue #5521 Spec - Service API Lifecycle Rejection Taxonomy Metrics Exposure

- Status: Implemented (agent-authored; human review requested in PR)
- Issue: #5521
- Parent: #3812
- Milestone: R50.26 Service API lifecycle rejection taxonomy metrics exposure

## Problem Statement
Runtime service API `/metrics` lacks lifecycle-rejection policy taxonomy markers, making it harder to verify fail-closed rejection contract drift from live telemetry.

## Scope
In scope:
- Add canonical lifecycle-rejection taxonomy constants (version + reason-codes CSV) to runtime service API code.
- Emit lifecycle-rejection taxonomy version and reason-code cardinality markers in `/metrics`.
- Extend endpoint tests to enforce marker presence and canonical count.

Out of scope:
- Behavioral changes to rejection mapping/status/error labels.
- New rejection policy categories beyond existing reason codes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_lifecycle_rejection_reason_taxonomy_info{version="..."} 1`.
- AC-2: `/metrics` includes `kamn_service_api_lifecycle_rejection_reason_code_count <n>`.
- AC-3: `<n>` derives from canonical runtime lifecycle-rejection taxonomy constants.
- AC-4: Targeted service API endpoint tests pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains lifecycle rejection taxonomy version marker with canonical version constant.
- C-02 (AC-2): metrics response contains lifecycle rejection reason-code count marker.
- C-03 (AC-3): count marker equals canonical CSV-derived non-empty reason-code count.
- C-04 (AC-4): targeted unit/integration endpoint tests pass.

## Success Metrics / Observable Signals
- Runtime metrics expose lifecycle-rejection taxonomy metadata for policy drift detection.
- No regression in existing service API route/metrics behavior tests.
