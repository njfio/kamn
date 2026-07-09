# 7066 Bind Three-Agent View Identities

## Objective

Make the MVP three-agent proof verifier bind every per-agent view artifact to
the exact actor identity it represents. The report, transcript, Pi actor
receipts, and view artifacts should all use one stable actor vocabulary:
`agent_a`, `agent_b`, and `agent_c_verifier`.

## Inputs/Outputs

Inputs:
- MVP proof report JSON.
- `three_agent_escrow_verification` claim when present.
- Three-agent transcript artifact.
- Agent A, Agent B, and Agent C verifier view artifact files.

Outputs:
- Verifier failures when an Agent A, Agent B, or Agent C verifier view artifact
  declares the wrong `agent` value.
- Generated MVP demo view artifacts whose `agent` fields match the report,
  transcript, and Pi actor receipt vocabulary.

## Boundaries/Non-goals

- Do not build a production multi-agent runtime or scheduler.
- Do not change Solana devnet settlement execution semantics.
- Do not change devnet keypair handling.
- Do not broaden production readiness, mainnet, generic exchange, or broad
  escrow claims.
- Do not add dependencies.
- Do not weaken existing transcript, local artifact, receipt, claim, demo, or
  verifier semantics.

## Failure Modes

- Agent A view artifact declares any `agent` other than `agent_a`.
- Agent B view artifact declares any `agent` other than `agent_b`.
- Agent C verifier view artifact declares any `agent` other than
  `agent_c_verifier`.
- Generated demo artifacts drift back to `agent_c` while the rest of the
  three-agent proof surface uses `agent_c_verifier`.
- Verifier continues to accept swapped participant view identities because the
  settlement and digest fields still match.

## Acceptance Criteria

- [ ] `verify-mvp-demo` rejects an Agent A view artifact whose `agent` field is
  not `agent_a`.
- [ ] `verify-mvp-demo` rejects an Agent B view artifact whose `agent` field is
  not `agent_b`.
- [ ] `verify-mvp-demo` rejects the verifier view artifact unless its `agent`
  field is `agent_c_verifier`.
- [ ] `make demo-mvp` writes `agent_c_verifier` inside the Agent C verifier
  view artifact.
- [ ] Existing local-only reports, devnet-backed reports, Pi actor receipt
  evidence, transcript binding, local artifact binding, formatting, strict
  clippy, and `make check` remain green.

## Files To Touch

- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_views.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_view_artifacts.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_view_artifact_contract.rs`
- `crates/kamn-e2e-harness/tests/support/three_agent_view_artifacts.rs`
- `specs/7066-bind-three-agent-view-identities.md`

## Error Semantics

- Missing or mismatched view artifact identities fail closed with explicit
  `Err(String)` messages.
- Identity validation must compare the artifact's `agent` field directly. It
  must not infer identity only from file path, claim field, digest string, or
  transcript path.
- Agent C verifier identity drift fails verification even if the view remains
  restricted-public.
- No fallback should convert a malformed view identity into a successful
  devnet-backed three-agent claim.

## Test Plan

Red:
- Add a verifier test rejecting an Agent A view artifact with a wrong `agent`
  field.
- Add a verifier test rejecting an Agent B view artifact with a wrong `agent`
  field.
- Add a verifier test rejecting an Agent C verifier view artifact with
  `agent_c` instead of `agent_c_verifier`.

Green:
- Require exact actor identity markers in participant and verifier view
  validation.
- Change generated Agent C verifier view artifacts to declare
  `agent_c_verifier`.
- Update fixtures to use the same identity vocabulary.

Refactor:
- Keep identity checks in small helpers beside existing view artifact
  validation.
- Reuse existing marker/extractor helpers.
- Keep touched Rust files under repo line-budget guidance.

Integration:
- Run `mvp_demo_three_agent_view_artifact_contract`.
- Run the broader MVP proof contract matrix covering claim, transcript, local
  artifact, agent harness, command, and evaluator runbook behavior.
- Run `cargo fmt --check`.
- Run strict workspace clippy.
- Run `make check`.
- Run local `make demo-mvp` and canonical `verify-mvp-demo`.
- Run the devnet-required demo and canonical verifier, or record explicit
  external NO-GO evidence if devnet is unavailable.

## Completion Evidence

To be filled during Phase 7.
