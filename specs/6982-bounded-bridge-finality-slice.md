# 6982-bounded-bridge-finality-slice

## Objective
Prove one current, operator-comprehensible KAMN bounded bridge finality slice on `main` that states exactly which bridge-finality-related behavior is executable today through deterministic receipt-finality normalization and service-api forwarded-bridge persistence, without overstating live chain-backed finality.

## Inputs/Outputs
- Inputs:
  - current receipt-finality normalization coverage in `crates/kamn-core/tests/cross_chain_receipt_finality.rs`
  - current service-api bridge persistence restart coverage in `crates/kamn-node/src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests/`
  - current runtime proof index and corrected audit response docs on `main`
- Outputs:
  - one dedicated bridge-finality validation doc under `docs/validation/`
  - one hard-fail docs contract binding that doc to the exact executable bridge anchors
  - minimal docs wiring so the proof is discoverable from the current runtime proof surfaces

## Boundaries/Non-goals
- Do not claim live chain-backed bridge finality.
- Do not claim external settlement on real networks.
- Do not redesign bridge persistence or receipt normalization.
- Do not add dependencies.
- Do not expand this into full bridge quorum or settlement guarantees.

## Failure modes
- The proof claims stronger bridge finality than current executable coverage supports.
- The proof omits either receipt-finality normalization or persisted forwarded bridge state.
- The docs contract can pass while the validation doc drifts away from the executable bridge anchors.
- The runtime proof surfaces continue to describe all bridge finality as unproven after this bounded proof is added.

## Acceptance criteria (testable booleans)
- [ ] A dedicated bridge-finality proof doc exists under `docs/validation/`.
- [ ] The proof links current executable coverage for Ethereum and Near receipt-finality normalization on the public core surface.
- [ ] The proof links current executable coverage for persisted `forwarded` bridge state across restart on the service-api path.
- [ ] The proof states clearly that it proves deterministic receipt-finality normalization and persisted forwarded bridge state, not live chain-backed bridge finality or external settlement.
- [ ] A hard-fail docs contract enforces the required proof anchors and bounded language.
- [ ] The finished proof is wired into `docs/validation/current-proven-runtime-slices.md`.

## Files to touch
- `specs/6982-bounded-bridge-finality-slice.md`
- one new validation doc under `docs/validation/`
- one new docs contract under `crates/kamn-node/tests/` or `crates/kamn-core/tests/`
- `docs/validation/current-proven-runtime-slices.md`
- optionally one existing review/docs surface for discoverability wiring

## Error semantics
- Missing proof anchors or missing boundary markers must fail loudly.
- The proof must not silently equate deterministic receipt normalization with live bridge finality.

## Test plan
- Phase 3 red test that fails because the bridge-finality validation doc and contract do not yet exist.
- One hard-fail docs contract asserting required bridge proof links and bounded-language markers.
- Re-run the referenced receipt-finality and bridge restart targets and touched-Rust policy before publish.

## Final implementation
- Validation runbook: [docs/validation/bridge-finality-slice.md](/home/n/Code/kamn/docs/validation/bridge-finality-slice.md)
- Hard-fail docs contract: [bridge_finality_slice_contract.rs](/home/n/Code/kamn/crates/kamn-node/tests/bridge_finality_slice_contract.rs)
- Runtime proof index wiring: [current-proven-runtime-slices.md](/home/n/Code/kamn/docs/validation/current-proven-runtime-slices.md)
- Review-surface wiring: [corrected-audit-response-2026-03-14.md](/home/n/Code/kamn/docs/review/corrected-audit-response-2026-03-14.md)

## Final evidence
- `cargo test -p kamn-node --test bridge_finality_slice_contract -- --nocapture`
- `cargo test -p kamn-core --test cross_chain_receipt_finality -- --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_persists_bridge_state_across_restart -- --exact --nocapture`
- `cargo test -p kamn-core --test corrected_audit_response_docs -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6982-touched-size.json`
- touched-Rust result: `policy_decision=GO`

## Observed proof
- Ethereum `finalized` and Near `final` receipts normalize to `Final` on the public core surface
- pending and failed receipt states remain fail-closed on the public core surface
- unknown Near finality labels are rejected rather than normalized loosely
- service-api bridge state reaches and retains `forwarded` state with stable target-message and forward-hash evidence across restart
- the proof is explicitly bounded to deterministic receipt-finality normalization and persisted forwarded bridge state rather than live chain-backed finality

## Deviations
- None. Current `main` already contained the necessary receipt-normalization and bridge-persistence coverage; this issue formalized it as one operator-readable bounded bridge-finality proof.
