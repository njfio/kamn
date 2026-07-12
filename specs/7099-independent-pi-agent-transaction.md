# Drive One Transaction Through Independent Pi Agents

## Objective

Prove that three independent Pi processes use three key-bound KAMN identities
and persistent MCP sessions to drive and verify one task-bound Solana devnet
settlement. Bind every actor receipt to an authenticated runtime response so
filesystem coordination cannot substitute for authorization or proof.

## Inputs/Outputs

Inputs are the local KAMN service endpoint, `kamn-mcp-server` binary, three
external key-file paths, three agent names, bounded coordination paths, funded
Solana devnet configuration, and existing Pi Codex OAuth. Key material and Pi
credentials remain outside artifacts and child environments.

Agent A outputs authenticated receipts for registration, task creation, escrow
funding/release, and its participant projection. Agent B outputs authenticated
receipts for registration, task acceptance, completion evidence/completion,
and its participant projection. Agent C outputs authenticated receipts for
registration, the restricted verifier projection, and public-proof
verification. A combined evidence artifact records process IDs, DIDs, request
sequence bounds, shared transaction facts, runtime response digests, and
coordination-artifact digests.

## Boundaries/Non-goals

- Reuse the project-local Pi extension, persistent MCP line-protocol session,
  service API routes, devnet settlement path, and canonical verifier.
- Coordination files may carry non-secret task, escrow, and artifact IDs. They
  never carry credentials and never authorize a KAMN request.
- Runtime receipt means the exact successful MCP/server response body, reduced
  to an integrity digest plus required public identifiers. A Pi-authored label
  is not a runtime receipt.
- The funded rehearsal is Solana devnet only. Local simulation, dry-run output,
  placeholders, and copied settlement evidence cannot satisfy success.
- Replacing the legacy generated transcript is Issue 8. Adding the canonical
  one-command runner is Issue 9.
- No new agent framework, generalized workflow engine, mainnet, custody,
  disputes, refunds, or multi-chain behavior.

## Actor Contract

Each role runs in a distinct Pi operating-system process. Each process starts
one role-specific persistent `kamn-mcp-server` child with its own key file and
monotonic request IDs beginning at one. The role receipt records the Pi process
ID, registered DID, first and last MCP request IDs, and digests of every runtime
response used for its claim.

The MCP child environment is deny-by-default. Endpoint, agent name, and key file
are passed as explicit process arguments; unrelated `KAMN_*` variables,
including signer secrets and settlement keypair configuration, are never
inherited by role children. Only ordinary process essentials and named
test-fixture controls may cross this boundary.

Agent A registers, creates a task, obtains the bound escrow, waits for Agent B
completion, authorizes the bounded release, and queries its participant view.
Agent B registers, accepts the handed-off task through its own authenticated
session, submits completion evidence, completes the task, and queries its
participant view. Agent C registers with verifier scope, receives only the task
ID through coordination, queries the signed restricted view, and verifies the
shared public commitment and settlement facts.

The extension derives one canonical agreement from the registered Agent A and
Agent B DIDs, the bounded devnet amount, and a run-unique transaction ID. Task
creation binds the provider DID, transaction ID, and terms digest. Acceptance,
completion, funding, and release use operation-specific idempotency keys from
that same agreement; completion also binds a completion-evidence digest. Pi
chooses when to invoke each role-limited tool, but cannot author or override
these settlement payloads.

All three outputs must agree on task ID, escrow ID, amount, network, settlement
signature, settlement commitment, and public commitment. Agent A and B retain
participant receipt fields; Agent C has none.

## Failure Modes

- Reused Pi process ID: `PI_ACTOR_PROCESS_REUSED`.
- Reused or missing DID: `PI_ACTOR_IDENTITY_INVALID`.
- Missing, overlapping, or non-monotonic request sequence:
  `PI_ACTOR_NONCE_STREAM_INVALID`.
- A handled MCP error omitted from an otherwise continuous request stream:
  `PI_ACTOR_NONCE_STREAM_INVALID`; error responses are integrity-digested but
  never counted as successful operation receipts.
- Missing or mismatched runtime response digest: `PI_RUNTIME_RECEIPT_MISMATCH`.
- Malformed, duplicate-key, type-confused, or unknown actor artifact field:
  `PI_RUNTIME_RECEIPT_MISMATCH`.
- Shared task, escrow, amount, network, signature, or commitment mismatch:
  `PI_TRANSACTION_FACT_MISMATCH`.
- Agent C participant field or digest exposure: `PI_VERIFIER_PRIVATE_LEAK`.
- Agent C evidence not bound to a server verifier response:
  `PI_VERIFIER_PROJECTION_MISSING`.
- Coordination artifact used as an authorization receipt:
  `PI_HANDOFF_AUTHORIZATION_FORBIDDEN`.
- Missing or inconsistent canonical agreement payload at create, accept,
  complete, fund, or release: hard MCP/service failure; no actor evidence.
- Pi/MCP process failure, timeout, or malformed response: observable hard
  failure; no partial success artifact.
- Devnet unavailable or settlement ambiguous: canonical `NO-GO`; never `GO`.

An ambiguous release response is reconciled inside the same Pi tool invocation.
The workflow retains the same MCP child, release idempotency key, and contiguous
request stream for a bounded retry. It returns success only after the runtime
reports the escrow released with finalized settlement evidence; exhaustion or
any non-ambiguous error hard-fails without writing actor success evidence.
Participant and verifier projection operations likewise poll boundedly when the
runtime view is valid but settlement fields are not finalized yet. Every
interim projection remains in contiguous provenance, and actor evidence uses
only the first complete finalized projection.
After task acceptance, Agent B waits on the authenticated participant
projection until it proves the task-bound escrow exists. Agent B cannot submit
completion before this runtime-backed funding barrier passes; coordination
files are not used for the barrier.
After escrow funding, Agent A waits on its authenticated task projection until
the task is completed before invoking release. This barrier prevents release
eligibility races without retrying unrelated release failures.

