# 6873 Split Middleware Impl

## Objective
Split `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` into bounded concern-based modules while preserving the existing service API middleware behavior and keeping the root file as a thin coordinator.

## Inputs/Outputs
- Input: current monolithic middleware implementation covering auth gating, request parsing, error translation, HTTP route handling, WebSocket route handling, relay payload parsing, and lifecycle rejection projection.
- Output: bounded sibling modules with the same externally callable middleware entrypoints and unchanged request/response semantics.

## Boundaries/Non-goals
- No intentional changes to request/response behavior.
- No redesign of unrelated service API handlers or server wiring.
- No new dependencies.
- No weakening of existing auth, replay, rate-limit, or anti-spam checks.

## Failure modes
- Extraction breaks real middleware wiring from `service_api_endpoint.rs`.
- HTTP or WebSocket route handlers diverge from current status/error behavior.
- Parsed payload helpers move but stop producing the same reason codes/messages.
- Root or extracted files remain above AGENTS.md file-size budget.
- New helper functions exceed the function-size cap.

## Acceptance criteria
- [ ] `middleware_impl.rs` is reduced to a thin root shell or bounded coordinator.
- [ ] auth middleware, request parsing/error response, HTTP route handling, WebSocket route handling, and payload parsing/rejection helpers are split into concern-based sibling modules.
- [ ] no extracted file exceeds 200 LOC.
- [ ] no newly introduced helper exceeds 25 LOC.
- [ ] existing service-api middleware behavior remains green under targeted tests.
- [ ] at least one extraction contract enforces the root shell and extracted module layout.

## Files to touch
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/`
- `crates/kamn-node/tests/service_api_endpoint_middleware_impl_extraction_contract.rs`
- `specs/6873-split-middleware-impl.md`

## Error semantics
- Existing `ServiceApiReasonedError` and `ServiceApiMiddlewareError` behavior must remain unchanged.
- Entry-point middleware continues to hard-fail through `service_api_middleware_error_response` with the same status/outcome/reason fields.
- Extracted interior helpers return typed errors/results and do not add logging side effects beyond current entrypoint behavior.

## Test plan
- Red extraction contract asserting:
  - root shell budget for `middleware_impl.rs`
  - required extracted module declarations
  - absence of moved monolith sections from the root file
  - bounded extracted files
- Green verification with targeted real tests covering service API endpoint middleware behavior.
- Run touched-Rust size policy against the issue branch once extraction is complete.
