# Milestone R50.28 - Service API Scope-Policy Fixture Schema Metrics Exposure

## Objective
Ship production service API `/metrics` telemetry for scope-policy fixture schema metadata so policy contract lineage is observable in runtime metrics.

## Scope
- Execute issue `#5525`.
- Emit `scope_policy_fixture_schema` marker in `/metrics` output.
- Validate with targeted unit/integration/functional service API endpoint lanes.

## Deliverables
- `specs/5525/spec.md`
- `specs/5525/plan.md`
- `specs/5525/tasks.md`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Exit Criteria
- `/metrics` includes scope-policy fixture schema marker.
- Marker value derives from canonical runtime constant.
- Targeted validation lanes pass.
