# 7007 Live Bridge Presence-Stream Slice

## Objective
Prove one bounded live Solana-backed bridge presence-stream slice on current `main`. The proof must show that the existing live bridge forward lane reaches the presence-mode websocket consumer surface, not just the upgrade-delivery websocket path. Publish one operator-facing validation runbook and one hard-fail docs contract that capture the exact commands and the exact limits of the claim.

## Inputs/Outputs
- Inputs:
  - running live Solana-backed bridge config through the existing service-api live bridge path
  - one presence-mode websocket consumer attached to the real service-api runtime
  - the existing live bridge proof lane already validated by current `main`
- Outputs:
  - one passing integration proof for presence-mode live bridge projection
  - one validation runbook under `docs/validation/`
  - one hard-fail docs contract under `crates/kamn-node/tests/`
  - updated runtime-proof index entry linking the runbook

## Boundaries/Non-goals
- Do not claim live on-chain settlement.
- Do not claim bridge finality beyond the existing bounded proofs.
- Do not add a new bridge runtime or new chain integration.
- Do not weaken the existing live bridge dispatch or live bridge websocket upgrade proofs.

## Failure Modes
- runbook omits the existing live bridge proof prerequisites
- docs contract omits the new runbook or proof-index marker
- presence-mode websocket stream fails to emit the live bridge projection event
- the proof overstates the claim as settlement, finality, or external economic movement

## Acceptance Criteria (testable booleans)
- [x] one docs contract fails when `docs/validation/live-solana-bridge-presence-stream-slice.md` is absent or missing required markers
- [x] one runbook exists that states the proof is bounded to live bridge projection on the presence stream
- [x] one integration test proves the presence-mode websocket consumer observes the live bridge projection event from the existing live bridge lane
- [x] `docs/validation/current-proven-runtime-slices.md` links the new runbook
- [x] the runbook explicitly states that the slice does not prove live on-chain settlement, bridge finality, or external economic settlement

## Files to Touch
- `specs/7007-live-bridge-presence-stream-slice.md`
- `docs/validation/live-solana-bridge-presence-stream-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/*presence*slice*contract*.rs`
- one real integration test under the existing websocket presence-mode test surface

## Error Semantics
- The integration proof must fail hard if the presence-mode stream does not emit the expected bridge event.
- The docs contract must fail hard if the runbook or index omits required markers or overstates the claim.
- No silent fallback to the existing upgrade websocket proof is allowed.

## Test Plan
1. Red:
   - add docs contract for the missing runbook and proof-index markers; confirm failure
2. Green:
   - publish the runbook
   - add one real presence-mode integration proof against the existing live bridge lane
   - wire the proof index entry
3. Verification:
   - docs contract
   - the presence-mode integration proof
   - touched-Rust policy gate

## Notes
- This issue intentionally deepens the live bridge value-flow proof through a different real consumer surface.
- It is not an external settlement issue.

## Final Evidence
- Docs contract:
  - `cargo test -p kamn-node --test live_solana_bridge_presence_stream_slice_contract -- --nocapture`
- Integration proof:
  - `cargo test -p kamn-node --bin kamn-node 'main_tests::service_api_endpoint_tests::websocket_contract_tests::presence_projection_contract_tests::live_bridge_presence_stream_contract_tests::integration_service_api_endpoint_websocket_presence_mode_streams_live_bridge_projection_event' -- --exact --nocapture`
- Review-surface contract:
  - `cargo test -p kamn-core --test corrected_audit_response_docs -- --nocapture`
- Touched-Rust policy:
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/7007-touched-size.json`
  - result: `policy_decision=GO`

## Deviations
- No behavioral deviation from the spec.
- The proof remains explicitly bounded to live bridge projection on the presence-mode websocket surface. It does not claim settlement, finality, or external economic movement.
