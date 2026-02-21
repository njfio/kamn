# Issue #5527 Spec - Service API Route-Authz Matrix Metrics Exposure

- Status: Reviewed (agent-authored; human review requested in PR)
- Issue: #5527
- Parent: #3812
- Milestone: R50.29 Service API route-authz matrix metrics exposure

## Problem Statement
Service API `/metrics` does not expose route-authz matrix schema/cardinality metadata, which makes runtime validation of protected/public route contract shape less observable.

## Scope
In scope:
- Add canonical runtime constants for route-authz matrix schema version and deterministic counts.
- Emit route-authz matrix metrics markers in `/metrics`.
- Extend targeted endpoint tests to assert marker presence and canonical values.

Out of scope:
- Route auth policy behavior changes.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_route_authz_matrix_schema_info{version="..."} 1`.
- AC-2: `/metrics` includes total/public/protected route count markers.
- AC-3: marker values derive from canonical runtime constants.
- AC-4: targeted endpoint lanes pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains route-authz matrix schema marker line.
- C-02 (AC-2): metrics response contains route-authz matrix count lines.
- C-03 (AC-3): tests assert count lines match canonical constants.
- C-04 (AC-4): targeted service API endpoint lanes pass.

## Success Metrics / Observable Signals
- Runtime metrics publish route-authz matrix schema and cardinality metadata.
- Existing endpoint behavior remains stable under targeted regression tests.
