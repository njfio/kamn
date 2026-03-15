# 6986-solana-devnet-bridge-smoke

## Objective
Prove one bounded operator-facing Solana devnet-addressed bridge smoke slice on current `main` by surfacing the existing executable Solana bridge path through a validation runbook and hard-fail docs contract, without overstating live Solana RPC/devnet settlement guarantees.

## Inputs/Outputs
- Inputs:
  - existing Solana bridge routing and quorum tests in `crates/kamn-core/tests/cross_chain_bridge.rs`
  - existing Solana outbound quorum coverage in `crates/kamn-core/tests/bridge_outbound_quorum_execution.rs`
  - current runtime proof index in `docs/validation/current-proven-runtime-slices.md`
- Outputs:
  - one new validation runbook under `docs/validation/`
  - one hard-fail docs contract binding the runbook to exact Solana bridge smoke anchors
  - proof-index wiring describing exactly what the slice proves
  - explicit bounded language that distinguishes devnet-addressed bridge proof from live Solana devnet-backed proof

## Boundaries/Non-goals
- Do not claim live Solana RPC/devnet-backed proof unless discovered and exercised from checked-in runtime surface.
- Do not claim bridge settlement finality or external fund movement.
- Do not redesign the cross-chain bridge engine.
- Do not add dependencies.
- Do not weaken existing bridge or finality tests.

## Failure modes
- The new runbook overstates a live Solana devnet proof when the executable evidence is only devnet-addressed on the public bridge surface.
- The docs contract can pass while drifting away from the real Solana bridge smoke anchors.
- The proof index links the slice without stating the bounded semantics clearly.
- The surfaced proof omits fail-closed replay or approval rejection behavior that is already part of the Solana bridge path.

## Acceptance criteria (testable booleans)
- [ ] A dedicated validation doc exists at `docs/validation/solana-devnet-bridge-smoke-slice.md`.
- [ ] A hard-fail docs contract enforces required markers for the Solana route, outbound quorum dispatch, inbound projection, replay rejection, and bounded non-goals.
- [ ] The runbook anchors to executable current-main tests that use the Solana devnet route id `solana:devnet:program:task-v1`.
- [ ] The runbook states explicitly whether the proof is live devnet-backed or only devnet-addressed on current `main`.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the new slice with bounded wording.
- [ ] The surfaced proof includes at least one outbound quorum success path and one fail-closed replay or approval rejection path.

## Files to touch
- `specs/6986-solana-devnet-bridge-smoke.md`
- `docs/validation/solana-devnet-bridge-smoke-slice.md`
- one new docs contract under `crates/kamn-node/tests/` or `crates/kamn-core/tests/`
- `docs/validation/current-proven-runtime-slices.md`
- optionally `docs/review/corrected-audit-response-2026-03-14.md` if the review surface should mention the new proof

## Error semantics
- The proof must use bounded language and must fail closed on unsupported claims.
- If current `main` does not expose a live Solana devnet-backed path, the runbook must say so explicitly rather than implying it.
- Replay and unauthorized-approver behavior must remain explicit fail-closed errors in the cited evidence.

## Test plan
- Phase 3 red tests: add a docs contract that fails because the runbook and proof-index markers do not exist yet.
- Green by publishing the bounded runbook and linking it from the proof index.
- Re-run the existing Solana bridge tests named by the runbook plus the new docs contract.
- Run touched-Rust policy before publish.
