# Issue 6668: Fix Service API Axum Ingress Request-Validation Status Ordering

- Status: Implemented

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

- [x] AC-1: The live websocket upgrade validation probe returns `400 Bad Request` with `service_api_ws_upgrade_header_missing` in the real validator path.
- [x] AC-2: The live invalid-method probe returns `405 Method Not Allowed` with `service_api_method_not_allowed` in the real validator path.
- [x] AC-3: The live unknown-route probe returns `404 Not Found` with `service_api_route_not_found` in the real validator path.
- [x] AC-4: `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` passes locally.
- [x] AC-5: Any touched docs/contracts remain aligned on the restored request-validation ordering semantics.

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

## Refactor Evidence

- The live validator now isolates request-validation, websocket, and concurrency probe families behind separate signer contexts instead of reusing one sender until anti-spam throttles the lane.
- Stale embedded Python runner inputs were removed so the signer-context flow is self-contained.
- The touched runtime shell files remain legacy oversized files outside the repo’s ideal size target; this issue kept the fix localized instead of attempting a risky decomposition of the full validator surface.

## Phase 6 integration evidence

- Executed:
  - `bash scripts/runtime/validate_service_api_axum_ingress_live.sh --output-json /tmp/6668-axum-ingress-live.json`
  - `bash scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
  - `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- Results:
  - The live validator returned `status=pass` and `final_decision=GO`.
  - Request-validation probes now classify through the documented envelopes:
    - websocket upgrade missing header -> `400` / `service_api_ws_upgrade_header_missing`
    - invalid method -> `405` / `service_api_method_not_allowed`
    - unknown route -> `404` / `service_api_route_not_found`
  - The axum-ingress contract lane passed end to end.
  - The policy checker wrapper passed end to end.
  - Both runtime scripts are already exercised by the broader harness through `scripts/ci/test_ci_tools.sh` and referenced from release/runbook docs, so the fix is wired into real entrypoints rather than a mock-only path.

## Deviations

- The underlying regression was not middleware ordering after all. The live validator itself had drifted behind the production auth contract by:
  - omitting `X-KAMN-Signer-Public-Key`
  - using legacy non-self-certifying sender DIDs
  - reusing one sender long enough to trigger anti-spam before the websocket success probe
- Restoring the live probe/auth inputs fixed the request-validation status ordering without changing production auth or route middleware behavior.