All failures hard-fail at the tool or verifier boundary. Existing artifacts are
written idempotently and conflicting replay is rejected.

## Acceptance Criteria

- [ ] Three positive, distinct Pi process IDs are recorded.
- [ ] Three distinct registered DIDs are bound to three external key files.
- [ ] Three independent monotonic MCP request streams are recorded.
- [ ] A failed settlement attempt followed by same-key reconciliation retains
      a continuous request/digest stream without converting the failure into
      successful operation evidence.
- [ ] One release invocation performs bounded same-child reconciliation after
      `SETTLEMENT_OUTCOME_AMBIGUOUS` and fails closed if confirmation is absent.
- [ ] Final projection operations retain the same MCP child while waiting for
      complete finalized settlement fields and fail closed on exhaustion.
- [ ] Agent B proves escrow funding from its authenticated runtime projection
      before completing the task, preventing acceptance/completion races.
- [ ] Agent A proves completed task state through its authenticated MCP session
      before invoking escrow release.
- [ ] Agent A performs create, release, and participant-view operations.
- [ ] Agent B performs accept, completion, and participant-view operations.
- [ ] Create, accept, complete, fund, and release payloads are derived from one
      canonical agreement and use distinct idempotency keys.
- [ ] Agent C performs verifier registration, restricted-view query, and public
      verification without participant-private fields.
- [ ] One task, escrow, amount, network, settlement signature, and public
      commitment match across all actor evidence.
- [ ] Every claimed actor operation references a matching runtime response
      digest.
- [ ] Coordination artifacts are identifier-only and cannot satisfy runtime
      receipt or authorization checks.
- [ ] Required-mode funded rehearsal produces finalized devnet evidence or an
      explicit `NO-GO` without a success claim.
- [ ] Canonical verification rejects each identity, nonce, receipt, privacy,
      and shared-fact failure mode.

## Files To Touch

- `.pi/extensions/kamn-mvp/mcp-session.ts` and focused role/session modules
- `.pi/extensions/kamn-mvp/live-mcp-tools.ts` and focused transaction tools
- `.pi/extensions/kamn-mvp/live-task-coordination*.ts`
- focused `.pi/extensions/kamn-mvp/*.test.ts` contracts
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness_*.rs`
- focused `crates/kamn-e2e-harness/tests/` verifier contracts
- `docs/validation/mvp-evaluator-demo.md` only for the bounded rehearsal

## Error Semantics

Pi extension interiors throw `Error` without logging. Tool entrypoints return
failure once and do not write success receipts after a failed MCP response.
Rust verification returns stable reason codes with actor and artifact context.
Neither layer includes keys, auth headers, signatures used for authentication,
OAuth state, or raw private participant payloads in errors or artifacts.

## Test Plan

RED:

- Accept one process ID or DID for multiple roles.
- Accept actor receipts containing only Pi-authored action labels.
- Accept missing/non-monotonic MCP request ranges.
- Accept a request range that silently omits a handled MCP error response.
- Accept Agent C evidence constructed from coordination files rather than the
  signed server verifier response.
- Accept copied or cross-transaction escrow/settlement facts.
- Accept participant-private markers in Agent C output.
- Forward unrelated signer, service-auth, settlement, or OAuth configuration to
  a spawned MCP role child.
- Accept malformed JSON types, duplicate keys, or unknown fields in an actor
  artifact.
- Accept a claimed successful actor operation without an exact request ID and
  runtime response digest mapping.

GREEN:

- Extend role configuration and persistent sessions to Agent C.
- Add the minimum role-specific MCP tools for task completion, escrow release,
  and participant/verifier projections.
- Carry explicit payloads through MCP accept, complete, and release dispatch so
  the independent actor path can satisfy the existing live-settlement contract.
- Derive the canonical agreement inside the extension after both participant
  DIDs are known; do not accept Pi-authored settlement payloads.
- Digest every exact handled runtime response for continuous request
  provenance, while returning only successful result bodies to operation
  evidence and callers.
- Map each successful role operation to its exact request ID and response digest;
  handled failures remain only in the contiguous provenance stream.
- Keep ambiguous release reconciliation inside one bounded workflow operation
  so Pi cannot exit between submission and same-key confirmation.
- Decode actor artifacts with strict typed JSON semantics before verification.
- Construct the MCP child environment from an explicit allowlist that excludes
  all secret-bearing KAMN and provider configuration.
- Extend canonical evidence verification with identity, nonce, receipt,
  privacy, and shared-fact checks.

REFACTOR:

- Keep role orchestration, session transport, receipt encoding, coordination,
  and verification separate.
- Reuse one response-digest and shared-fact validator.
- Keep files below 200 lines and functions below 25 lines.
- Remove obsolete local-only receipt fields only when behavior is covered.

INTEGRATION:

- Run TypeScript extension contracts and Rust verifier negative contracts.
- Run three separate Pi processes with bounded tool allowlists and existing
  Codex OAuth against one local KAMN stack.
- Run one bounded required-mode devnet settlement through the actor path.
- Verify the resulting canonical report and Pi evidence after all processes
  exit.
- Run formatting, strict clippy, package suites, and `make check` before PR.

## Rollback

Preserve spec, RED, GREEN, REFACTOR, and INTEGRATION commits independently.
Use a fresh coordination directory and release idempotency key per rehearsal.
Never delete or retry an ambiguous settlement by creating a new transaction;
reconcile the persisted signature first. Roll back Pi tool exposure separately
from server settlement state.
