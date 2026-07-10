# Bind Independent Agent Task To Devnet Escrow

## Objective

Make the independent Pi A/B/C task and Solana devnet settlement one causal,
verifiable story by placing the validated task-binding digest in the escrow
funding request and propagating the actual service escrow ID through proof.

## Inputs/Outputs

Inputs:

- Existing live task handoff, Agent A/B receipts, and restricted Agent C
  observation paths from issue 7084.
- Existing funded Solana devnet settlement configuration.
- Devnet-required canonical MVP execution.

Outputs:

- `live-task-settlement-binding.json` copied into the run proof directory with
  task ID, accepted state, three process IDs, source digests, Agent C public
  commitment, and a tagged binding digest.
- Escrow funding payload v2 containing the task ID and binding digest.
- Devnet evidence containing the actual returned/persisted service escrow ID.
- Three-agent transcript, views, receipts, and report claim using the live task
  ID and actual service escrow ID.

## Boundaries/Non-goals

- Validate existing issue 7084 artifacts; do not invent a second actor format.
- Standalone devnet settlement may run without live task evidence, but it must
  not emit `three_agent_escrow_verification` in that mode.
- Three-agent settlement proof requires both devnet evidence and live binding.
- Do not claim server-side view authorization, mainnet, economic value,
  generalized exchange, disputes, refunds, or production readiness.
- Do not add dependencies or change Solana transfer semantics.

## Failure Modes

- Partial live-task path configuration: fail before demo work.
- Missing, malformed, unknown-field, stale, altered, or secret-like source:
  fail without attempting settlement.
- Task/state/source-digest/PID/view-scope/redaction mismatch: fail closed.
- Binding output conflicts or aliases a source path: fail closed.
- Escrow funding payload omits or changes task/binding fields: fail.
- Returned escrow ID does not match the deterministic funding payload: fail.
- Persisted escrow ID/signature differs from returned evidence: fail.
- Report/transcript/view/receipt task, binding, escrow, or settlement mismatch:
  fail canonical verification.

## Acceptance Criteria

- [ ] All four issue 7084 artifacts are revalidated by the Rust harness.
- [ ] A canonical binding artifact is written before settlement starts.
- [ ] The exact binding digest and task ID enter the escrow funding request.
- [ ] Devnet evidence records the actual returned service escrow ID.
- [ ] Three-agent artifacts use live task ID and actual escrow ID everywhere.
- [ ] Report and artifact indexes include the binding path and digest.
- [ ] Canonical verification revalidates binding content and cross-artifact IDs.
- [ ] Missing binding suppresses the three-agent claim without weakening the
  standalone `devnet_settlement_asset_movement` claim.
- [ ] Negative tests reject forged binding, source, escrow, and report fields.
- [ ] A funded run confirms the transaction and canonical verifier passes.
- [ ] Pi verifies the final report without rerunning settlement.
- [ ] Formatting, strict clippy, `make check`, and targeted tests pass.

## Files To Touch

- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/command_config.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/live_task_binding.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/devnet_settlement*.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_*.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report*.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_*contract.rs`
- `crates/kamn-e2e-harness/tests/support/mvp_demo_command.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

All source, binding, escrow, persistence, and verifier errors return `Err` with
the failing boundary named. Required devnet mode remains explicit `NO-GO` only
for external settlement availability failures; malformed configured task
evidence is a hard configuration/proof error and cannot degrade to NO-GO.

## Test Plan

RED:

- Require task-binding paths/configuration and exact source verification.
- Require actual escrow ID and task-binding fields in devnet evidence.
- Require live task/escrow/binding agreement across all three-agent artifacts.
- Reject unrelated, altered, partial, private, and synthetic evidence.

GREEN:

- Add the minimal Rust task-binding reader/writer.
- Include binding fields in escrow funding and settlement evidence.
- Replace synthetic three-agent IDs with bound live IDs.

REFACTOR:

- Centralize shared transaction/escrow/binding fields for artifact builders.
- Keep all files/functions within repository limits and remove duplication.

INTEGRATION:

- Reuse funded ignored devnet wallets and create fresh A/B/C task evidence.
- Run the bound devnet-required demo once and confirm the Solana transaction.
- Run canonical and Pi verification against the immutable report.
