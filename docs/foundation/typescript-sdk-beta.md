# TypeScript SDK Beta and Shared Schema Package (Issues #218, #219, #485)

This document captures the first TypeScript SDK implementation slice and the shared protocol schema package used to keep language SDK behavior aligned.

## Scope Delivered
- Added `packages/kamn-schema` with canonical message envelope primitives:
  - constants for canonical type, encryption algorithm, and proof purpose.
  - `createCanonicalMessageEnvelope(...)` helper.
  - `validateCanonicalMessageEnvelope(...)` strict validation rules.
  - `canonicalPayload(...)` deterministic payload serialization.
- Added `packages/kamn-sdk` with dependency-light in-memory SDK parity:
  - `KAMNClient` for register/resolve/send/receive/receiveStream/task/escrow/search/reputation flows.
  - `SDKError` explicit typed errors.
  - send path enforces schema validation through `kamn-schema`.

## Shared Schema Rules (Parity Targets)
The TypeScript schema package mirrors canonical constraints used by core protocol docs:
- envelope type must be `kamn:message:v1`.
- sender and recipients must be valid `kamn:did:agent:*` values.
- expiry must be strictly after creation.
- nonce must be a positive integer.
- message type must be in canonical allowed set.
- encryption algorithm must be `X25519-XChaCha20-Poly1305`.
- recipient keys and body entries must be non-empty.
- proof purpose must be `authentication`.
- proof verification method must be bound to sender DID (`<from>#...`).

## TypeScript SDK Beta Behavior
- IDs are deterministic (`agent_<n>`, `msg_<n>`, `task_<n>`, `escrow_<n>`).
- inbox reads are draining by design.
- async `receiveStream(...)` yields drained inbox messages in deterministic order.
- escrow release is one-way and idempotency-protected.
- search results are deterministic and sorted by DID.
- schema violations in send path are surfaced as `SDKError`.

## Fast and Cost-Effective Validation
This slice avoids dependency-heavy toolchains and uses Node 22 native TypeScript stripping:
- `node --experimental-strip-types --test ...`

PR-fast validation commands:

```bash
npm --prefix packages/kamn-schema test
npm --prefix packages/kamn-sdk test
```

These tests are deterministic, run in milliseconds, and require no package install at this stage.

## Local Validation
Run from repository root:

```bash
npm --prefix packages/kamn-schema test
npm --prefix packages/kamn-sdk test
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test typescript_sdk_beta_docs
cargo test -p kamn-core
```
