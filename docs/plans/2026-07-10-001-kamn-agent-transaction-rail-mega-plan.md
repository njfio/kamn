---
title: KAMN Agent Transaction Rail
type: feat
status: proposed
date: 2026-07-10
origin: docs/brainstorms/2026-06-26-kamn-forward-strategy-requirements.md
supersedes: docs/plans/2026-06-26-001-kamn-mvp-demo-readiness-plan.md
artifact_status: historical
current_status: completed
---

# KAMN Agent Transaction Rail Mega Plan

> Historical decision artifact. The implementation completed on 2026-07-13
> through tracker `#7088`; this plan is not production or mainnet evidence, and
> settlement claims remain limited to Solana devnet.

## Executive Decision

KAMN should stop expanding as a broad messaging/network/governance platform and
finish one narrow product:

> An authenticated agent-to-agent job transaction rail where Agent A hires
> Agent B, task-bound escrow is funded and settled on Solana devnet, and Agent C
> independently verifies the public proof without seeing participant-private
> data.

This plan moves KAMN from a collection of individually credible proof slices to
one coherent, runtime-enforced, evaluator-friendly transaction.

The final MVP is not production escrow, a marketplace, a general messaging
network, a multi-chain bridge, or a production consensus system. It is one
complete transaction whose identity, authorization, state, settlement, views,
and proof can all be traced to runtime receipts.

## Simple Human Explanation

When this plan is complete, a human evaluator should be able to say:

1. Agent A and Agent B each created a cryptographically authenticated KAMN
   identity.
2. Agent A offered Agent B a specific job with agreed terms and a devnet amount.
3. Only Agent B could accept and complete that job.
4. Only Agent A could fund and authorize release of the associated escrow.
5. KAMN moved developer-test value on Solana devnet and persisted the confirmed
   transaction signature.
6. Agent A and Agent B saw richer participant records.
7. Agent C saw only the restricted public proof.
8. All three views agreed on the public transaction facts.
9. A verifier independently recomputed the proof from runtime artifacts.
10. One command reproduced the story from a fresh checkout.

## Why This Plan Exists

KAMN already has real components:

- Key-bound DIDs and signed request authentication.
- Nonce and replay protection.
- Agent registration and profile persistence.
- Messages, channels, tasks, escrow endpoints, websocket events, and audit
  output.
- JSON/SQLite-oriented durable state surfaces.
- CLI, SDK, MCP, Pi extension, and evaluator harness entrypoints.
- Independent Pi Agent A and Agent B task coordination.
- Independent Agent C restricted observation artifacts.
- Real Solana devnet transfer, confirmation, balance, and persisted signature
  evidence.
- A fail-closed proof report and canonical verifier.

The current weakness is composition. The agent task flow, Agent C observation,
and devnet settlement are proven in separate slices. The generated three-agent
transcript describes a unified story, but the same independent agents do not yet
drive the same task-bound escrow and settlement transaction.

The service also authenticates signers without yet enforcing a complete
business authorization model. Task and escrow transitions are not consistently
bound to the creator, assignee, funder, beneficiary, or verifier role.

## Current Truth

### Real Now

- A local KAMN node and service API run.
- Signed requests are verified against key-bound DIDs.
- Nonces are persistent and replay attempts fail.
- Agent registration is service-backed.
- Tasks can be created, queried, accepted, and completed.
- Escrow records can be funded and released.
- Escrow release can execute a real Solana devnet transfer when configured.
- State, websocket events, audit evidence, proof reports, and report verification
  exist.
- Separate Pi processes can coordinate one accepted task.
- A third Pi process can produce and verify a source-bound restricted artifact.

### Not Real Enough

- A scope header is checked against a route, but there is no complete
  server-side grant/role model proving that the signer is entitled to that
  scope.
- Registration is not clearly required before every protected product action.
- Task mutation does not consistently enforce creator/assignee roles.
- Escrow records do not yet carry complete task, actor, amount, terms, and
  release-policy semantics.
- Escrow release authorization is not tied to the escrow owner or task state.
- Crash recovery around an externally successful but not-yet-persisted Solana
  transfer can create ambiguous retry risk.
- Participant-private and restricted-public views are evaluator artifacts, not
  service-generated role projections.
- The three-agent settlement transcript contains generated step labels rather
  than receipts emitted by the same live actors for every step.
- Placeholder orchestration paths still exist outside the canonical MVP lane.
- The repository has too much governance, script, spec, fixture, and marker-test
  surface relative to the product path.

## Product Contract

### Canonical User Story

1. Agent A registers.
2. Agent B registers.
3. Agent C registers as a verifier or obtains an authenticated verifier grant.
4. Agent A creates transaction `T` and task `K` for Agent B.
5. The task includes a canonical terms digest and devnet amount.
6. Agent B accepts task `K`.
7. Agent A funds escrow `E`, which is permanently linked to `T` and `K`.
8. Agent B submits completion evidence and completes task `K`.
9. Agent A authorizes escrow release.
10. KAMN persists a release intent before external submission.
11. KAMN submits and confirms one Solana devnet transfer.
12. KAMN persists the transaction signature and marks escrow settled.
13. Agent A and Agent B query participant-private views.
14. Agent C queries a restricted-public verification view.
15. Agent C independently verifies the shared public commitments.
16. The proof exporter writes a human and machine report from those receipts.

