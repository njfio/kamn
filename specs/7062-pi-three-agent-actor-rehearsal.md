# 7062 Pi Three-Agent Actor Rehearsal

## Objective

Add actor-specific Pi/MCP harness evidence for the devnet-backed three-agent
MVP proof. The current harness evidence proves that Pi extension tools can drive
the aggregate verifier path. This issue makes the actor rehearsal explicit:
Agent A, Agent B, and Agent C each record the step they performed and bind their
observation to the report's per-agent view artifacts.

## Inputs/Outputs

Inputs:
- MVP proof report JSON.
- `three_agent_escrow_verification` claim when present.
- Agent A, Agent B, and Agent C view artifact paths/digests from the report
  claim.
- Existing Pi extension evidence artifact.

Outputs:
- Optional `three_agent_actor_rehearsal` object in the Pi agent-harness
  evidence artifact.
- Actor observations for:
  - `agent_a`: registration plus transaction/task invocation.
  - `agent_b`: registration plus task acceptance.
  - `agent_c_verifier`: restricted-public verification.
- Verifier errors when actor observations are missing, mismatched, overclaiming,
  or inconsistent with devnet-backed settlement/view evidence.

## Boundaries/Non-goals

- Do not build a production multi-agent runtime or generalized orchestration
  system.
- Do not change Solana devnet settlement execution semantics.
- Do not add dependencies.
- Do not claim mainnet, production readiness, generalized exchange,
  consensus/finality, or broad privacy guarantees.
- Do not require actor rehearsal evidence for local-only reports that do not
  claim `three_agent_escrow_verification`.
- Do not expose raw participant-private payloads.

## Failure Modes

- Pi evidence claims three-agent success but omits `three_agent_actor_rehearsal`.
- Actor rehearsal omits Agent A, Agent B, or Agent C verifier observations.
- Agent A or Agent B observation lacks registration or its expected flow action.
- Agent C observation lacks `verify_proof` or claims participant-private scope.
- Any actor view artifact path or view digest differs from the report claim.
- Actor rehearsal says settlement is local-only, dry-run, placeholder, or absent
  when the report claims three-agent devnet-backed success.
- Actor rehearsal exposes raw private payload markers.

## Acceptance Criteria

- [ ] Pi extension evidence can include `three_agent_actor_rehearsal` with
  explicit Agent A, Agent B, and Agent C verifier observations.
- [ ] Agent A observation proves `register` plus `invoke_transaction` and binds
  to the Agent A view artifact path and digest.
- [ ] Agent B observation proves `register` plus `accept_task` and binds to the
  Agent B view artifact path and digest.
- [ ] Agent C observation proves `verify_proof`, binds to the Agent C verifier
  view artifact path and digest, and preserves `restricted-public` scope.
- [ ] `verify-mvp-demo` rejects missing, mismatched, dry-run/placeholder,
  local-only settlement, raw-private-leaking, or over-disclosing actor
  rehearsal evidence.
- [ ] Local-only reports remain valid with no actor rehearsal object.
- [ ] Existing MVP demo, devnet settlement, view artifact, Pi extension,
  formatting, strict clippy, and `make check` gates remain green.

## Files To Touch

- `.pi/extensions/kamn-mvp/index.ts`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness_three_agent.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/*.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

- Missing actor rehearsal fields fail closed with explicit `Err(String)`
  messages.
- Actor rehearsal validation is only required when agent-harness evidence is
  present for a report that claims `three_agent_escrow_verification`.
- Actor rehearsal validation compares evidence fields against the verified
  report claim instead of trusting the evidence artifact alone.
- Agent C participant-private scope, private digest exposure, or nonzero private
  field count fails verification.
- Dry-run, placeholder, or local-only settlement labels in actor rehearsal fail
  verification when three-agent success is claimed.

## Test Plan

Red:
- Add command-level verifier tests rejecting Pi harness evidence with no
  `three_agent_actor_rehearsal` for a devnet-backed three-agent report.
- Add tests rejecting missing Agent A/B/C observations.
- Add tests rejecting Agent C participant-private scope or private digest
  exposure.
- Add tests rejecting actor view artifact path/digest mismatches and
  local-only/dry-run settlement labels.
- Add/update extension contract tests requiring the Pi extension to emit the
  actor rehearsal object.

Green:
- Extend the Pi extension evidence writer to derive actor rehearsal fields from
  the report's three-agent claim.
- Extend the Rust verifier to validate actor rehearsal fields when a
  three-agent claim is present.
- Keep local-only reports and reports without agent-harness evidence unchanged.

Refactor:
- Keep actor-rehearsal validation in small helpers.
- Reuse existing flat marker/extractor helpers; do not add a JSON dependency.
- Keep touched Rust files under repo line-budget guidance.

Integration:
- Run the agent-harness contract tests.
- Run the three-agent view artifact, claim, transcript, local artifact, command,
  and runbook contract tests.
- Run `cargo fmt --check`, strict workspace clippy, and `make check`.
- Run local `make demo-mvp` and canonical verifier.
- Run devnet-required `make demo-mvp` and canonical verifier, or record explicit
  external NO-GO evidence if devnet is unavailable.
