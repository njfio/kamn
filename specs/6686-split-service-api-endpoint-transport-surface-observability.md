# Issue 6686: Split service_api_endpoint_tests Transport Surface Observability Tranche

## Objective

Extract the service API transport-surface and observability coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file keeps shrinking while the HTTP route, TLS, keep-alive, ingress-correlation, and metrics paths remain exercised from the real `kamn-node` test entrypoint.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- a new transport-surface-observability submodule declaration from the root test file
- moved HTTP route, TLS, keep-alive, ingress correlation, response-body, and metrics coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lower staged root threshold

## Boundaries/Non-goals

- Do not change route behavior, TLS behavior, metrics output semantics, or ingress-correlation semantics
- Do not extract the remaining small serde/error/resource-transition unit regressions in this issue
- Do not weaken existing transport assertions to make the split pass
- Keep the tranche limited to transport, connection behavior, and service API observability coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps the moved transport-surface test markers after extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real transport or observability path

## Acceptance Criteria

- [x] AC-1: `service_api_endpoint_tests.rs` declares a new transport-surface-observability submodule and no longer retains the moved transport-surface test markers.
- [x] AC-2: Extracted transport-surface-observability files stay at or below 200 lines each.
- [x] AC-3: The staged root threshold ratchets down from `2400` to `1350` lines or lower.
- [x] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [x] AC-5: At least one extracted transport-surface integration test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6686-split-service-api-endpoint-transport-surface-observability.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains the moved transport coverage or the extracted files exceed budget.
- The moved tests must continue exercising the real `kamn-node` transport and observability paths.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the transport-surface-observability module markers, moved test markers, and a lowered staged root threshold.
2. Extract the transport and observability coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` transport or observability tests.
4. Record integration evidence and any deviations in this spec.

## Deviations

- No behavioral deviations were introduced. The extraction preserved the transport, TLS, keep-alive, ingress-correlation, and observability assertions while moving them into bounded files.

## Phase 6 Evidence

- Root wiring:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` declares `#[path = "service_api_endpoint_tests/transport_surface_observability_contract_tests.rs"]` and `mod transport_surface_observability_contract_tests;`
- File sizes:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `1224`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests.rs`: `8`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/http_connection_contract_tests.rs`: `24`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/observability_contract_tests.rs`: `109`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/route_tls_contract_tests.rs`: `139`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/support.rs`: `91`
- Touched-Rust size policy:
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6686-touched-size.json`
  - Result: `status=pass`, `policy_decision=GO`
- Targeted evidence:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node transport_surface_observability_contract_tests -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_supports_keep_alive_requests_on_single_connection -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node regression_service_api_runtime_observability_projects_live_metrics_under_traffic -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --nocapture`