### Required Shared Public Facts

Every view must agree on:

- `transaction_id`
- `task_id`
- `escrow_id`
- transaction state
- task state
- escrow state
- terms digest
- amount in lamports
- settlement network
- settlement transaction signature
- settlement commitment/finality
- runtime receipt digests
- public commitment digest
- timestamps or monotonic sequence markers

### Participant-Private Facts

Agent A and Agent B may see:

- Full task terms or encrypted task payload references.
- Participant identities and roles.
- Completion evidence references.
- Release authorization evidence.
- Participant-specific audit details.
- Any private data explicitly included in the accepted terms.

The MVP does not need to invent sensitive fields merely to demonstrate privacy.
It needs at least one real participant-only field whose absence from Agent C is
enforced and tested.

### Restricted Agent C Facts

Agent C may see only:

- Shared public facts listed above.
- Public receipt and artifact digests.
- Redaction and view-scope markers.
- Enough settlement evidence to independently verify the devnet transaction.

Agent C must not receive:

- Raw task payload or completion payload.
- Participant-private digest fields.
- Signing keys, auth headers, nonces, or credentials.
- Unredacted participant-only audit details.
- Any field not in the restricted projection allowlist.

## Claim Contract

The proof report must continue to use these labels:

- `real`: behavior actually executed by the local runtime.
- `local-only`: real local behavior without external value movement.
- `devnet-backed`: Solana devnet evidence exists and is independently checked.
- `dry-run`: deliberate non-live behavior that cannot satisfy required MVP
  success.
- `placeholder`: illustrative or unimplemented behavior that cannot satisfy
  required MVP success.
- `roadmap`: explicitly deferred behavior.

Additional rules:

- No settlement, escrow release, transfer, or value-movement success may be
  `local-only`.
- No generated narrative step may be presented as an actor receipt.
- No report field may imply server authorization unless the server enforced it.
- A devnet transaction is necessary but not sufficient for escrow success. The
  transaction must also be bound to the authorized task/escrow state machine.
- The canonical verifier must recompute artifact digests and compare runtime
  receipts, not trust copied report fields.

## Target Architecture

Prefer extending existing surfaces. Do not introduce a parallel service.

### Existing Surfaces To Reuse

- `crates/kamn-node/src/service_api_endpoint/`
- `crates/kamn-node/src/service_api_endpoint/message_store/`
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/`
- `crates/kamn-sdk/src/service_client.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-mcp-server/src/dispatch.rs`
- `.pi/extensions/kamn-mvp/`
- `crates/kamn-e2e-harness/src/mvp_demo/`
- `docs/validation/mvp-evaluator-demo.md`
- `Makefile`

### New Concepts Allowed

Only add concepts required to complete the transaction:

- Explicit authorization grants or role assignments.
- Canonical transaction record.
- Task-bound escrow terms.
- Durable settlement intent and reconciliation status.
- Server-generated role projection.
- Runtime actor receipt.

Do not add a new general policy engine, workflow framework, blockchain adapter,
message broker, or database unless the current architecture cannot satisfy a
specific acceptance criterion.

## Proposed Data Model

Exact serialization and migration details belong in issue specs, but the
runtime must represent these facts.

### Agent Registration And Grant

```text
AgentRecord
  did
  public_key_binding
  registration_status
  capabilities
  created_at

AgentGrant
  did
  transaction_id or resource_id
  role: initiator | provider | verifier
  allowed_actions
  status: active | revoked
```

For the MVP, grants may be transaction-scoped rather than a broad RBAC system.
That is smaller and safer.

### Transaction

```text
AgentTransaction
  transaction_id
  initiator_did
  provider_did
  verifier_policy
  task_id
  escrow_id
  terms_digest
  amount_lamports
  state
  created_sequence
  updated_sequence
```

### Task

```text
TaskRecord
  task_id
  transaction_id
  creator_did
  assignee_did
  terms_digest
  private_payload_reference or private_payload_digest
  completion_evidence_digest
  state
```

### Escrow

```text
EscrowRecord
  escrow_id
  transaction_id
  task_id
  funder_did
  beneficiary_did
  amount_lamports
  network
  release_authority_did
  release_policy
  state
  settlement_intent_id
  settlement_tx_signature
  settlement_commitment
```

### Settlement Intent

```text
SettlementIntent
  settlement_intent_id
  escrow_id
  idempotency_key
  expected_transaction_signature or signed_transaction_digest
  state: prepared | submitted | confirmed | reconciled | failed
  submission_attempt_count
  last_error_code
```

### Runtime Receipt

```text
RuntimeReceipt
  receipt_id
  transaction_id
  actor_did
  actor_role
  action
  resource_id
  before_state
  after_state
  sequence
  public_commitment
  artifact_digest
```

## State Machines

### Transaction State

```text
created
  -> accepted
  -> escrow_funded
  -> work_completed
  -> release_authorized
  -> settlement_submitted
  -> settled
