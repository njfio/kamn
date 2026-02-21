# Issue #5523 Spec - Service API Websocket Reason Taxonomy Metrics Exposure

- Status: Implemented (agent-authored; human review requested in PR)
- Issue: #5523
- Parent: #3812
- Milestone: R50.27 Service API websocket reason taxonomy metrics exposure

## Problem Statement
Service API runtime `/metrics` does not expose websocket reason taxonomy metadata, reducing live observability for websocket contract drift across upgrade and presence validation paths.

## Scope
In scope:
- Add canonical websocket reason taxonomy constants (version + reason-codes CSV) in runtime service API code.
- Emit websocket taxonomy version and reason-code count markers in `/metrics`.
- Extend endpoint tests to assert marker presence and canonical count.

Out of scope:
- Changes to websocket validation logic or error semantics.
- New websocket reason categories.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_websocket_reason_taxonomy_info{version="..."} 1`.
- AC-2: `/metrics` includes `kamn_service_api_websocket_reason_code_count <n>`.
- AC-3: `<n>` derives from canonical runtime websocket taxonomy constants.
- AC-4: Targeted service API endpoint tests pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains websocket taxonomy version marker with canonical version constant.
- C-02 (AC-2): metrics response contains websocket reason-code count marker.
- C-03 (AC-3): count marker equals canonical CSV-derived non-empty reason-code count.
- C-04 (AC-4): targeted endpoint unit/integration/functional lanes pass.

## Success Metrics / Observable Signals
- Runtime metrics publish websocket reason taxonomy metadata for monitoring and policy-audit consumers.
- Existing service API endpoint behavior remains stable under targeted regression tests.
