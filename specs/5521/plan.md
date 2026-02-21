# Issue #5521 Plan - Service API Lifecycle Rejection Taxonomy Metrics

## Approach
1. Add RED assertions for missing lifecycle rejection taxonomy markers in existing `/metrics` tests.
2. Add canonical lifecycle rejection taxonomy constants and derive count in runtime snapshot construction.
3. Emit new metrics lines and run targeted test lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-26-service-api-lifecycle-rejection-taxonomy-metrics-exposure/index.md`
- `specs/5521/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: metric constants drift from actual rejection policy mappings.
  - Mitigation: define canonical constants in runtime module and derive counts via CSV parsing in both snapshot and tests.
- Risk: additive metrics changes break strict expectations in existing tests.
  - Mitigation: update existing metrics assertion tests to include new required lines.

## Interfaces / Contracts
- Service API `/metrics` text payload contract only.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
