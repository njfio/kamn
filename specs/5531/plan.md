# Issue #5531 Plan - Service API Scope-Policy Fixture Metadata Parity Metrics

## Approach
1. Add RED `/metrics` assertions for fixture metadata parity markers across existing endpoint metrics lanes.
2. Add runtime helper deriving fixture metadata parity values from canonical fixture content.
3. Emit metadata parity markers in metrics payload and run targeted lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-31-service-api-scope-policy-fixture-metadata-parity-metrics-exposure/index.md`
- `specs/5531/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: fixture metadata parsing drift from canonical fixture format.
  - Mitigation: parse canonical fixture metadata keys directly and validate through RED assertions in endpoint tests.

## Interfaces / Contracts
- Service API `/metrics` text payload contract.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