```

Failure states:

```text
rejected
cancelled
settlement_failed
settlement_ambiguous
```

No transition may skip required prior states.

### Task State

```text
submitted -> accepted -> completed
```

Rules:

- Only the named provider can accept.
- Only the named provider can complete.
- Completion must include an evidence digest.
- A completed task cannot return to accepted.

### Escrow State

```text
created -> funded -> release_authorized -> submitting -> released
```

Rules:

- Escrow is created for exactly one transaction and task.
- Only the initiator/funder may fund in the MVP.
- Release requires task completion.
- Only the release authority may authorize release.
- External transfer submission is idempotent and recoverable.
- `released` requires confirmed devnet evidence and persisted signature.

## Authorization Matrix

| Action | Agent A | Agent B | Agent C |
| --- | --- | --- | --- |
| Register own identity | Allow | Allow | Allow |
| Create transaction/task | Allow | Deny | Deny |
| Read participant task view | Allow | Allow after assignment | Deny |
| Accept task | Deny | Allow when assigned | Deny |
| Complete task | Deny | Allow when assigned | Deny |
| Fund escrow | Allow | Deny | Deny |
| Authorize release | Allow after completion | Deny | Deny |
| Query participant escrow view | Allow | Allow | Deny |
| Query restricted verification view | Allow | Allow | Allow |
| Verify public proof | Allow | Allow | Allow |
| Read raw private payload | Policy-dependent participant only | Policy-dependent participant only | Deny |

Every deny row requires a negative integration test with a real signed request.

## Security Invariants

1. Authentication and authorization are separate checks.
2. A self-asserted scope is never treated as a grant.
3. Registration is required before protected product actions.
4. Resource ownership/role is checked on every task and escrow operation.
5. Authorization decisions use the signer DID from verified request context,
   never an untrusted body field.
6. Nonce/replay state remains durable across restart.
7. State transitions are idempotent or reject duplicates explicitly.
8. External transfer retries cannot create an untracked second transfer.
9. Agent C responses are built from an allowlist, not by deleting fields from a
   participant response.
10. Secrets never enter proof artifacts, logs, reports, or git.
11. A failed external call cannot silently advance state to released.
12. A confirmed external call followed by a persistence failure enters an
   explicit recoverable/ambiguous state and reconciles by transaction signature.

## Program Structure

This program should be executed as a parent tracker plus small child issues.
Every child follows `AGENTS.md`: issue, spec, RED, GREEN, REFACTOR, integration,
proof, PR, merge.

## First Five Issues In Dependency Order

### Issue 1: Parent Tracker - Complete The Agent Transaction Rail

Purpose:

- Own the program checklist and dependency graph.
- State the product and claim boundaries.
- Link every child issue, spec, PR, demo artifact, and residual risk.

Acceptance:

- No implementation is committed under the tracker itself.
- Each child issue has one concern and testable acceptance criteria.
- The tracker distinguishes runtime proof from report-only proof.

### Issue 2: Enforce Registered Identity And Transaction-Scoped Grants

Depends on: Issue 1.

Purpose:

- Separate signature authentication from business authorization.
- Require registered identities for protected transaction actions.
- Add transaction/resource-scoped roles or grants.

Likely files:

- `crates/kamn-node/src/service_api_endpoint/auth/`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/auth_flow/`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/agent_ops.rs`
- `crates/kamn-sdk/src/service_request_auth.rs`
- service API auth integration tests

RED first:

- Unregistered signed agent can currently call a protected task route.
- Registered Agent C can currently present `tasks:write` or `escrow:write` and
  reach a mutation route.
- Revoked/absent grant is not rejected independently of signature validity.

Acceptance:

- Valid signature without registration returns structured forbidden/unauthorized
  evidence.
- Valid registration without required grant returns structured forbidden
  evidence.
- Scope header remains signed but is treated only as request intent.
- Authorization decision names actor DID, resource, requested action, decision,
  and reason code without leaking secrets.

Risk gate:

- Do not implement a global permission system. Use the minimum transaction or
  resource-scoped grant needed by the MVP.

### Issue 3: Bind Task Lifecycle To Creator And Assigned Provider

Depends on: Issue 2.

Purpose:

- Persist creator, provider, transaction ID, terms digest, and completion
  evidence.
- Enforce the task state machine and actor roles.

Likely files:

- `crates/kamn-node/src/service_api_endpoint/message_store/models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- task route handlers
- `crates/kamn-sdk/src/service_models.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- MCP and CLI task commands

RED first:

- Unassigned agent can accept a task.
- Initiator can complete a provider task.
- Provider can mutate a different provider's task.
- Task can skip accepted and move directly to completed.
- Completion can omit evidence digest.

Acceptance:

- Task creation takes the authenticated initiator from request context.
- Task names exactly one provider for the canonical MVP path.
- Only that provider can accept and complete.
- Every transition emits a durable runtime receipt.
- Retry of an identical transition is deterministic; conflicting retry fails.

Risk gate:

- Data snapshot/schema changes require explicit approval under `AGENTS.md`.
- Preserve migration and restart compatibility; do not silently discard old
  records.

### Issue 4: Add Task-Bound Escrow Terms And Release Authorization

Depends on: Issues 2 and 3.

Purpose:

- Replace generic escrow state with task-bound business semantics.
- Enforce funder, beneficiary, amount, network, and release authority.

Likely files:

- escrow models and store operations
- escrow route handlers
- settlement configuration models
- SDK, agent library, MCP, and CLI escrow commands
- persistence/restart tests

RED first:

- Escrow can be funded without a valid task.
- Escrow can be funded by the wrong actor.
- Escrow amount can disagree with task transaction terms.
- Escrow can be released before task completion.
- Escrow can be released by Agent B or Agent C.
- Escrow can be rebound to another task.

Acceptance:

- Escrow names transaction, task, funder, beneficiary, amount, network, and
  release authority.
- Funding validates task acceptance and terms agreement.
- Release authorization validates completed task and authenticated actor.
- Every transition emits a durable runtime receipt.
- Local-only release cannot satisfy an asset-movement success claim.

### Issue 5: Make Devnet Settlement Idempotent And Recoverable

Depends on: Issue 4.

Purpose:

- Bind one Solana devnet transfer to one authorized escrow release.
- Prevent duplicate transfer on retry or crash.
- Reconcile ambiguous external outcomes.

Likely files:

- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/`
- `state_routes_release.rs`
- escrow settlement persistence
- MVP devnet settlement runner and verifier

