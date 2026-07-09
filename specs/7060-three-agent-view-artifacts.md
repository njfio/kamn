# 7060 Three-Agent View Artifacts

## Objective

Add durable per-agent view artifacts for the devnet-backed three-agent MVP
proof. The current report proves the three-agent boundary through inline claim
fields and one transcript artifact. This issue makes Agent A, Agent B, and
Agent C inspectable as separate views over the same transaction, escrow, and
Solana devnet settlement evidence.

## Inputs/Outputs

Inputs:
- MVP `run_id`.
- Successful `DevnetSettlementEvidence`.
- Existing `three_agent_escrow_verification` claim.
- Existing `three-agent-transcript.json`.

Outputs:
- `.kamn/demo/<run-id>/proof/agent-a-view.json`.
- `.kamn/demo/<run-id>/proof/agent-b-view.json`.
- `.kamn/demo/<run-id>/proof/agent-c-verifier-view.json`.
- Report artifact entries for all three view artifacts.
- Transcript and claim fields that bind the three view artifacts by digest or
  path.
- Verifier errors when the report, transcript, or view artifacts disagree.

## Boundaries/Non-goals

- Do not build a production privacy system or generalized disclosure engine.
- Do not change Solana devnet settlement execution semantics.
- Do not add dependencies.
- Do not claim mainnet, production readiness, generalized exchange,
  consensus/finality, or broad privacy guarantees.
- Do not expose raw participant-private payloads.

## Failure Modes

- Devnet-backed report omits any Agent A/B/C view artifact entry.
- Transcript or claim omits the per-agent view artifact binding fields.
- A participant view is missing, malformed, or not labelled
  `participant-private`.
- Agent C view is missing, malformed, not labelled `restricted-public`, or
  exposes participant-private payloads/private digests.
- Any view artifact settlement signature, amount, payer, recipient, commitment,
  transaction id, or escrow id differs from the transcript or report claim.
- View artifact digests in the transcript or claim do not match the artifact
  contents.
- Local-only reports incorrectly require or claim three-agent view artifacts.

## Acceptance Criteria

- [ ] Devnet-backed reports include `agent_a_view`, `agent_b_view`, and
  `agent_c_verifier_view` artifact entries.
- [ ] `three_agent_escrow_verification` includes per-agent view artifact paths
  and view digests.
- [ ] `three-agent-transcript.json` includes the same per-agent view digests.
- [ ] Agent A and Agent B view artifacts are labelled `participant-private`,
  include nonzero participant-private field counts, and do not include raw
  private payloads.
- [ ] Agent C view artifact is labelled `restricted-public`, includes the
  shared public verification digest and devnet settlement evidence, and includes
  no private payloads or participant-private view digests.
- [ ] `verify-mvp-demo` rejects missing, mismatched, raw-private-leaking, or
  over-disclosing view artifacts.
- [ ] Local-only reports remain valid without three-agent view artifacts.
- [ ] Existing devnet-backed settlement, transcript, Pi/agent harness boundary,
  local artifact binding, formatting, clippy, and `make check` gates remain
  green.

## Files To Touch

- `crates/kamn-e2e-harness/src/mvp_demo/report_artifacts.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_claim.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_transcript.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_views.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/mod.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_view_artifact_contract.rs`
- Existing three-agent, agent harness, report, and runbook tests as needed.

## Error Semantics

- Missing or malformed per-agent view artifacts fail closed with explicit
  `Err(String)` messages.
- Command-level verification must read and validate the view artifacts from the
  report, not trust inline claim fields alone.
- Any raw private payload marker in any view artifact fails verification.
- Agent C receiving participant-private scope, private field count, raw private
  payload, or participant-private digest fails verification.
- Missing view artifacts in local-only reports are allowed because local-only
  reports do not claim settlement or three-agent verification.

## Test Plan

Red:
- Add command-level verifier tests that reject devnet-backed reports missing
  Agent A/B/C view artifacts.
- Add tests that reject Agent C over-disclosure and raw private payload leakage.
- Add tests that reject mismatched settlement signatures or view digests between
  the report claim, transcript, and view artifacts.
- Add a local-only acceptance case showing no view artifacts are required when
  three-agent settlement is not claimed.

Green:
- Generate per-agent view artifacts after successful devnet settlement evidence.
- Add report artifact entries and claim/transcript binding fields.
- Add view artifact validation to the command-level verifier.
- Keep local-only mode unchanged.

Refactor:
- Keep path generation, view JSON generation, and verification helpers split
  into small functions/files.
- Reuse existing flat marker/extractor helpers; do not add a JSON dependency.
- Keep all touched files under the repo's line-budget guidance.

Integration:
- Run the new view artifact contract test.
- Run existing MVP claim, three-agent transcript, three-agent claim, agent
  harness, local artifact, and runbook tests.
- Run `cargo fmt --check`, strict workspace clippy, and `make check`.
- Run local `make demo-mvp` plus canonical verifier.
- Run devnet-required `make demo-mvp` plus canonical verifier, or record an
  explicit Solana devnet NO-GO if external devnet is unavailable.
