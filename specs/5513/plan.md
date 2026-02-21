# Issue #5513 Plan - Service API Cross-Store Replay Metrics Integration

## Approach
1. Add RED assertions in service API endpoint tests for the new cross-store replay metrics lines.
2. Extend `ServiceApiSnapshot` and `build_service_api_snapshot` to include taxonomy version + reason-code count from `kamn-core` policy APIs.
3. Emit additive metrics lines in `/metrics` payload rendering.
4. Run targeted `kamn-node` service API tests and format checks.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/milestones/r50-22-service-api-cross-store-replay-telemetry-exposure/index.md`
- `specs/5513/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: additive metrics drift across runtime modes.
  - Mitigation: assert the new lines in both unknown and daemon-observability test paths.

## Interfaces / Contracts
- Additive service API metrics contract only.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
