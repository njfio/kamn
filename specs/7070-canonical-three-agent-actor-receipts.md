# 7070 Canonical Three-Agent Actor Observation Receipts

## Objective

Make the canonical MVP three-agent proof include first-class observation
receipt artifacts for Agent A, Agent B, and Agent C verifier. The direct
`verify-mvp-demo` path should not rely on optional Pi harness evidence to prove
that each actor had a bounded, actor-specific view of the same devnet-backed
escrow settlement.

## Inputs/Outputs

Inputs:
- MVP proof report JSON.
- `three_agent_escrow_verification` claim when present.
- Agent A, Agent B, and Agent C verifier view artifacts.
- Three-agent transaction transcript artifact.

Outputs:
- Agent A, Agent B, and Agent C observation receipt artifacts under the MVP
  run proof directory.
- Receipt artifact paths and `sha256:<hex>` digest fields on the
  `three_agent_escrow_verification` claim.
- Verifier failures when a required receipt is missing, stale, bound to the
  wrong actor, bound to the wrong view digest, or leaks participant-private
  data to Agent C.

## Boundaries/Non-goals

- Do not build new escrow, settlement, exchange, runtime, Pi, or MCP
  architecture.
- Do not change Solana devnet settlement execution, RPC configuration, keypair
  handling, or funding behavior.
- Do not make optional Pi harness evidence mandatory for direct MVP report
  verification.
- Do not broaden claims to production readiness, mainnet, generalized escrow,
  compliance, or real-value asset movement.
- Do not weaken claim labels, tests, formatting, clippy, `make check`, or proof
  semantics.

## Failure Modes

- A devnet-backed three-agent report verifies even though Agent A, Agent B, and
  Agent C receipts are absent.
- A receipt digest is a marker string instead of a verifier-computed
  `sha256:<hex>` digest.
- A receipt artifact is edited while the report digest remains stale and
  `verify-mvp-demo` still passes.
- Agent A or Agent B receipt points at the wrong participant view artifact or
  view digest.
- Agent C receipt points at a participant-private view, includes a
  `participant_private_view_digest`, or exposes raw private payload.
- Receipt actor identity or action sequence drifts from the transcript story.

## Acceptance Criteria

- [ ] Devnet-backed three-agent MVP reports include Agent A, Agent B, and Agent
  C observation receipt artifact paths and `sha256:<hex>` digest fields.
- [ ] The demo writes receipt artifacts under `.kamn/demo/<run-id>/proof/`
  whenever `three_agent_escrow_verification` is claimed.
- [ ] Agent A and Agent B receipts are `participant-private`, bind to their own
  view artifact and view digest, and may include participant-private digest
  references without raw private payload.
- [ ] Agent C receipt is `restricted-public`, binds to the verifier view
  artifact and view digest, and omits `participant_private_view_digest` and raw
  private payload.
- [ ] `verify-mvp-demo` rejects missing, stale, mismatched, wrong-actor, or
  private-leaking receipt artifacts.
- [ ] Existing transcript, view, actor identity, artifact digest, Pi harness,
  local artifact, devnet settlement, formatting, strict clippy, `make check`,
  local demo, and devnet-required demo evidence remain green.

## Files To Touch

- `crates/kamn-e2e-harness/src/mvp_demo/artifact_digest.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/mod.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report_artifacts.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_claim.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_receipts.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_transcript.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_receipt_contract.rs`
- `crates/kamn-e2e-harness/tests/support/three_agent_view_artifacts.rs`
- `specs/7070-canonical-three-agent-actor-receipts.md`

## Error Semantics

- Missing receipt artifact fields fail closed with explicit `Err(String)`
  messages.
- Receipt digest checks must recompute the digest from the artifact content,
  excluding the embedded receipt digest field to avoid self-reference.
- Actor identity, action, view scope, view artifact, view digest, and shared
  settlement fields must be compared against the report claim and existing view
  artifacts.
- Agent C receipt privacy violations are verifier failures. No fallback may
  downgrade the receipt to local-only or optional evidence when
  `three_agent_escrow_verification` is present.

## Test Plan

Red:
- Add a contract test proving `verify-mvp-demo` currently accepts a
  devnet-backed three-agent report without canonical actor receipt fields.
- Add tests proving stale receipt content, mismatched Agent A/B view digest,
  wrong actor identity, and Agent C private digest exposure are rejected.
- Add generated-report coverage requiring receipt digest claims to start with
  `sha256:`.

Green:
- Add a focused `three_agent_receipts` module to write and validate receipt
  artifacts.
- Generate receipt artifacts after view artifacts exist and before the
  three-agent claim is rendered.
- Add receipt paths and computed digests to the
  `three_agent_escrow_verification` claim and artifact index.
- Recompute and compare receipt digests during `verify-mvp-demo`.

Refactor:
- Reuse existing artifact digest and view validation helpers where practical.
- Keep Pi actor tool receipt validation separate from canonical proof receipt
  validation.
- Keep touched files within repo size guidance by splitting receipt-specific
  helpers into their own module.

Integration:
- Run `mvp_demo_three_agent_receipt_contract`.
- Run transcript, view artifact, view digest, claim, command, local artifact,
  and agent harness MVP proof contract tests.
- Run `cargo fmt --check`.
- Run strict workspace clippy.
- Run `make check`.
- Run local `make demo-mvp` and canonical `verify-mvp-demo`.
- Run the devnet-required demo and canonical verifier, or record explicit
  external NO-GO evidence if Solana devnet is unavailable.
