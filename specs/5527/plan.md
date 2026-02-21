# Issue #5527 Plan - Service API Route-Authz Matrix Metrics

## Approach
1. Add RED `/metrics` assertions for missing route-authz matrix markers.
2. Add canonical runtime constants for route-authz matrix schema and counts.
3. Emit markers in metrics payload and rerun targeted lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-29-service-api-route-authz-matrix-metrics-exposure/index.md`
- `specs/5527/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker counts drift from authz matrix contract.
  - Mitigation: define canonical constants in runtime module; tests compare metrics lines against those constants.

## Interfaces / Contracts
- Service API `/metrics` text payload contract.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
