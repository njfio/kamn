# Issue #5525 Plan - Service API Scope-Policy Fixture Schema Metrics

## Approach
1. Add RED assertions in existing `/metrics` endpoint test lanes for missing scope-policy fixture schema marker.
2. Make fixture schema version constant available in runtime service API code and snapshot model.
3. Emit marker line in metrics payload and rerun targeted lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-28-service-api-scope-policy-fixture-schema-metrics-exposure/index.md`
- `specs/5525/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: fixture schema constant remains test-only and drifts from runtime output.
  - Mitigation: promote constant to runtime scope and use directly in snapshot/metrics tests.

## Interfaces / Contracts
- Service API `/metrics` text payload contract.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
