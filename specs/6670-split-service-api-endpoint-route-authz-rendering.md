# Issue 6670: Split service_api_endpoint_tests Route Authz And Rendering Tranche

## Objective

Extract the route/authz helper surface and the route-rendering contract coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file continues shrinking and the service API route contract surface becomes reviewable by concern.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- new route/rendering submodule declarations from the root test file
- moved route/authz helpers and route-rendering contract coverage in bounded files
- updated split contract coverage for the new extraction markers and staged root threshold

## Boundaries/Non-goals

- Do not rewrite service API behavior
- Do not re-extract websocket or auth/scope files already moved
- Do not solve the entire remaining oversized root file in one issue
- Keep the tranche limited to route/authz helpers plus route-rendering/route-observability contract coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps moved route/rendering tests or helpers after the extraction
- New submodules exceed the 200 LOC budget
- Root file staged threshold does not improve after the extraction
- Contract coverage is lost or detached from the real `kamn-node` test entrypoint

## Acceptance Criteria

- [ ] AC-1: `service_api_endpoint_tests.rs` declares a new route/rendering submodule and no longer retains the moved route/rendering test markers.
- [ ] AC-2: Route/authz helpers needed only by the moved coverage are extracted with the new module instead of remaining in the root file.
- [ ] AC-3: Extracted route/rendering files stay at or below 200 lines each.
- [ ] AC-4: The staged root threshold ratchets down from the current `8156` lines.
- [ ] AC-5: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.

## Files To Touch

- `specs/6670-split-service-api-endpoint-route-authz-rendering.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/route_render_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/route_render_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains moved helpers/tests or new extracted files exceed budget
- The moved tests must continue exercising the real `kamn-node` service API contract path
- No silent fallback to leaving duplicate helper logic in both root and extracted modules

## Test Plan

1. Add red split-contract assertions for the new route/rendering module markers, moved helper/test markers, and a lowered staged root threshold.
2. Extract the route/rendering coverage into bounded files until the split contract passes.
3. Run the targeted split contract and any directly affected `kamn-node` tests.
4. Record integration evidence in this spec.
