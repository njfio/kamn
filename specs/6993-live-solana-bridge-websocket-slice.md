# 6993-live-solana-bridge-websocket-slice

## Objective
Prove that the existing live Solana-backed service-api bridge forward lane reaches the real websocket event stream on current `main`. The slice must show that a websocket subscriber observes a `service-api.bridge.forwarded` event carrying non-placeholder `target_message_id` and `forward_tx_hash` values after a live Solana-backed bridge forward.

## Inputs/Outputs
- Inputs:
  - existing websocket upgrade route `/v1/events/ws`
  - existing service-api bridge submit and forward routes
  - existing live Solana bridge env `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`
  - existing live Solana proof runner from `#6989`
- Outputs:
  - one websocket integration test exercising upgrade plus live bridge submit/forward on the same server
  - one bounded validation runbook under `docs/validation/`
  - one docs contract anchoring the runbook and runtime-proof index entry

## Boundaries/Non-goals
- Do not claim live on-chain settlement.
- Do not claim bridge finality beyond the existing bounded proofs.
- Do not add a Solana SDK dependency.
- Do not weaken websocket auth, replay protection, or request signature requirements.
- Do not replace the existing live bridge dispatch proof; this issue deepens it through a downstream consumer surface.

## Failure modes
- The websocket stream does not emit a `service-api.bridge.forwarded` event after live bridge forward.
- The websocket event emits placeholder `msg-bridge-target-*` or `sha256:bridge-forwarded-*` values on the live lane.
- Empty live Solana RPC configuration fails late instead of at startup.
- The runbook overstates the slice as settlement or finality proof.

## Acceptance criteria (testable booleans)
- [ ] A websocket integration test proves a live Solana-backed bridge forward emits a `service-api.bridge.forwarded` event.
- [ ] The forwarded websocket event carries non-placeholder `target_message_id` and `forward_tx_hash` values.
- [ ] The integration test uses the real websocket upgrade plus the real bridge submit/forward routes on current `main`.
- [ ] A bounded validation runbook documents what this slice proves and what it does not prove.
- [ ] The runtime proof index links the websocket slice.

## Files to touch
- `specs/6993-live-solana-bridge-websocket-slice.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests/`
- `crates/kamn-node/tests/`
- `docs/validation/`
- `docs/review/corrected-audit-response-2026-03-14.md`

## Error semantics
- Missing or invalid `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL` remains a startup failure.
- Live bridge forward failures remain explicit `service_api_live_bridge_dispatch_failed` errors.
- Websocket upgrade/auth failures remain fail-closed and unchanged.

## Test plan
- Phase 3 red tests: add a docs contract for the missing runbook and add one websocket integration test covering upgrade plus live bridge forward.
- Green by wiring any missing websocket test support and publishing the bounded runbook.
- Re-run the new websocket slice test, the docs contract, and the touched-Rust size policy before publish.
