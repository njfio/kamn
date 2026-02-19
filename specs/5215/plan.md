# Issue #5215 Plan

- Issue: #5215
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Establish RED by adding a decomposition contract test that fails against current monolith line budget.
2. Extract cohesive code into submodules under `service_api_endpoint/`:
   - `request_auth.rs`: auth parsing, header extraction, nonce/signature validation.
   - `routing.rs`: route parsing helpers and HTTP route handlers.
   - `websocket.rs`: websocket header validation, upgrade response, event streaming.
   - `payload.rs`: payload parsing/serialization and JSON escape helpers.
3. Keep shared types/constants in root and wire submodule calls with `pub(super)` visibility.
4. Run targeted service API test suite and regression subset.
5. Update issue/process markers and set spec to `Implemented` when merged.

## Risks and Mitigations
- Risk: behavior drift from moving private helpers.
  - Mitigation: move logic verbatim first, then run focused regressions for reason-code stability.
- Risk: visibility/import breakage across moved functions.
  - Mitigation: use explicit `pub(super)` exports and compile/test after each extraction step.
- Risk: root line count unchanged if too little extracted.
  - Mitigation: enforce budget in a deterministic test and verify with `wc -l`.

## Interfaces / Contracts
- Preserve these public APIs and signatures:
  - `build_service_api_snapshot`
  - `render_service_api_endpoint_response`
  - `serve_service_api_endpoint`
  - `project_service_api_lifecycle_rejection`
  - `parse_service_api_payload`
  - `service_api_payload_decode_reason_code`
- Preserve all existing service-api reason codes and route path constants.
