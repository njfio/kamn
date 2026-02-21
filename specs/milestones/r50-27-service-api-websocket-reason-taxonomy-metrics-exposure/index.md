# Milestone R50.27 - Service API Websocket Reason Taxonomy Metrics Exposure

## Objective
Ship production service API websocket taxonomy telemetry so `/metrics` publishes canonical websocket reason contract metadata.

## Scope
- Execute issue `#5523`.
- Emit websocket reason taxonomy version and reason-code count markers in `/metrics`.
- Validate with targeted service API endpoint tests and formatting checks.

## Deliverables
- `specs/5523/spec.md`
- `specs/5523/plan.md`
- `specs/5523/tasks.md`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Exit Criteria
- `/metrics` includes deterministic websocket taxonomy markers.
- Marker values derive from canonical runtime constants.
- Targeted endpoint lanes pass.
