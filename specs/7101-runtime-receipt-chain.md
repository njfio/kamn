# Replace Generated Narrative With A Runtime Receipt Chain

## Objective

Build the canonical three-agent transcript from the strict Agent A, Agent B,
and Agent C runtime receipt artifacts merged under #7099. A generated label,
handoff identifier, copied settlement field, or harness-authored view body must
not satisfy any successful transaction step.

## Inputs/Outputs

Inputs are the three `kamn.mvp.pi-transaction-actor.v1` artifacts already
accepted by `verify_pi_transaction_actor_paths`, the task-bound settlement
claim, and runtime-owned participant/restricted projection artifacts.

Output is `kamn.mvp.runtime-receipt-chain.v1`, referenced by the canonical
three-agent claim. Each ordered step records actor, action, request ID, outcome,
runtime response digest, before/after state, and the shared public transaction
facts required for that step. The chain has its own recomputed artifact digest.

## Boundaries/Non-goals

- Reuse the strict actor decoder and canonical report verifier.
- Do not add another agent framework, runtime state machine, or settlement path.
- Identifier-only handoffs may identify a task but never become chain steps.
- Failed and ambiguous receipts remain provenance but never satisfy a success
  step.
- Legacy generated transcript v1 may remain readable for old local-only reports,
  but it cannot satisfy the runtime-receipt-chain success claim.
- One-command orchestration is #7102; evaluator explorer UX is #7103.
- Do not commit live artifacts, keys, OAuth state, or private participant data.

## Receipt Chain Contract

The canonical successful order is:

1. Agent A registration.
2. Agent B registration.
3. Agent C registration.
4. Agent A task creation (`none -> submitted`).
5. Agent B task acceptance (`submitted -> accepted`).
6. Agent A escrow funding (`none -> funded`).
7. Agent B task completion (`accepted -> completed`).
8. Agent A escrow release authorization and finalized settlement
   (`funded -> released`).
9. Agent A participant projection.
10. Agent B participant projection.
11. Agent C restricted verifier projection.

Per-actor request IDs remain monotonic within their independent streams; the
cross-actor ordering above is the business-state order, not a claim that nonce
values share one global clock. Every step digest must exist in the matching
actor artifact. Mutation steps require exactly one successful receipt. Repeated
projection reads are allowed, but only the final successful projection receipt
can satisfy the corresponding chain step.

All steps carry one task ID, transaction ID, escrow ID, amount, network,
settlement signature, settlement commitment, and public commitment where those
facts exist. Earlier steps may use explicit `not-yet-created` state markers,
never invented IDs. Agent C carries restricted-public scope and no participant
private receipt digest.

## Failure Modes

- Missing required receipt: `RUNTIME_RECEIPT_CHAIN_STEP_MISSING`.
- Duplicate successful mutation: `RUNTIME_RECEIPT_CHAIN_STEP_DUPLICATED`.
- Invalid business-state order: `RUNTIME_RECEIPT_CHAIN_ORDER_INVALID`.
- Receipt/digest mismatch: `RUNTIME_RECEIPT_CHAIN_DIGEST_MISMATCH`.
- Cross-transaction/task/escrow facts: `RUNTIME_RECEIPT_CHAIN_FACT_MISMATCH`.
- Error receipt used as success: `RUNTIME_RECEIPT_CHAIN_OUTCOME_INVALID`.
- Handoff/generated narrative used as proof: `RUNTIME_RECEIPT_CHAIN_SOURCE_INVALID`.
- Verifier private disclosure: `RUNTIME_RECEIPT_CHAIN_VERIFIER_PRIVATE_LEAK`.
- Artifact digest mismatch: `RUNTIME_RECEIPT_CHAIN_ARTIFACT_MISMATCH`.

All failures hard-fail report generation or verification. There is no fallback
from a configured runtime chain to generated transcript strings.

## Acceptance Criteria

- [ ] Every required chain step references one exact actor runtime receipt and digest.
- [ ] Missing, reordered, duplicated, skipped, or cross-transaction receipts fail closed.
- [ ] Business-state order covers registration, create, accept, fund, complete,
      release/finalization, participant projections, and restricted verification.
- [ ] Before/after state and all shared facts remain consistent across the chain.
- [ ] Failed/ambiguous responses remain provenance but cannot satisfy success.
- [ ] Handoff artifacts and generated step labels cannot satisfy chain steps.
- [ ] Agent C is sourced from the final restricted server projection and exposes
      no participant-private field or digest.
- [ ] Canonical report generation consumes the verified runtime chain.
- [ ] Legacy v1 generated transcripts cannot satisfy the runtime-chain claim.
- [ ] Positive/negative contracts, integration wiring, formatting, strict
      clippy, and `make check` pass.

## Files To Touch

- `crates/kamn-e2e-harness/src/mvp_demo/pi_transaction_actor_*.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_transcript*.rs`
- focused runtime-chain modules under `crates/kamn-e2e-harness/src/mvp_demo/`
- `crates/kamn-e2e-harness/src/mvp_demo/runner*.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_runtime_receipt_chain_contract.rs`
- focused shared fixtures under `crates/kamn-e2e-harness/tests/support/`

## Error Semantics

Interior decoding and validation return stable reason-code-prefixed `String`
errors without logging. Runner/report entrypoints surface the error once and do
not emit a PASS claim or partial chain artifact after failure.

## Test Plan

RED:

- A generated v1 transcript plus valid labels passes without actor receipts.
- Remove one required actor receipt while retaining the generated step string.
- Reorder accept/fund or complete/release business steps.
- Duplicate release success or use an ambiguous release response as success.
- Copy one receipt digest or settlement fact from another transaction.
- Replace Agent C projection receipt with a handoff digest.

GREEN:

- Expose strict decoded actor receipts to a focused chain builder.
- Select exactly-once successful mutation receipts and final projection receipts.
- Build and digest the ordered v1 runtime chain.
- Wire report generation and verification to require the chain when actor
  evidence paths are configured.

REFACTOR:

- Keep actor decoding, receipt selection, order/fact validation, serialization,
  and report wiring separate.
- Keep files below 200 lines and functions below 25 lines.
- Reuse existing digest and strict JSON helpers.

INTEGRATION:

- Build a report from the verified #7099 actor fixture set.
- Reject every negative fixture through the public verifier entrypoint.
- Run focused tests, formatting, strict workspace clippy, and `make check`.

## Rollback

Preserve spec, RED, GREEN, REFACTOR, and INTEGRATION commits independently.
Runtime-chain report wiring can be reverted without changing task, escrow,
settlement, Pi, or MCP runtime behavior.
