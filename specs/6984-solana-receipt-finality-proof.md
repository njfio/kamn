# 6984-solana-receipt-finality-proof

## Objective
Add Solana receipt-finality normalization to the public cross-chain receipt surface and prove one bounded operator-facing Solana receipt-finality slice on `main`, without overstating live devnet or settlement guarantees.

## Inputs/Outputs
- Inputs:
  - current receipt-finality normalization surface in `crates/kamn-bridges/src/cross_chain_receipt.rs`
  - current bridge-finality proof and runtime proof index docs on `main`
  - existing Solana bridge routing/quorum tests in `crates/kamn-core/tests/cross_chain_bridge.rs` and `bridge_outbound_quorum_execution.rs`
- Outputs:
  - Solana-aware receipt-finality normalization on the public core surface
  - executable Solana finality tests covering final, pending, failed, and invalid-label cases
  - one dedicated validation doc under `docs/validation/`
  - one hard-fail docs contract binding the proof to the exact Solana anchors
  - proof-index wiring for discoverability

## Boundaries/Non-goals
- Do not claim live Solana devnet proof yet.
- Do not claim live chain-backed settlement.
- Do not redesign bridge routing or quorum behavior.
- Do not add dependencies.
- Do not expand this into full bridge settlement or economic-finality guarantees.

## Failure modes
- Solana is added to the enum surface but normalization rules remain ambiguous or undocumented.
- Final, pending, failed, and invalid-label behavior for Solana is not executable on the public core surface.
- The new proof overstates live chain-backed proof or devnet readiness.
- The docs contract can pass while the proof drifts away from the Solana test anchors.

## Acceptance criteria (testable booleans)
- [ ] `CrossChainReceiptNetwork` supports Solana on the public receipt-normalization surface.
- [ ] Solana success normalization accepts explicit finality labels and maps them deterministically to `Final` or `Pending`.
- [ ] Solana failed and pending receipt statuses remain fail-closed regardless of label.
- [ ] Unsupported Solana finality labels return typed `UnsupportedFinalityLabel` errors.
- [ ] A dedicated validation doc exists under `docs/validation/` for the bounded Solana receipt-finality slice.
- [ ] A hard-fail docs contract enforces the required Solana proof anchors and bounded language.
- [ ] The finished proof is wired into `docs/validation/current-proven-runtime-slices.md`.

## Files to touch
- `specs/6984-solana-receipt-finality-proof.md`
- `crates/kamn-bridges/src/cross_chain_receipt.rs`
- `crates/kamn-core/tests/cross_chain_receipt_finality.rs`
- one new validation doc under `docs/validation/`
- one new docs contract under `crates/kamn-node/tests/` or `crates/kamn-core/tests/`
- `docs/validation/current-proven-runtime-slices.md`

## Error semantics
- Unknown Solana finality labels must fail with typed `UnsupportedFinalityLabel` errors.
- The proof must not silently equate normalized Solana labels with live devnet evidence or settlement guarantees.

## Test plan
- Phase 3 red tests that fail because Solana is not yet supported on the public receipt-normalization surface and the validation doc/contract do not exist.
- Green by implementing minimal Solana normalization rules and publishing the bounded proof.
- Re-run the Solana receipt-finality tests, the new docs contract, and touched-Rust policy before publish.
