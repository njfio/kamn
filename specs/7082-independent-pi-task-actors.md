# Independent Pi Task Actors

## Objective

Prove Agent A and Agent B are driven by separate Pi processes, not one Pi process invoking two KAMN identities, while they coordinate one real local service-backed task through independent persistent MCP children.

## Inputs/Outputs

Inputs:

- Existing Agent A/B live MCP binary, endpoint, name, and key-file configuration.
- `KAMN_MVP_LIVE_TASK_HANDOFF_FILE`: shared non-secret handoff path.
- `KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE`: Agent A observation receipt path.
- `KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE`: Agent B observation receipt path.
- Optional positive integer `KAMN_MVP_LIVE_TASK_COORDINATION_TIMEOUT_MS`.
- Two concurrently running Pi processes with disjoint tool allowlists.

Outputs:

- Handoff artifact `kamn.mvp.live-task-handoff.v1` containing only schema version, task ID, and SHA-256 digest.
- Agent receipts `kamn.mvp.live-task-actor-receipt.v1` containing only schema version, actor, task ID, accepted state, Pi process ID, and SHA-256 digest.
- Verifier result requiring matching task IDs/states, valid digests, expected actor labels, and distinct positive Pi process IDs.
- Human output labelled `real local-only independent Pi actors`.

## Boundaries/Non-goals

- Reuse `LiveTaskWorkflow`, `McpSession`, and existing create/accept/query tools.
- Agent A process may access only Agent A live/coordination tools; Agent B process may access only Agent B live/coordination tools.
- Use the filesystem only as a bounded handoff/evidence boundary, not as a broker, queue, database, or auth mechanism.
- Handoff and receipt paths must reject secret-like markers and resolve explicitly from the evaluator working directory.
- Never put DIDs, title, description, task payload, key paths/contents, headers, signatures, auth nonces, or credentials in coordination artifacts.
- Do not claim Agent C, disclosure asymmetry, service-side redaction, escrow, settlement, asset movement, devnet execution, or restart durability.

## Failure Modes

- Missing/blank path or timeout environment: reject before coordination work.
- Secret-like coordination path: reject before filesystem access.
- Existing handoff for a different task: reject instead of overwrite.
- Missing handoff: poll until timeout or abort, then reject.
- Handoff older than the coordination age limit: reject as stale.
- Malformed schema, unknown field, blank/invalid task ID, or digest mismatch: reject.
- Agent B imports handoff before registration: allowed, but accept still requires B registration.
- Agent A polling sees `submitted`: continue until accepted within timeout.
- Agent A polling sees an unexpected state or different task ID: reject immediately.
- Receipt write before accepted observation: reject.
- Existing receipt differs from the current actor/task/state/PID: reject instead of overwrite.
- Verifier sees missing/malformed receipts, same Pi PID, wrong actor, different task ID, non-accepted state, or digest mismatch: reject.
- Repeated identical handoff/receipt write: succeed idempotently.

## Acceptance Criteria

- [ ] `kamn_live_agent_a_publish_task_handoff` writes the minimal validated handoff after Agent A creates a task.
- [ ] `kamn_live_agent_b_receive_task_handoff` waits for, validates, and imports that task ID into Agent B's workflow.
- [ ] `kamn_live_agent_a_wait_for_task_acceptance` polls through Agent A's existing MCP session and writes Agent A's accepted-state receipt.
- [ ] `kamn_live_agent_b_write_task_receipt` writes Agent B's receipt only after B queries accepted state.
- [ ] `kamn_live_verify_independent_actor_receipts` validates handoff/receipts and proves distinct Pi process IDs.
- [ ] Coordination artifacts contain only their specified allowlisted fields.
- [ ] Writes are idempotent for identical content and fail for conflicting content.
- [ ] Polling honors timeout/abort and rejects stale or malformed artifacts.
- [ ] Node tests cover success, field boundaries, digest tampering, conflict, stale/timeout/abort, receipt mismatch, and same-PID rejection.
- [ ] Rust source/runbook contracts pin tool names, configuration, separate Pi commands, and claim boundary.
- [ ] A real concurrent Pi run proves separate A/B Pi process IDs, distinct KAMN DIDs, independent MCP nonce sequences, and one matching accepted task ID.
- [ ] A separate Pi verifier process validates the two actor receipts without becoming KAMN Agent C.
- [ ] Formatting, strict clippy, targeted contracts, `make check`, canonical demo, and canonical verifier pass.

## Files To Touch

- `.pi/extensions/kamn-mvp/live-task-coordination.ts`
- `.pi/extensions/kamn-mvp/live-task-coordination.test.ts`
- `.pi/extensions/kamn-mvp/live-task-workflow.ts`
- `.pi/extensions/kamn-mvp/live-task-workflow.test.ts`
- `.pi/extensions/kamn-mvp/live-mcp-tools.ts`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_demo_runbook_contract.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

All coordination and verification failures throw `Error` at the Pi tool boundary and report no partial success. Filesystem `ENOENT` is retryable only while waiting for the handoff; malformed or conflicting artifacts fail immediately. Workflow query errors retain existing MCP hard-fail semantics. The verifier reads artifacts without mutating them.

## Test Plan

RED:

- Add pure Node tests for handoff/receipt write-read-verify behavior and every failure mode.
- Extend workflow tests to require external task import, accepted-state polling, and receipt observation access.
- Extend Rust source/runbook contracts with coordination tools/configuration and separate Pi-process commands.

GREEN:

- Implement the minimal coordination artifact module and workflow methods.
- Register five coordination/verifier tools and document the concurrent evaluator sequence.

REFACTOR:

- Keep artifact IO/digest validation separate from MCP workflow state and Pi registration adapters.
- Verify all files/functions remain within repository limits and remove duplicated validation.

INTEGRATION:

- Start one disposable local node and fresh Agent A/B keys.
- Launch Agent B Pi first so it waits for handoff, then Agent A Pi so it creates/publishes/polls.
- Run a separate verifier-only Pi process over the completed artifacts.
- Inspect node logs and structured receipts, then run all local gates and canonical proof commands.
