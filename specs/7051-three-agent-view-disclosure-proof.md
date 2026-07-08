# 7051 Three-Agent View Disclosure Proof

## Objective
Deepen the MVP three-agent escrow proof so it shows differential visibility:
agent A and agent B each retain participant-private proof evidence, while the
third-party verifier can validate the same transaction, escrow, and settlement
from restricted public commitments without seeing participant-private evidence.

## Inputs/Outputs
Inputs:
- MVP proof report JSON emitted by `make demo-mvp`.
- The existing `three_agent_escrow_verification` claim.
- Devnet settlement evidence already required for escrow, settlement, or asset
  movement claims.

Outputs:
- `verify-mvp-demo` rejects reports without explicit view-scope evidence for
  agent A, agent B, and the verifier.
- `verify-mvp-demo` rejects reports where participant views are not richer than
  the verifier view.
- `verify-mvp-demo` rejects reports that leak verifier-private evidence or raw
  participant-private payloads to the verifier.
- The generated proof report includes explicit public-view and private-view
  digest fields without exposing raw private payloads.
- The human-readable report states the participant/verifier visibility boundary.

## Boundaries/Non-goals
- Do not add a new service architecture, settlement architecture, dispute flow,
  bridge, or mainnet behavior.
- Do not claim production readiness or generalized privacy guarantees.
- Do not claim that the verifier sees the same information as participants.
- Do not expose raw private payloads in the proof report.
- Do not weaken existing local-only, devnet-backed, dry-run, placeholder, or
  roadmap label semantics.
- Do not use fake in-memory value movement to satisfy settlement or escrow
  claims.

## Failure modes
- The report omits a view scope for agent A, agent B, or the verifier.
- Agent A or agent B lacks participant-private proof evidence.
- The verifier view contains private evidence or a verifier private-view digest.
- Agent A, agent B, and verifier public-view digests disagree.
- Private payload redaction is missing or false.
- Shared transaction, escrow, settlement, or amount commitments regress from the
  existing three-agent verifier rules.

## Acceptance criteria
- [ ] The verifier rejects three-agent claims missing any of
  `agent_a_view_scope`, `agent_b_view_scope`, or `verifier_view_scope`.
- [ ] The verifier rejects participant views where the private field count is
  zero or not greater than the verifier private field count.
- [ ] The verifier rejects verifier views where `verifier_private_field_count`
  is nonzero.
- [ ] The verifier rejects claims that include a `verifier_private_view_digest`.
- [ ] The verifier rejects claims where `private_payload_redacted` is false.
- [ ] The verifier rejects claims where `agent_a_public_view_digest`,
  `agent_b_public_view_digest`, and `verifier_public_view_digest` differ.
- [ ] The generated MVP report includes participant-private digest fields for
  agent A and agent B, a shared public-view digest, and no verifier private
  digest.
- [ ] The generated human-readable report calls out that participants see
  private evidence while the verifier validates restricted public commitments.
- [ ] `cargo test -p kamn-e2e-harness --test mvp_demo_three_agent_claim_contract -- --nocapture`
  passes.
- [ ] The canonical report verifier passes against the generated report.

## Files to touch
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_claim.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_verify.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_claim_contract.rs`

## Error semantics
- Missing disclosure fields must fail closed with explicit `Err(String)`
  verifier messages.
- Verifier private evidence must fail closed even when shared settlement
  commitments match.
- Public-view digest mismatches must fail closed and identify the mismatched
  commitment class.
- No fallback may convert missing devnet settlement evidence into local-only,
  dry-run, placeholder, or roadmap success.

## Test plan
Red:
- Add negative verifier tests for missing view scope, missing participant
  private count, verifier private count, verifier private digest leakage,
  missing redaction, and public-view digest mismatch.
- Update the positive fixture to include the intended differential disclosure
  fields only after observing the new negative tests fail on current code.

Green:
- Extend `three_agent_escrow_verification` generation with explicit view scope,
  private field counts, participant private-view digests, public-view digests,
  and redaction markers.
- Extend the three-agent verifier to validate the new disclosure invariants.
- Extend the human-readable report to summarize the visibility boundary.

Refactor:
- Keep verifier helpers small and focused.
- Reuse existing flat claim-field parsing patterns unless a narrower helper is
  required.

Integration:
- Run targeted three-agent contract tests.
- Generate an MVP demo report and run the canonical verifier against it.
- Run formatting, strict clippy, and `make check` before PR.

## Completion evidence
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `make check`
- `cargo test -p kamn-e2e-harness --test mvp_demo_three_agent_claim_contract -- --nocapture`
- `make demo-mvp`
- `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`

Deviations: none yet.
