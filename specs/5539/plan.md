# Issue #5539 Plan - Service API Scope-Policy Fixture Allow/Deny Route Coverage Metrics

## Approach
1. Add RED `/metrics` assertions for fixture unique allow-route and unique deny-route markers across existing endpoint metrics lanes.
2. Extend runtime fixture projection helper to derive unique allow/deny route counts from canonical fixture rows.
3. Emit allow/deny route-coverage markers in metrics payload and run targeted lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/scope_fixture.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-35-service-api-scope-policy-fixture-allow-deny-route-coverage-metrics-exposure/index.md`
- `specs/5539/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: allow/deny route categorization drift if fixture parsing is inconsistent.
  - Mitigation: compute expected values by parsing canonical fixture rows in tests and compare against emitted metrics values.

## Interfaces / Contracts
- Service API `/metrics` text payload contract.

## Validation Strategy
- `cargo test -p kamn-node --test service_api_endpoint_module_extraction_contract -- --exact conformance_service_api_endpoint_root_stays_within_line_budget`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