RED first:

- Transfer succeeds but persistence fails; retry would submit again.
- Timeout occurs after submission but before confirmation.
- Duplicate release request arrives concurrently.
- Persisted signature does not match confirmed transaction.
- Balance evidence does not match configured amount.

Acceptance:

- Durable settlement intent exists before submission.
- One idempotency key maps to one escrow release.
- Retry checks known signature/status before resubmission.
- Ambiguous outcomes are explicit and cannot return success.
- `released` requires confirmed signature, expected amount/balances, expected
  recipient, expected network, and persisted evidence.
- A restart integration test reconciles a submitted/confirmed intent.

Risk gate:

- This is the highest-risk issue. Do not merge without targeted security review,
  crash-window tests, and a real funded devnet rehearsal.

## Remaining Child Issues

### Issue 6: Add Server-Generated Participant And Verifier Projections

Depends on: Issues 2 through 5.

Goal:

- Generate views from authenticated server context.
- Enforce participant-private versus restricted-public disclosure.

RED first:

- Agent C can query participant view.
- Participant response and verifier response are identical.
- Verifier response includes private digest/payload fields.
- Public commitment disagrees across views.

Acceptance:

- Separate explicit response types/serializers exist.
- Participant membership is checked before private projection.
- Restricted view uses an allowlist.
- A/B see at least one real field C cannot see.
- Shared public facts and commitments match exactly.

### Issue 7: Drive The Full Transaction Through Independent Pi Agents

Depends on: Issue 6.

Goal:

- Reuse current persistent MCP sessions and coordination tools.
- Make independent Agent A, B, and C drive the same transaction.

Required actor behavior:

- Agent A: register, create transaction/task, fund escrow, authorize release,
  query participant view.
- Agent B: register, accept task, submit completion evidence, complete task,
  query participant view.
- Agent C: register/authenticate as verifier, query restricted view, verify
  public proof.

RED first:

- Existing Pi flow cannot carry transaction/escrow IDs through all steps.
- Actor receipt can be produced without a matching runtime receipt.
- Agent C artifact can be created without server restricted-view evidence.

Acceptance:

- Three distinct Pi process IDs.
- Three distinct key-bound DIDs.
- Independent nonce streams.
- One shared transaction/task/escrow ID set.
- All actor receipts reference runtime receipt digests.
- No filesystem handoff is treated as authorization.

### Issue 8: Replace Generated Three-Agent Narrative With Runtime Receipt Chain

Depends on: Issue 7.

Goal:

- Remove fixed transcript step labels from success evidence.
- Build transcript strictly from runtime receipts.

RED first:

- Missing actor receipt is currently replaceable by a generated step string.
- Reordered, duplicated, or cross-transaction receipts are accepted.
- Settlement evidence from another escrow can be copied into the report.

Acceptance:

- Every transcript step names a runtime receipt artifact and digest.
- Receipt chain order and before/after state are verified.
- Transaction, task, escrow, actor, amount, and settlement signature remain
  consistent across the chain.
- Synthetic step names cannot satisfy required claims.

### Issue 9: Add One Canonical Agent Transaction Demo Command

Depends on: Issue 8.

Goal:

- Produce one evaluator command and one report.

Recommended staged command:

```bash
KAMN_MVP_AGENT_DRIVER=pi \
KAMN_MVP_DEVNET_MODE=required \
KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com \
make demo-agent-transaction
```

After repeated success, promote this path to `make demo-mvp` and retain the old
local proof lane under an explicitly local-only name.

Acceptance:

- Fresh checkout prerequisites are checked before work begins.
- Local KAMN runtime is started and stopped automatically.
- Agent processes are launched with bounded tool allowlists.
- Existing Codex/Pi auth is used without committed secrets.
- Devnet configuration is validated before task creation.
- Any failure produces a human-readable `NO-GO` report and leaves no running
  child processes.
- Success writes one proof directory, `report.json`, and `report.md`.
- Report separates local runtime proof from devnet settlement proof.

### Issue 10: Add Independent Evaluator Verification And Explorer Links

Depends on: Issue 9.

Goal:

- Make a skeptical evaluator able to verify the result without trusting the
  demo runner.

Acceptance:

- Canonical verifier recomputes all artifact digests.
- Verifier checks receipt chain, authorization decisions, role projections,
  settlement signature, recipient, amount, commitment, and balance movement.
