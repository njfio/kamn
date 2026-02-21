# Issue #5519 Plan - Service API Auth/Scope Taxonomy Runtime Metrics

## Approach
1. Add RED test assertions in service API endpoint tests for missing auth/scope taxonomy marker lines in `/metrics`.
2. Wire canonical taxonomy versions and reason-code counts into runtime metrics rendering.
3. Run targeted service API endpoint + formatting lanes.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs` (taxonomy constants visibility / helpers as needed)
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-25-service-api-auth-and-scope-taxonomy-metrics-exposure/index.md`
- `specs/5519/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: duplicated taxonomy literals drift from canonical definitions.
  - Mitigation: derive values from canonical constants already validated by route/scope fixture tests.
- Risk: metric cardinality mismatch if CSV parsing includes empty fields.
  - Mitigation: derive count via split/filter non-empty pattern used by existing cross-store marker tests.

## Interfaces / Contracts
- Service API `/metrics` text format contract.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
