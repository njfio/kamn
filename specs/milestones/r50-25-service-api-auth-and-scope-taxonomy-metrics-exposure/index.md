# Milestone R50.25 - Service API Auth and Scope Taxonomy Metrics Exposure

## Objective
Ship production telemetry in service API `/metrics` for auth and scope-policy taxonomy contracts, improving runtime observability and feature-delivery rebalancing.

## Scope
- Create and execute issue `#5519`.
- Expose auth and scope-policy taxonomy version and reason-code count markers in runtime `/metrics`.
- Validate with targeted endpoint tests and formatting checks.

## Deliverables
- `specs/5519/spec.md`
- `specs/5519/plan.md`
- `specs/5519/tasks.md`
- `crates/kamn-node/src/service_api_endpoint.rs` (if snapshot fields are needed)
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Exit Criteria
- `/metrics` emits deterministic auth/scope taxonomy marker lines.
- Marker values derive from canonical constants.
- All ACs in `specs/5519/spec.md` pass in targeted tests.