- Human report links the Solana explorer devnet transaction.
- Verification can run after demo processes have exited.
- Tampering with any shared fact fails with a specific reason code.

### Issue 11: Restart, Retry, Concurrency, And Failure Matrix

Depends on: Issues 5 through 10.

Required tests:

- Node restart after task creation.
- Node restart after escrow funding.
- Node restart after settlement intent persistence.
- Process crash after transfer submission.
- Duplicate Agent A release request.
- Concurrent release requests.
- Stale nonce/replay after restart.
- Devnet RPC timeout and recovery.
- Agent process exits mid-flow.
- Proof generation fails after successful settlement and is retried.

Acceptance:

- No duplicate asset movement.
- No state rollback from settled to pre-settlement.
- No false `GO` after ambiguous settlement.
- Retried proof generation does not resubmit settlement.

### Issue 12: Reduce Repository Surface Around The MVP

Depends on: canonical path proven green.

Goal:

- Make the product easier to understand and maintain.

Actions requiring explicit approval:

- Archive or delete superseded specs.
- Isolate placeholder orchestration under a clearly non-MVP namespace.
- Remove duplicate docs-marker/source-marker tests where behavior tests exist.
- Consolidate redundant scripts and fixtures.
- Demote unused crates/surfaces from the front-door architecture map.

Acceptance:

- Root README points to one product path.
- Placeholder scenarios cannot appear in MVP success output.
- Test taxonomy distinguishes behavior, integration, live, docs-contract, and
  legacy compatibility tests.
- Build/test time and repository file counts move downward.
- No proven runtime behavior is lost.

## Phased Execution Program

## Phase 0: Baseline And Scope Freeze

What this phase proves:

- Everyone is building the same narrow product.
- Existing claims remain honest while implementation changes.

Actions:

1. Open parent tracker.
2. Record current `main` SHA and latest green commands.
3. Link this plan and the origin brainstorm.
4. Freeze new non-MVP nouns/features.
5. Capture the current local and devnet proof bundles as comparison fixtures,
   without committing secrets or mutable live artifacts.

Acceptance:

- Parent tracker has problem, outcome, non-goals, dependency order, and claim
  boundary.
- First five child issues exist with acceptance criteria.
- No feature implementation begins before its issue/spec.

Checkpoint:

- Tag or record baseline commit in tracker; do not create a release tag unless
  repository policy allows it.

Rollback:

- No runtime changes in this phase.

## Phase 1: Authorization Foundation

Issues:

- Issue 2.

What this phase proves:

- A valid signer is not automatically authorized.
- Registration and role/grant checks happen server-side.

Tests go red first:

- Signed but unregistered mutation.
- Signed wrong-role mutation.
- Self-asserted privileged scope.
- Revoked grant.
- Grant for another transaction/resource.

Completion evidence:

```bash
cargo test -p kamn-node service_api -- --nocapture
cargo test -p kamn-sdk service -- --nocapture
cargo test -p kamn-agent-lib -- --nocapture
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
```

Risk gate:

- Security review must confirm actor DID comes from verified request context.
- Do not merge if a caller-controlled body field can choose authorization
  identity.

Rollback:

- Keep new authorization behavior behind explicit API/schema version until all
  current clients are migrated.

## Phase 2: Task And Transaction State Machine

Issues:

- Issue 3.

What this phase proves:

- One transaction and task have explicit actors, terms, and legal transitions.

Tests go red first:

- Wrong provider acceptance/completion.
- Missing/incorrect terms digest.
- Illegal transition order.
- Cross-transaction resource substitution.
- Duplicate/conflicting transition.

Completion evidence:

```bash
cargo test -p kamn-node task_escrow -- --nocapture
cargo test -p kamn-sdk task -- --nocapture
cargo test -p kamn-mcp-server task -- --nocapture
cargo test -p kamn-cli task -- --nocapture
make check
```

Risk gate:

- Snapshot/schema migration test must load the previous format or fail with an
  explicit migration error.

Rollback:

- Preserve old task records read-only during migration; never silently rewrite
  or discard them.

## Phase 3: Task-Bound Escrow And Devnet Safety

Issues:

- Issues 4 and 5, sequentially.

What this phase proves:

- Escrow belongs to one authorized task.
- One release intent results in at most one devnet transfer.

Tests go red first:

- Wrong actor fund/release.
- Wrong task/amount/recipient.
- Release before completion.
- Crash and retry windows.
- Concurrent release.
- Ambiguous RPC outcome.

Completion evidence:

```bash
cargo test -p kamn-node task_escrow -- --nocapture
cargo test -p kamn-node live_settlement -- --nocapture
cargo test -p kamn-e2e-harness devnet_settlement -- --nocapture
make check
```

Required live evidence:

```bash
KAMN_MVP_DEVNET_MODE=required \
KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com \
KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE=<path-to-payer-keypair.json> \
KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY=<recipient> \
KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS=<bounded-test-amount> \
make demo-mvp
```

Risk gate:

- No live rehearsal until negative authorization and idempotency tests pass.
- Use only bounded devnet amounts.
- A missing or ambiguous result is `NO-GO`.

Rollback:

- Disable live settlement configuration and leave intent in explicit failed or
  ambiguous state. Never fake local completion.

## Phase 4: Runtime Disclosure Views

Issues:

- Issue 6.

What this phase proves:

