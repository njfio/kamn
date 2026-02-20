# Issue #5281 Plan

- Issue: #5281
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Extend websocket route handling in `kamn-node` to support an explicit presence mode (`x-kamn-events-mode: presence`).
2. Parse and validate presence-mode headers deterministically; fail closed on missing/invalid mode/inputs.
3. Call `data_layer_m9_gateway_project_presence_event` with a local `DataLayerM9RealtimeDeliveryRegistry` to project owner-scoped presence payloads.
4. Keep existing default state-transition websocket payload path unchanged for backward compatibility.
5. Add integration/regression websocket tests to validate deterministic presence payloads and fail-closed branches.
6. Run scoped verification (`fmt`, strict `clippy`, targeted service API tests).

## Affected Areas
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` (if route handler needs response mapping adjustments)
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5281/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: websocket compatibility regression for existing clients.
  - Mitigation: preserve default state-transition event path and keep existing tests as regression coverage.
- Risk: scope-denial mapping could lose M9 reason taxonomy.
  - Mitigation: map bridge errors directly and preserve source reason markers for owner-scope and visibility failures.
- Risk: header parsing introduces ambiguous behavior.
  - Mitigation: explicit mode-gated parsing with fail-closed missing/invalid header reason codes.

## Interfaces / Contracts
- New presence-mode websocket request contract:
  - `x-kamn-events-mode: presence`
  - owner/agent header set for connect/query projection.
- Existing `/v1/events/ws` default contract remains unchanged without the presence-mode header.

## ADR
Not required; this is an incremental runtime integration slice under existing Phase-5 architecture.
