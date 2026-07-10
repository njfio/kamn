# Independent Agent C Restricted Task Observation

## Objective

Prove a third independent Pi process can verify a restricted-public observation
for the same accepted task coordinated by independent Agent A and Agent B
processes, without receiving participant-private or identity material.

## Inputs/Outputs

Inputs:

- Existing live task handoff and accepted Agent A/B actor receipts.
- `KAMN_MVP_LIVE_TASK_AGENT_C_OBSERVATION_FILE`: restricted observation path.
- A third Pi process invoking only the Agent C observation tool.

Outputs:

- `kamn.mvp.live-task-restricted-observation.v1` with schema, task ID,
  accepted state, public commitment, source artifact digests, zero private-field
  count, restricted-public scope, redaction marker, and artifact digest.
- Agent C result with its Pi process ID and the bounded claim
  `real local-only independent Agent C artifact observation`.

## Boundaries/Non-goals

- Reuse the existing handoff and Agent A/B receipts as the source of truth.
- The observation is an integrity-protected evaluator artifact, not a service
  API response or authorization boundary.
- Agent C must not register a KAMN identity or receive participant key material.
- Do not expose DIDs, names, task payloads, credentials, auth headers,
  signatures, nonces, participant-private fields, or private-view digests.
- Do not claim server-side redaction, escrow, settlement, exchange, asset
  movement, Solana devnet execution, or restart durability.
- Add no dependency and no new runtime architecture.

## Failure Modes

- Missing, blank, secret-like, or unsafe observation path: fail before access.
- Missing, malformed, altered, unknown-field, or conflicting source artifact:
  fail without writing an observation.
- Source task mismatch, non-accepted state, or duplicate Agent A/B process ID:
  fail closed.
- Agent C process ID equals Agent A or Agent B: fail closed.
- Existing observation differs from the expected content: reject overwrite.
- Observation has private fields, private scope, missing redaction, nonzero
  private-field count, wrong public commitment, or digest mismatch: reject.

## Acceptance Criteria

- [x] A builder derives one minimal restricted observation from the verified
  handoff and Agent A/B receipts.
- [x] The observation binds the task, accepted state, source digests, and one
  deterministic public commitment without copying participant-private data.
- [x] Agent C verifies the artifact from a Pi process distinct from Agent A/B.
- [x] Writes are idempotent for identical content and conflicting writes fail.
- [x] Unknown fields, tampering, task mismatch, same-PID reuse, private-field
  leakage, private scope, and non-accepted state fail loudly.
- [x] The Pi extension exposes one Agent C tool with no KAMN identity config.
- [x] Node and Rust contracts pin the artifact fields, tool, configuration,
  three-process runbook, and exact claim limitation.
- [x] A live three-process Pi rehearsal produces matching task evidence.
- [x] Existing targeted tests, formatter, strict clippy, `make check`, canonical
  demo, and canonical report verifier remain green.

## Files To Touch

- `.pi/extensions/kamn-mvp/restricted-task-observation.ts`
- `.pi/extensions/kamn-mvp/restricted-task-observation.test.ts`
- `.pi/extensions/kamn-mvp/live-task-coordination-tools.ts`
- `.pi/extensions/kamn-mvp/live-task-coordination.ts`
- `.pi/extensions/kamn-mvp/live-task-evidence.ts`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_demo_runbook_contract.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

All validation and filesystem failures throw `Error` at the Pi tool boundary.
No partial success or fallback is allowed. The Agent C tool reads source
artifacts and writes only the restricted observation. Repeated identical writes
succeed; conflicting existing evidence fails.

## Test Plan

RED:

- Add Node tests for the artifact field allowlist, commitment binding,
  idempotency, tampering, unknown/private fields, task mismatch, and all three
  process-ID collision cases.
- Extend Rust source and runbook contracts for the Agent C tool and boundary.

GREEN:

- Implement the minimal observation builder/verifier and register one Pi tool.
- Document the third independent Pi command after Agent A/B completion.

REFACTOR:

- Keep source receipt parsing shared and observation policy isolated.
- Verify changed files and functions satisfy repository size limits.

INTEGRATION:

- Run Agent A and Agent B concurrently against one disposable local node.
- Run Agent C in a third Pi process over their accepted-task evidence.
- Inspect the artifact allowlist and process IDs, then run all proof gates.

## Integration Evidence

- 2026-07-10: Agent A, Agent B, and Agent C ran as separate Pi processes with
  PIDs `29023`, `27635`, and `32181`.
- All three artifacts bound to accepted task `task-local-7c1b978c62c40958`.
- The local service returned `201` for task creation and `200` for acceptance
  and both participant task queries.
- Agent C reported `real local-only independent Agent C artifact observation`.
- The restricted observation had the exact 12-field allowlist, zero private
  fields, a redaction marker, source digests, and a public commitment.
- The evidence remains artifact-level and does not claim server-side
  authorization, settlement, asset movement, or devnet execution.
- `cargo fmt --check`, strict workspace clippy, and `make check` passed.
- All 30 Pi extension tests, 23 harness claim contracts, and the runbook
  contract passed.
- Review-driven verification re-reads the handoff and both actor receipts,
  compares task/state and all source digests, and rechecks all three Pi PIDs.
- Post-fix Pi revalidation ran Agent C as PID `73743`; task and source digests
  matched the original A/B evidence, all process IDs differed, scope remained
  `restricted-public`, and private-field count remained zero.
- `make demo-mvp` returned `GO` in optional-devnet local mode, with no value
  movement claim, and the canonical report verifier returned `PASS`.
