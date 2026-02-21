# Issue #5523 Plan - Service API Websocket Taxonomy Runtime Metrics

## Approach
1. Add RED `/metrics` assertions for websocket taxonomy markers in existing service API endpoint tests.
2. Add canonical websocket taxonomy constants and derive cardinality in service API snapshot construction.
3. Emit websocket taxonomy markers from metrics payload and run targeted verification lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-27-service-api-websocket-reason-taxonomy-metrics-exposure/index.md`
- `specs/5523/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: websocket taxonomy constants drift from real reason-code surface.
  - Mitigation: define canonical constants in runtime module and use those constants in tests/metrics derivation.
- Risk: additive metrics changes break strict endpoint assertions.
  - Mitigation: update all `/metrics` assertion lanes consistently (unit + http + tls + functional).

## Interfaces / Contracts
- Service API `/metrics` text contract.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
