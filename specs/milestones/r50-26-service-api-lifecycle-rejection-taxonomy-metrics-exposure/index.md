# Milestone R50.26 - Service API Lifecycle Rejection Taxonomy Metrics Exposure

## Objective
Ship production telemetry for service API lifecycle-rejection policy taxonomy so runtime `/metrics` exposes canonical rejection contract metadata.

## Scope
- Execute issue `#5521`.
- Expose lifecycle rejection reason taxonomy version and reason-code count markers in `/metrics`.
- Validate through targeted service API endpoint tests and formatting checks.

## Deliverables
- `specs/5521/spec.md`
- `specs/5521/plan.md`
- `specs/5521/tasks.md`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Exit Criteria
- `/metrics` contains deterministic lifecycle rejection taxonomy markers.
- Marker values are sourced from canonical runtime constants.
- Targeted test lanes pass.
