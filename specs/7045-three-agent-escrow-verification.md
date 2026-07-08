# 7045 Three-Agent Escrow Verification Proof Contract

## Objective
Add an MVP proof-report contract for the intended three-agent KAMN escrow story:
two participant agents transact through escrow/settlement, and a third verifier
agent validates the transaction from a restricted proof view. The contract must
prove that the verifier can validate public commitments without receiving the
participant-private context visible to the original agents.

## Inputs/Outputs
Inputs:
- MVP demo proof report JSON emitted by `make demo-mvp`
- Current devnet settlement claim evidence when settlement or asset movement is
  claimed
- Three-agent perspective proof fields embedded in the report claim matrix

Outputs:
- `verify-mvp-demo` fails when the three-agent proof claim is absent or
  malformed
- `verify-mvp-demo` passes when participant and verifier views share matching
  commitments and the verifier view excludes private fields
- The human-readable report keeps the claim boundary clear

## Boundaries/Non-goals
- Do not claim production readiness, mainnet settlement, generalized dispute
  resolution, or broad bridge finality.
- Do not add a new service architecture or new dependency.
- Do not weaken existing local-only, devnet-backed, dry-run, placeholder, or
  roadmap label semantics.
- This issue establishes the report/verifier contract and demo evidence shape.
  A later issue may drive the same path through a fully live Pi/MCP
  multi-agent orchestration.

## Failure modes
- The report omits the three-agent escrow verification claim.
- The verifier perspective is missing.
- The verifier perspective leaks participant-private fields.
- Participant and verifier views disagree on transaction id, terms digest,
  escrow id, settlement signature, settlement commitment, finality, or amount.
- Settlement/value movement is labeled local-only, dry-run, placeholder, or
  roadmap while being counted as success.
- The claim uses placeholder settlement evidence or omits devnet-backed markers.

## Acceptance criteria
- [ ] The verifier rejects reports without `three_agent_escrow_verification`.
- [ ] The verifier rejects reports where `verifier_private_view_visible` is true.
- [ ] The verifier rejects reports with mismatched shared commitments across
  participant and verifier views.
- [ ] The verifier rejects reports where the three-agent claim is not
  `devnet-backed` when it references settlement/value movement.
- [ ] The generated MVP report includes one passing
  `three_agent_escrow_verification` claim when devnet settlement evidence is
  present.
- [ ] The generated MVP report includes participant views for `agent_a` and
  `agent_b`, a restricted `agent_c_verifier` view, and shared commitment fields.
- [ ] `cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract` passes.
- [ ] `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report
  .kamn/demo/latest/proof/report.json` passes after `make demo-mvp`.

## Files to touch
- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/verify.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_claim_contract.rs`
- Optional helper extraction under `crates/kamn-e2e-harness/src/mvp_demo/`

## Error semantics
- Missing or malformed three-agent evidence must return an explicit
  `Err(String)` from the verifier.
- Verifier-view privacy leaks must fail closed.
- Shared-commitment mismatches must fail closed and identify the mismatched
  commitment class.
- No fallback may convert missing devnet-backed settlement evidence into a
  local-only success.

## Test plan
Red:
- Add negative verifier tests for missing claim, verifier private leak,
  mismatched commitments, and non-devnet-backed three-agent settlement labels.
- Add a positive fixture test with matching participant/verifier commitments.

Green:
- Extend the MVP report claim matrix with
  `three_agent_escrow_verification` when devnet settlement evidence exists.
- Extend verifier checks to validate the required three-agent fields and
  privacy/commitment invariants.

Refactor:
- Extract small claim-marker helpers if needed to keep verifier functions short
  and focused.

Integration:
- Run targeted claim-contract tests.
- Run the canonical report verifier against the latest demo report.
- Run `make demo-mvp` if the branch reaches integration wiring.
