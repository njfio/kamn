# Issue #5529 Plan - Service API Scope-Policy Fixture Row-Count Metrics

## Approach
1. Add RED `/metrics` assertions for fixture row-count markers in existing endpoint test lanes.
2. Add runtime helper deriving fixture `total/allow/deny` counts from canonical fixture content.
3. Emit row-count markers in metrics payload and re-run targeted lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-30-service-api-scope-policy-fixture-row-count-metrics-exposure/index.md`
- `specs/5529/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker counts drift if manually hardcoded.
  - Mitigation: derive counts from canonical fixture text in runtime helper and from fixture parser in tests.

## Interfaces / Contracts
- Service API `/metrics` text payload contract.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
