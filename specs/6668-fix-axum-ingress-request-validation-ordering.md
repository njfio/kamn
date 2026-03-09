# Issue 6668: Fix Service API Axum Ingress Request-Validation Status Ordering

## Objective

Restore the documented request-validation ordering in the live axum-ingress validator path so malformed websocket upgrade requests, invalid methods, and unknown routes are classified by request-shape/route validation before auth failures, producing the expected `400`, `405`, and `404` envelopes in the real runtime lane.

## Inputs/Outputs

### Inputs
- `scripts/runtime/validate_service_api_axum_ingress_live.sh`
- `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/*`
- Existing service API docs and runtime policy contracts

### Outputs
- Live validator path returns the expected request-validation statuses and reason codes again
- Regression coverage that fails closed if auth preempts request-validation classification in this lane
- Updated spec evidence documenting the restored ordering semantics

## Boundaries/Non-goals

- Do not redesign the full service API auth model
- Do not weaken auth enforcement for valid protected routes
- Do not rewrite unrelated runtime-suite lanes
- Keep the fix limited to request-validation ordering and the contracts/docs that describe it

## Failure Modes

- Websocket upgrade probe still returns `401` instead of the documented `400`
- Invalid-method or unknown-route probes are classified through auth instead of method/route handling
- JSON error envelopes lose the documented reason codes or message markers
- Runtime docs/contracts claim one ordering while the live lane executes another

## Acceptance Criteria

- [ ] AC-1: The live websocket upgrade validation probe returns `400 Bad Request` with `service_api_ws_upgrade_header_missing` in the real validator path.
- [ ] AC-2: The live invalid-method probe returns `405 Method Not Allowed` with `service_api_method_not_allowed` in the real validator path.
- [ ] AC-3: The live unknown-route probe returns `404 Not Found` with `service_api_route_not_found` in the real validator path.
- [ ] AC-4: `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` passes locally.
- [ ] AC-5: Any touched docs/contracts remain aligned on the restored request-validation ordering semantics.

## Files To Touch

- `specs/6668-fix-axum-ingress-request-validation-ordering.md`
- `scripts/runtime/validate_service_api_axum_ingress_live.sh`
- `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- additional service API runtime/doc contract files only if required by the restored behavior

## Error Semantics

- Request-validation classification must fail closed with the documented status code and reason code for malformed websocket upgrade, invalid method, and unknown route probes
- Auth failures must still surface as unauthorized for routes that reach auth after passing request-shape/route classification
- No silent fallback from request-validation taxonomy to auth taxonomy in the targeted live lane

## Test Plan

1. Reproduce the current failure in `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`.
2. Add failing regression coverage for the observed request-validation ordering mismatch.
3. Implement the minimal runtime fix to restore request-validation ordering.
4. Re-run the targeted runtime policy lane and any touched Rust/docs contracts.
5. Record live-lane integration evidence in this spec.
