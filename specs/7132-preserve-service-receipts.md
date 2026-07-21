# Issue 7132: Preserve Service Receipts Through SDK And MCP

## Objective

Preserve service-issued authority for every canonical registration, task, and
escrow mutation from the durable KAMN service response through `kamn-sdk`,
`kamn-agent-lib`, and `kamn-mcp-server`. MCP must return a versioned structured
authority envelope without minting a replacement receipt or promoting local
transport metadata into service authority.

## Inputs And Outputs

Inputs:

- Authenticated service responses for agent registration, task create/accept/
  complete, and escrow fund/release.
- Durable task and escrow receipt records owned by the service message store.
- The persisted agent profile returned by registration and subsequent queries.
- MCP tool name, authenticated role DID, request ID, and backend result.

Outputs:

- Service mutation models containing the exact `receipt_id` and
  `receipt_digest` issued from canonical durable service facts.
- A deterministic query-backed registration authority commitment when no
  registration transition receipt exists.
- A `kamn.mcp.authority-receipt.v1` structured MCP result that labels service
  authority separately from transport provenance.
- Compatible text content derived from the same validated structured result.

## Boundaries And Non-Goals

- Do not build the multi-receipt chain commitment or projection changes owned
  by #7133.
- Do not migrate Pi evidence or canonical tool allowlists owned by #7134.
- Do not add independent verifier or demo closeout behavior owned by #7135.
- Do not change #7125 settlement receipt authority, create a settlement path,
  add dependencies, or add public CLI flags.
- Query tools remain query results. Only registration may use the explicitly
  approved query-backed profile authority commitment.
- Process IDs, MCP request IDs, nonces, response hashes, and client-computed
  digests are transport provenance and can never populate service authority.

## Authority Schema

Mutation tools return this Rust-owned logical shape:

```json
{
  "schema_version": "kamn.mcp.authority-receipt.v1",
  "authority_kind": "service-receipt",
  "source": "kamn-service",
  "actor_did": "kamn:did:...",
  "tool": "complete_task",
  "resource_id": "task-...",
  "resulting_state": "completed",
  "service_receipt_id": "task-transition-receipt-00000003",
  "service_receipt_digest": "sha256:..."
}
```

Registration uses `authority_kind: service-profile-commitment`, the registered
DID as `resource_id`, and a service-origin `profile_commitment` in place of a
fabricated receipt ID. Transport request IDs remain outside the authority
object in the existing dispatch/JSON-RPC envelope.

The service computes each receipt digest from a domain-separated canonical
encoding of the complete durable receipt record. Field names, order, optional
markers, and digest domain are explicit constants in service-owned Rust code.
SDK, agent-lib, and MCP preserve and validate the resulting strings but never
recompute them as an authority source.

## Failure Modes

- A canonical mutation response omits or empties its service receipt ID.
- A receipt digest is missing, malformed, or not a lowercase `sha256:` value.
- A retry returns a different receipt ID or digest for the same idempotent
  operation.
- Task creation does not append a durable receipt before the response returns.
- Registration authority does not match the persisted/queryable profile.
- SDK parsing drops, rewrites, or substitutes a service authority field.
- MCP receives an authority value whose actor, tool, resource, or state does
  not match the authenticated operation.
- An MCP mutation backend returns legacy resource/state JSON without service
  authority.
- A query response is mislabeled as a mutation receipt.
- Structured and compatible text representations disagree.

## Error Semantics

- Missing canonical authority returns `MCP_AUTHORITY_RECEIPT_MISSING` at the
  MCP boundary and an explicit SDK transport-contract error internally.
- Malformed schema, digest, actor, tool, resource, state, or authority kind
  returns `MCP_AUTHORITY_RECEIPT_INVALID`.
- Durable task receipt creation or persistence failure returns the existing
  structured service persistence error and emits no successful mutation.
- Registration profile mismatch retains the existing hard-fail identity error.
- Interior SDK/agent/MCP helpers return typed errors without logging. The MCP
  protocol boundary renders one structured error and never falls back to a
  compatibility-only authority source.

## Acceptance Criteria

- [x] Task create appends and returns one durable service receipt with a
  service-origin digest.
- [x] Task accept/complete and escrow fund/release return their existing exact
  durable receipt IDs plus service-origin digests.
