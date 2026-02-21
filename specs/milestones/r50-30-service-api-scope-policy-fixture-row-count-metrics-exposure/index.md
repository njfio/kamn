# Milestone R50.30 - Service API Scope-Policy Fixture Row-Count Metrics Exposure

## Objective
Ship production `/metrics` markers for scope-policy fixture row cardinalities (`total`, `allow`, `deny`) derived from canonical fixture content.

## Scope
- Execute issue `#5529`.
- Derive fixture row counts from canonical fixture text at runtime snapshot construction.
- Emit row-count markers in service API `/metrics`.

## Deliverables
- `specs/5529/spec.md`
- `specs/5529/plan.md`
- `specs/5529/tasks.md`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Exit Criteria
- `/metrics` contains fixture row-count markers (`total`, `allow`, `deny`).
- Marker values derive from canonical fixture content.
- Targeted endpoint lanes pass.
