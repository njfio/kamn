# Issue #7133: Commit Durable Receipt Chains In Service Projections

## Objective

Make the service derive one deterministic, versioned receipt-chain commitment
from durable authorization, task, escrow, and settlement facts. Participant
projections expose only receipt details authorized for the requesting actor;
the restricted-public projection exposes the common commitment and existing
allowlisted shared facts without participant-private evidence.

## Inputs And Outputs

### Inputs

- The persisted task and its canonical terms, transaction, creator, provider,
  lifecycle state, and completion evidence.
- The single escrow bound to that task and its funding, release authority,
  amount, network, policy, and lifecycle state.
- Durable allowed authorization receipts matched to mutation receipts by
  actor, action, and resource. The earliest matching allowed receipt is
  canonical because ingress and route revalidation may persist more than one
  authorization check for the same mutation.
- Durable task and escrow transition receipts, including their service-derived
  receipt digests.
- The #7125 settlement intent and finalized escrow evidence when settlement is
  present.
- The authenticated requester DID and requested projection scope.

### Outputs

- Both projection scopes use
  `kamn.runtime.task-disclosure-projection.v2`.
- Both scopes include `receipt_chain_commitment`, formatted as
  `sha256:<64 lowercase hex characters>` and derived under the domain
  `kamn.service.receipt-chain.v1`.
- `public_commitment` includes the receipt-chain commitment as one canonical
  shared field, so participant and restricted-public projections agree on the
  same public result.
- Participant-private output includes actor-scoped receipt entries containing
  only `receipt_id`, `receipt_digest`, `action`, `resource_id`, and
  `resulting_state` for mutations performed by the requester.
- Restricted-public output includes no receipt entries, receipt IDs, receipt
  digests, actor DIDs, correlation IDs, idempotency keys, roles, or private
  completion evidence.

## Canonical Receipt Chain

The service selects records for exactly one task, its bound escrow, and its
settlement intent. It validates the applicable lifecycle prefix in this order:

1. `task:create`
2. `task:accept`
3. `escrow:fund`
4. `task:complete`
5. `escrow:release-authorize`
6. confirmed settlement

Each mutation entry commits its phase, receipt ID, service receipt digest,
actor, action, resource binding, prior state, resulting state, and the digest
of its matching allowed authorization receipt. The settlement entry commits
the settlement intent ID, escrow binding, actor, idempotency key, amount,
network, expected signature, signed transaction digest, and confirmed state.
Length-prefixed fields, an entry count, and the fixed domain separate all
values. Optional later phases may be absent only when the current task and
escrow states prove that the lifecycle has not reached them.

The chain is derived from persisted records on every projection read. No
Pi-local value, MCP response, caller-supplied commitment, or cached projection
is an authority source.

## Boundaries And Non-Goals

- Do not change Pi actor-evidence schemas or the independent verifier.
- Do not add grants or broaden Agent C visibility.
- Do not introduce a second settlement path or weaken #7125 settlement checks.
- Do not add dependencies, chains, public CLI flags, or service routes.
- Do not expose correlation IDs, idempotency keys, authorization roles, signed
  transaction JSON, or another participant's private receipt details.
- Do not change SDK or agent-lib projection behavior; they continue preserving
  the server-generated JSON without rewriting it.

## Failure Modes

- Missing or duplicate required lifecycle receipts.
- Reordered lifecycle phases or state transitions that do not form the legal
  prefix for the persisted task and escrow states.
- A receipt bound to another task, escrow, transaction, actor, action, terms,
  amount, network, or release policy.
- A missing, denied, or mismatched authorization receipt for a mutation entry.
- Duplicate receipt IDs or conflicting actor-scoped idempotency keys.
- Settlement evidence that is missing, non-confirmed, cross-escrow,
  cross-actor, or inconsistent with the finalized escrow evidence.
- A participant projection containing another actor's private receipt entry.
- A restricted-public projection containing any non-allowlisted private field.
- Persistence read or decode failure.

## Error Semantics

- Any receipt-chain validation failure returns HTTP 500 through the existing
  task-projection error envelope with code `SERVICE_RECEIPT_CHAIN_INVALID`.
- Missing escrow binding retains `TASK_ESCROW_BINDING_MISSING`.
- Unregistered and non-participant access retain the existing 403 codes.
- Persistence failures retain the existing hard-fail persistence response.
- There is no fallback to the legacy public commitment or an incomplete chain.

## Acceptance Criteria

- [x] The service deterministically derives the versioned chain commitment from
  durable authorization, task, escrow, and conditional settlement facts.
- [x] Canonical entries bind order, actor, action, state, resource, and service
  receipt digest.
- [x] Retry and restart preserve receipt identity and chain commitment without
  duplicating a transition or settlement.
- [x] Duplicate, reordered, cross-resource, cross-actor, and conflicting-key
  chains fail with `SERVICE_RECEIPT_CHAIN_INVALID`.
- [x] Creator and provider projections expose only requester-owned receipt
  identity, digest, action, resource, and resulting-state details.
- [x] Restricted-public output exposes the shared chain commitment and existing
  allowlisted facts without receipt or participant-private detail.
- [x] Participant and restricted-public projections share the same chain and
  public commitments.
- [x] Persisted-state tests cover tampered order, actor, action, state, resource,
  retry, restart, authorization, and settlement cases.
- [x] Formatting, strict Clippy, focused tests, and real service projection
  wiring pass.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/projection_models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/authority_digest.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/task_projection.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/task_projection/commitment.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/task_projection/receipt_chain.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/task_projection/receipt_chain/*.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/queries/task_projection.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/update_routes/state_routes_release/live_settlement.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/update_routes/state_routes_release/live_settlement/release_authority.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_projection_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_projection_receipt_chain_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_projection_settlement_contract_tests.rs`
- Focused test support or extracted contract files when required by size gates.

## Test Plan

### RED

- Add persisted-state integration contracts for deterministic commitments,
  participant privacy, restricted-public allowlisting, retry, and restart.
- Add tamper cases for order, duplicate IDs, actor, action, state, resource,
  authorization matching, idempotency, and settlement binding.
- Confirm the new contracts fail because v2 chain fields and fail-closed chain
  validation do not exist.

### GREEN

- Implement the minimum canonical digesting, chain validation, projection
  models, and error mapping required to pass the RED contracts.

### REFACTOR

- Keep hashing, chain selection/validation, projection assembly, and HTTP error
  translation single-purpose and within repository size limits.
- Run formatting, strict Clippy, focused tests, and touched-size policy.

### INTEGRATION

- Exercise both signed service projection routes against persisted state after
  retry and service restart.
- Prove both scopes return the same chain/public commitments and that the
  restricted-public serialized field set is exact.

## Validation Evidence

- Ubuntu run `29856598101` passed formatting, strict Clippy, touched Rust size,
  service/runtime integration, all receipt-chain contracts, the settlement
  projection contract, and both repeated/restart release contracts. The job
  was later cancelled by its 20-minute cap during unrelated workspace work.
- Ubuntu run `29858368256` passed formatting, strict Clippy, touched Rust size,
  production panic/expect, and service/runtime integration after refactor. Its
  workspace test step failed while linking unrelated `kamn-e2e-harness` tests
  because the runner reported `No space left on device`.
- The post-refactor focused local settlement projection contract passed with
  `1 passed; 0 failed; 689 filtered out`. Cargo was then interrupted while
  loading unrelated integration binaries containing no matching test.
- No shell, Python, workflow, or template surface changed. Rust LOC delta is
  `+804`; shell-to-Rust ratio delta is `0.0` (neutral).
