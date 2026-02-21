# Issue #5519 Spec - Service API Auth and Scope Taxonomy Metrics Exposure

- Status: Implemented (agent-authored; human review requested in PR)
- Issue: #5519
- Parent: #3812
- Milestone: R50.25 Service API auth and scope taxonomy metrics exposure

## Problem Statement
Service API `/metrics` currently exposes cross-store replay taxonomy markers but omits service API auth and scope-policy taxonomy markers, leaving key policy contract versions non-observable in runtime telemetry.

## Scope
In scope:
- Expose auth reason taxonomy version and reason-code count in `/metrics`.
- Expose scope-policy taxonomy version and reason-code count in `/metrics`.
- Ensure values derive from canonical taxonomy constants.
- Add/extend tests to enforce marker presence and cardinality.

Out of scope:
- Changes to auth/scope policy enforcement behavior.
- Route/scope matrix semantics changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_auth_reason_taxonomy_info{version=\"...\"} 1`.
- AC-2: `/metrics` includes `kamn_service_api_auth_reason_code_count <n>` where `<n>` matches canonical auth taxonomy code count.
- AC-3: `/metrics` includes `kamn_service_api_scope_policy_reason_taxonomy_info{version=\"...\"} 1`.
- AC-4: `/metrics` includes `kamn_service_api_scope_policy_reason_code_count <n>` where `<n>` matches canonical scope-policy taxonomy code count.
- AC-5: Targeted service API endpoint tests pass.

## Conformance Cases
- C-01 (AC-1): `/metrics` response contains auth taxonomy info marker with canonical version.
- C-02 (AC-2): `/metrics` response contains auth reason-code count marker equal to canonical CSV-derived count.
- C-03 (AC-3): `/metrics` response contains scope-policy taxonomy info marker with canonical version.
- C-04 (AC-4): `/metrics` response contains scope-policy reason-code count marker equal to canonical CSV-derived count.
- C-05 (AC-5): targeted endpoint test lanes pass.

## Success Metrics / Observable Signals
- Runtime metrics expose service API auth/scope taxonomy versioning and cardinality markers.
- Service API observability consumers can validate taxonomy drift without reading fixture files.