- Privacy differences are enforced by the server.

Tests go red first:

- Agent C participant-view request.
- Nonparticipant participant-view request.
- Restricted view extra-field injection.
- Shared commitment mismatch.
- Private field count/digest leak.

Completion evidence:

```bash
cargo test -p kamn-node projection -- --nocapture
cargo test -p kamn-sdk projection -- --nocapture
cargo test -p kamn-e2e-harness three_agent -- --nocapture
make check
```

Risk gate:

- Restricted projection must be an allowlist serializer.
- Do not claim encrypted private storage unless that is separately proven.

Rollback:

- Fail closed and return no projection if actor role cannot be resolved.

## Phase 5: Independent Agent Execution

Issues:

- Issue 7.

What this phase proves:

- Independent agents, not one harness process impersonating roles, drive the
  transaction.

Tests go red first:

- Same Pi PID reused.
- Same DID reused.
- Shared nonce domain.
- Actor receipt missing runtime receipt digest.
- Transaction/task/escrow mismatch.

Completion evidence:

- Separate Pi process output for A, B, and C.
- Node logs show distinct DIDs and nonce sequences.
- Runtime state shows one transaction/task/escrow relationship.
- Agent C receives only server restricted projection.

Risk gate:

- Filesystem coordination may carry non-secret IDs, but it cannot authorize
  actions or manufacture server evidence.

Rollback:

- Existing two-agent and artifact-level checks remain available as local-only
  diagnostic lanes, not as final MVP success.

## Phase 6: Receipt-Built Proof And Canonical Command

Issues:

- Issues 8, 9, and 10.

What this phase proves:

- The final report is a projection of runtime receipts, not a story generator.
- One command executes and verifies the complete product.

Tests go red first:

- Missing/reordered/cross-run receipts.
- Copied settlement signature.
- Generated step without runtime artifact.
- Report path substitution.
- Artifact digest tampering.
- Agent C private field leak.

Completion evidence:

```bash
make demo-agent-transaction
cargo run -p kamn-e2e-harness -- \
  verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

Required human output:

- What ran locally.
- What ran on Solana devnet.
- Which agents participated.
- Which fields were private/restricted.
- Solana explorer link.
- Exact `GO` or `NO-GO` reason.
- Explicit non-claims.

Risk gate:

- Do not promote the new command to README canonical status until three
  consecutive fresh-run rehearsals pass.

Rollback:

- Keep report schema versioned. Old verifier supports old reports; new success
  requires new schema and receipt chain.

## Phase 7: Failure, Restart, And Evaluator Hardening

Issues:

- Issue 11.

What this phase proves:

- The MVP is reproducible and survives predictable failures without false
  settlement claims.

Completion evidence:

- Restart/retry/concurrency matrix.
- Three consecutive successful local rehearsals.
- At least one successful funded devnet rehearsal after final code changes.
- At least one deliberate `NO-GO` rehearsal.
- Independent verifier run after all demo processes exit.

Risk gate:

- Any duplicate-transfer possibility is release-blocking.
- Any private-field disclosure is release-blocking.
- Any false `GO` is release-blocking.

## Phase 8: Repository Reduction And MVP Release Candidate

Issues:

- Issue 12.

What this phase proves:

- A maintainer can find and understand the product path.

Actions:

- Update README around the single transaction story.
- Add a concise architecture diagram and claim matrix.
- Move deep historical process material away from the first evaluator path.
- Archive/delete only with explicit approval.
- Remove redundant marker tests only after behavior is covered.
- Clearly label legacy placeholder harnesses.

Completion evidence:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
make test
make pre-push
make demo-agent-transaction
cargo run -p kamn-e2e-harness -- \
  verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

Release candidate criteria:

- All MVP claims map to runtime receipts.
- All asset movement maps to confirmed devnet evidence.
- Authorization matrix is enforced by integration tests.
- Agent C disclosure is server-enforced.
- One command and one report are sufficient for evaluation.
- No known duplicate-transfer or private-disclosure risk.
- Remaining non-MVP work is explicitly listed as roadmap.

## Verification Matrix

| Claim | Required evidence | Negative proof |
| --- | --- | --- |
| Agent identity | key-bound DID, registration record, signed request | unregistered/invalid signer rejected |
| Authorization | transaction grant and decision receipt | wrong-role action rejected |
| Task agreement | A create receipt, B accept receipt, terms digest | wrong provider/terms rejected |
| Completion | B completion receipt and evidence digest | A/C completion rejected |
| Escrow funding | task-bound escrow record and A receipt | wrong task/actor/amount rejected |
| Release authorization | completed task and A authorization receipt | early/wrong-role release rejected |
| Devnet movement | signature, confirmation, recipient, balances | missing/ambiguous evidence is NO-GO |
| Participant privacy | A/B server projection | nonparticipant private request denied |
| Agent C verification | restricted server projection and verifier receipt | private field injection rejected |
| Full transaction | ordered runtime receipt chain | missing/reordered/cross-run receipt rejected |
| Durability | restart and reconciliation evidence | no duplicate transfer after retry |

## Human Evaluator Script

The final evaluator experience should take approximately this form:

1. Read the first README section.
2. Confirm prerequisites.
3. Run one command.
4. Watch concise progress for Agent A, Agent B, Agent C, and Solana devnet.
5. Open `report.md`.
6. Open the Solana devnet explorer link.
7. Run the canonical verifier.
8. Inspect A/B/C view differences.
9. Run one tamper fixture and see verification fail.

The evaluator should not need to understand 16 crates, 400 specs, or the legacy
orchestration framework to determine whether the MVP works.

## Evidence Bundle Layout

Recommended final structure:

```text
.kamn/demo/<run-id>/
  state/
    service-api-state.*
  receipts/
    agent-a-register.json
    agent-b-register.json
    agent-c-register.json
    task-create.json
    task-accept.json
    escrow-fund.json
    task-complete.json
    release-authorize.json
    settlement-submit.json
    settlement-confirm.json
    agent-a-view.json
    agent-b-view.json
    agent-c-view.json
  proof/
    receipt-chain.json
    audit-export.json
    settlement-evidence.json
    report.json
    report.md