- [x] Idempotent retry returns the original receipt ID and digest byte-for-byte.
- [x] Registration returns a deterministic service-profile commitment bound to
  the authenticated DID and persisted profile.
- [x] SDK and agent-lib mutation models preserve every authority field without
  substitution; query models remain authority-free.
- [x] MCP returns `kamn.mcp.authority-receipt.v1` for every canonical mutation
  and the approved registration commitment variant.
- [x] MCP structured results separate service authority from request/process
  transport provenance and compatible text is derived from the same value.
- [x] Missing, empty, malformed, cross-tool, cross-actor, cross-resource, or
  legacy mutation results fail closed with the specified error code.
- [x] A real-backend MCP contract proves the returned receipt ID and digest
  equal the durable service response byte-for-byte.
- [x] Formatting, strict Clippy, focused SDK/agent-lib/MCP tests, and the real
  backend MCP path pass.

## Validation Evidence

- Ubuntu fast-gate run `29843773248` passed formatting, strict Clippy, touched
  Rust size policy, live transport, and service HTTP/WebSocket/TLS lanes.
- The same run passed the #7132-focused agent-lib, CLI, MCP authority, real
  backend, node durable-retry, SDK live task/escrow, and inventory contracts.
- The broad job reached its 20-minute ceiling after retrying five unrelated
  targets: rolling governance-history compliance, one supervisor timing test,
  and three pre-existing settlement actor-evidence targets returning
  `SETTLEMENT_EVIDENCE_INVALID`. No #7132 target remained failing.
- Local Rust compilation was not used as authority because rustc repeatedly
  stalled in the macOS loader while linking `kamn-sdk`; formatting and policy
  checks completed locally.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/models.rs`
- `crates/kamn-node/src/service_api_endpoint/escrow_models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/task_models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops/lifecycle.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops/lifecycle/receipt.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops/escrow_lifecycle/receipt.rs`
- Focused node task/escrow persistence and route response tests.
- `crates/kamn-sdk/src/service_models.rs`
- `crates/kamn-sdk/src/service_client_message_task_routes.rs`
- `crates/kamn-sdk/src/service_client_content_routes.rs`
- `crates/kamn-sdk/src/service_client.rs`
- Focused SDK response parsing tests.
- `crates/kamn-agent-lib/src/lib.rs` and focused propagation tests only if
  public re-export/validation wiring is required.
- `crates/kamn-mcp-server/src/dispatch.rs`
- A bounded MCP authority-envelope module if needed to keep files and
  functions within policy.
- `crates/kamn-mcp-server/src/protocol.rs` and `tools.rs` only where required
  for structured results/output schema wiring.
- Focused dispatch, stdio protocol, inventory, and real-backend tests.

## Test Plan

### RED

1. Prove task create, task transition, and escrow response parsers currently
   lose `receipt_id` and have no service digest.
2. Prove task creation currently appends no durable receipt.
3. Require idempotent retry to preserve both receipt ID and digest.
4. Require MCP mutation results to contain and validate the v1 authority
   envelope; reject legacy resource/state-only results.
5. Require registration to expose service-profile commitment authority.
6. Require missing, malformed, mismatched, and query-as-mutation cases to fail
   with the specified error codes.

All RED tests must fail for the missing authority behavior, not for fixture or
compilation mistakes.

### GREEN

Implement the minimum service receipt/digest emission, SDK model parsing, and
MCP structured-envelope validation required to pass RED. Do not add receipt
chain, Pi evidence, or verifier behavior.

### REFACTOR

- Centralize service receipt canonicalization and digest validation.
- Keep each function at or below 25 lines and each file at or below 200 lines.
- Remove duplicated mutation JSON formatting from MCP dispatch.
- Keep query and mutation response types distinct.
- Verify retry idempotency and hard-fail error propagation.

### INTEGRATION

```bash
cargo test -p kamn-node task_escrow_persistence
cargo test -p kamn-sdk service
cargo test -p kamn-agent-lib
cargo test -p kamn-mcp-server --test tool_dispatch_contract
cargo test -p kamn-mcp-server --test stdio_protocol_contract
cargo test -p kamn-mcp-server --test real_backend_integration_contract
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Rollback

- Revert the #7132 commit range if exact service authority cannot remain
  backward compatible at the SDK/MCP boundary.
- Never retain an MCP-only or client-computed receipt as a fallback.
- Preserve existing durable task/escrow records and #7125 settlement evidence
  during rollback.
