# Issue #5525 Spec - Service API Scope-Policy Fixture Schema Metrics Exposure

- Status: Reviewed (agent-authored; human review requested in PR)
- Issue: #5525
- Parent: #3812
- Milestone: R50.28 Service API scope-policy fixture schema metrics exposure

## Problem Statement
Service API `/metrics` exposes multiple taxonomy markers but omits the scope-policy fixture schema version, reducing runtime visibility into scope-policy fixture contract lineage.

## Scope
In scope:
- Promote scope-policy fixture schema version constant for runtime use.
- Emit scope-policy fixture schema marker line in `/metrics`.
- Extend endpoint tests to enforce marker presence with canonical constant.

Out of scope:
- Scope-policy behavior changes.
- Fixture content/row updates.

## Acceptance Criteria
- AC-1: `/metrics` includes `kamn_service_api_scope_policy_fixture_schema_info{version="..."} 1`.
- AC-2: marker version comes from canonical runtime constant.
- AC-3: targeted service API endpoint lanes pass.

## Conformance Cases
- C-01 (AC-1): metrics response contains fixture schema marker line.
- C-02 (AC-2): tests assert marker line version equals canonical constant.
- C-03 (AC-3): targeted endpoint unit/integration/functional tests pass.

## Success Metrics / Observable Signals
- Runtime telemetry includes scope-policy fixture schema version marker.
- Existing service API metrics contract remains stable with additive marker only.