```

Every artifact must include schema version, run/transaction identity, integrity
digest, and only the fields appropriate to its role.

## Error Taxonomy

At minimum, support stable reason codes for:

- `AGENT_NOT_REGISTERED`
- `ACTION_NOT_GRANTED`
- `RESOURCE_ROLE_MISMATCH`
- `TASK_STATE_INVALID`
- `TASK_TERMS_MISMATCH`
- `ESCROW_TASK_MISMATCH`
- `ESCROW_AMOUNT_MISMATCH`
- `ESCROW_RELEASE_NOT_AUTHORIZED`
- `SETTLEMENT_INTENT_CONFLICT`
- `SETTLEMENT_OUTCOME_AMBIGUOUS`
- `SETTLEMENT_EVIDENCE_INVALID`
- `PROJECTION_SCOPE_DENIED`
- `PRIVATE_FIELD_DISCLOSURE`
- `RECEIPT_CHAIN_INVALID`
- `PROOF_ARTIFACT_TAMPERED`

Interior code returns typed/structured errors. Entrypoints log once and map to
HTTP/MCP/CLI responses with correlation IDs.

## Idempotency And Recovery Rules

- Registration uses DID uniqueness.
- Transaction/task/escrow creation uses explicit idempotency keys.
- Repeated identical transitions return the existing receipt or deterministic
  success.
- Conflicting reuse of an idempotency key fails.
- Settlement intent is durable before external submission.
- Retry reconciles known signed transaction/signature before creating another.
- Proof generation is separate from settlement execution and may be retried
  without resubmission.
- Cleanup commands never delete a run containing ambiguous settlement state.

## Observability Requirements

Required runtime events:

- identity registered
- authorization allowed/denied
- transaction created
- task accepted
- escrow funded
- task completed
- release authorized
- settlement prepared/submitted/confirmed/reconciled
- participant projection read
- verifier projection read
- proof exported/verified

Logs and metrics may include IDs, states, decisions, reason codes, and
correlation IDs. They must not include keys, signatures used as credentials,
raw private payloads, auth headers, or private completion data.

## Fresh Checkout Requirements

The final command must validate:

- Rust toolchain available.
- Required local shell/runtime tools available.
- `pi` available only when Pi driver mode is selected.
- Pi/Codex authentication available without committed secrets.
- Solana RPC URL is devnet.
- Keypair file exists and is not inside the repository.
- Recipient is valid and differs from payer when required.
- Payer has sufficient bounded devnet balance.
- Required ports are available or dynamically allocated.
- No stale run artifacts are reused as current success.

## Performance And Ergonomics Targets

- One obvious canonical command.
- No manual multi-terminal coordination for the evaluator path.
- Concise progress output; deep logs remain in the run directory.
- Warm local-only run target: under 3 minutes.
- Fresh build target: under 15 minutes on a normal developer machine.
- Verifier target: under 30 seconds after build.
- All child processes terminate on success, failure, or interrupt.
- Re-running the command creates a new run without corrupting prior evidence.

Targets are goals, not reasons to weaken validation.

## Checkpoint And Rollback Strategy

### Per-Issue Checkpoints

- Spec commit.
- RED test commit with observed failure.
- GREEN implementation commit.
- Mandatory refactor commit.
- Integration wiring commit.
- Evidence/docs closeout commit.

### Branch Strategy

- One issue branch per child issue.
- Merge in dependency order.
- Never stack multiple unresolved security/payment issues in one PR.
- Rebase/refresh before each new issue.

### Runtime Compatibility

- Version new state and report schemas.
- Preserve the ability to read prior state or fail with an explicit migration
  requirement.
- Keep old demo lane until the new canonical lane passes three rehearsals.
- Never reinterpret an old local-only artifact as new devnet-backed evidence.

### External Failure Rollback

- Solana RPC unavailable: `NO-GO`, no local settlement success.
- Faucet unavailable: use pre-funded evaluator key or `NO-GO`.
- Pi unavailable/auth blocked: core deterministic runtime tests remain valid,
  but agent-driven MVP is `NO-GO`.
- Persistence failure after external submission: enter ambiguous/reconciliation
  state; do not resubmit blindly.

## Do Not Build Yet

- Mainnet settlement.
- Multi-chain settlement.
- Generalized bridge product.
- Production custody.
- Dispute marketplace or mediator network.
- Token economics.
- Broad governance system.
- New consensus protocol.
- Production multi-node fault tolerance.
- General agent discovery marketplace.
- Web dashboard or polished UI before the CLI/demo path is complete.
- New content lifecycle features.
- New scenario families.
- New SDK languages.
- Broad plugin ecosystem.
- Compliance or certification claims.

## Repository Work To Pause

Until the canonical transaction is complete, pause work whose primary output is:

- Another planning taxonomy.
- Another docs-only proof label.
- Another source-marker contract without runtime behavior.
- Another wrapper/matrix family.
- Another placeholder orchestration step.
- Another crate that does not serve the transaction path.
- Another chain/provider adapter.

Exceptions:

- Security fixes.
- Dependency/supply-chain fixes.
- Build/gate repairs.
- Changes directly required by this plan.

## Success Metrics

### Product Metrics

- One complete agent transaction per demo run.
- Three independent authenticated actor processes.
- One task, one escrow, one devnet settlement, one receipt chain.
- Zero unauthorized successful transitions in the negative matrix.
- Zero duplicate transfers across retry/restart tests.
- Zero private fields in Agent C projection.

### Evaluator Metrics

- One command from fresh checkout.
- One human report.
- One canonical verifier command.
- One explorer link.
- One deliberate tamper test that visibly fails.
- Evaluator can explain the product in one sentence after the run.

### Engineering Metrics

- Formatting and strict clippy green.
- All targeted and integration tests green.
- Full pre-push gate green before final PR.
- Placeholder count in canonical MVP path equals zero.
- Repository docs/spec/script surface decreases after cleanup phase.

## Final Definition Of Done

The program is complete only when all statements below are true:

- [ ] Agent A, B, and C are distinct registered, key-bound identities.
- [ ] Server authorization rejects validly signed wrong-role requests.
- [ ] Task is bound to creator, provider, transaction, and terms digest.
- [ ] Escrow is bound to task, actors, amount, network, and release policy.
- [ ] Only authorized transitions succeed.
- [ ] Task completion precedes release authorization.
- [ ] Settlement intent is durable and idempotent.
- [ ] One confirmed Solana devnet transfer is linked to the escrow.
- [ ] Retry/restart cannot create a second transfer.
- [ ] A/B participant views are server-generated and richer than C's view.
- [ ] C's restricted view is server-generated and allowlisted.
- [ ] All views agree on shared public commitments.
- [ ] Three Pi agents drive the same runtime transaction.
- [ ] Every transcript step references a runtime receipt.
- [ ] Proof report contains no synthetic required step.
- [ ] Canonical verifier independently recomputes evidence.
- [ ] `make demo-agent-transaction` succeeds from a fresh checkout.
- [ ] Devnet failure produces explicit `NO-GO`.
- [ ] Local-only mode never claims settlement success.
- [ ] Human report explains real, local-only, devnet-backed, and non-claimed
      behavior.
- [ ] README presents this path as the product front door.
- [ ] Placeholder/legacy paths are separated from MVP success.
- [ ] Independent security and code review have no unresolved high findings.
- [ ] PR and post-merge checks are green.

## Recommended Execution Mode

Use the main thread with `ce:work` and execute serially by issue dependency.

Why:

- Authorization, task state, escrow state, settlement safety, and projection
  logic touch shared service boundaries.
- Parallel implementation would create conflicting assumptions around models
  and state transitions.
- Security/payment issues benefit from one accountable integration owner.

Use bounded subagents only for independent sidecars:

- Explorer: map exact files/tests before each issue.
- Test engineer: design negative authorization and crash-window tests.
- Security reviewer: review grants, role checks, secret handling, and settlement
  idempotency.
- Verifier: independently inspect proof artifacts and final command output.
- Code reviewer: review the completed diff before push.

Do not allow subagents to recursively orchestrate or modify overlapping files.

## Suggested Continuous Execution Loop

For each child issue:

1. Refresh `main` and confirm no newer product branch should be the base.
2. Open/read issue and acceptance criteria.
3. Commit spec before tests.
4. Add failing behavior and negative tests.
5. Record RED evidence.
6. Implement minimum GREEN behavior.
7. Refactor to repository limits.
8. Wire SDK, MCP, CLI, Pi, and runtime entrypoints only where required.
9. Run targeted tests.
10. Run formatting, strict clippy, and `make check`.
11. Run real integration rehearsal.
12. Update spec with deviations/evidence.
13. Obtain focused review.
14. Push, open PR, wait for CI, merge, and verify post-merge `main`.
15. Update parent tracker before starting the next issue.

## Final Recommendation

Continue KAMN, but judge every new change by one question:

> Does this make the single agent job transaction more authorized, more real,
> more recoverable, more private, or easier to verify?

If the answer is no, defer it until after the MVP transaction rail is complete.

## Sources

- Origin requirements:
  `docs/brainstorms/2026-06-26-kamn-forward-strategy-requirements.md`
- Prior readiness plan:
  `docs/plans/2026-06-26-001-kamn-mvp-demo-readiness-plan.md`
- Current evaluator runbook:
  `docs/validation/mvp-evaluator-demo.md`
- Current Agent C artifact boundary:
  `specs/7084-independent-agent-c-observation.md`
- Current report-level three-agent contract:
  `specs/7045-three-agent-escrow-verification.md`
- Repository process:
  `AGENTS.md`
