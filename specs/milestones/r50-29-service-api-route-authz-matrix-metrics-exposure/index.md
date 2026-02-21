# Milestone R50.29 - Service API Route-Authz Matrix Metrics Exposure

## Objective
Ship production service API `/metrics` markers for route-authz matrix schema and deterministic cardinalities.

## Scope
- Execute issue `#5527`.
- Emit route-authz matrix schema version + total/public/protected route count markers.
- Validate with targeted service API endpoint test lanes.

## Deliverables
- `specs/5527/spec.md`
- `specs/5527/plan.md`
- `specs/5527/tasks.md`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Exit Criteria
- `/metrics` includes deterministic route-authz matrix markers.
- Marker values derive from canonical runtime constants.
- Targeted lanes pass.
