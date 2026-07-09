# Issue 7072: Pi Agent Canonical Receipts

## Objective

Require the Pi/project-local agent harness evidence path to prove the same
canonical three-agent observation receipt contract that the direct MVP demo
verifier already enforces for devnet-backed three-agent escrow verification.

## Inputs/Outputs

- Input: MVP proof report JSON passed to `verify-mvp-demo`.
- Input: optional `agent_harness_evidence` artifact referenced by the report.
- Input: canonical Agent A, Agent B, and Agent C observation receipt artifacts
  referenced by the `three_agent_escrow_verification` claim and artifact index.
- Output: `verify-mvp-demo` returns `PASS` only when direct report validation and
  agent-harness evidence both agree with the canonical receipt artifacts.
- Output: Pi extension evidence includes human-readable canonical receipt
  references so an evaluator can inspect which proof artifacts the agent path
  drove or verified.

## Boundaries/Non-goals

- Do not add another settlement or escrow architecture.
- Do not add another agent framework or dependency.
- Do not change local-only, optional-devnet, or devnet-required claim semantics.
- Do not turn dry-run, placeholder, or local-only value movement into
  devnet-backed success.
- Do not weaken direct report verification, clippy, formatting, or existing
  harness evidence checks.

## Failure Modes

- A report has `mcp_agent_harness_verification` and a devnet-backed
  `three_agent_escrow_verification`, but the harness evidence omits canonical
  receipt references.
- Harness evidence references a receipt artifact or digest that does not match
  the report claim.
- Agent C harness-visible receipt data leaks participant-private view digests or
  raw private payload.
- The Pi extension docs or source imply generic harness success without naming
  the canonical receipt boundary.

## Acceptance Criteria

- [ ] `verify-mvp-demo` rejects agent-harness evidence for a devnet-backed
      three-agent claim when canonical observation receipt references are absent
      from the harness evidence artifact.
- [ ] `verify-mvp-demo` rejects agent-harness evidence when canonical receipt
      references in the artifact mismatch the report claim.
- [ ] `verify-mvp-demo` rejects Agent C verifier receipt evidence that includes
      participant-private digest or raw private payload markers.
- [ ] Valid Pi extension evidence containing canonical receipt references still
      verifies.
- [ ] The project-local Pi extension source and evaluator runbook name the
      canonical receipt artifacts/digests as part of the agent-driven proof.

## Files To Touch

- `.pi/extensions/kamn-mvp/evidence.ts`
- `.pi/extensions/kamn-mvp/actor-receipts.ts`
- `.pi/extensions/kamn-mvp/index.ts`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness_three_agent.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness_actor_receipts.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/*`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

- Missing canonical receipt evidence fails loudly with an error naming
  `three_agent_actor_observation_receipts`.
- Mismatched report-vs-evidence fields fail loudly with the specific receipt
  field context.
- Agent C private marker leakage fails loudly with `agent_c_verifier`.
- Direct reports without agent-harness evidence continue to verify through the
  direct canonical receipt verifier only.

## Test Plan

- Red: add negative tests for missing harness canonical receipt evidence,
  mismatched canonical receipt digest references, and Agent C private leakage.
- Red: add source/runbook marker tests requiring Pi guidance to name canonical
  observation receipt evidence.
- Green: extend Pi extension evidence to emit canonical receipt references and
  extend Rust harness evidence validation to check those references.
- Refactor: keep added files/functions small, reuse existing receipt verifier
  helpers, and avoid duplicate string parsing where local helpers already exist.
- Integration: run the focused agent-harness contract, the receipt contract, the
  broader MVP contract matrix, `cargo fmt --check`, strict clippy, `make check`,
  and the canonical local/devnet demo verifier paths.
