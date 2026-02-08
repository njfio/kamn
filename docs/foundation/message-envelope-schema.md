# Canonical Message Envelope Schema (Issue #112)

This document defines the first implementation slice for canonical KAMN
message envelope schema and validation.

## Schema Surface
- `EnvelopeMetadata`: canonical envelope identity/routing metadata (`id`, `type_name`, `from`, `to`, `created`, `expires`, threading IDs, nonce).
- `EnvelopeHeader`: message type, priority, content type, and encryption metadata.
- `EnvelopeEncryption`: algorithm and recipient key references.
- `AttachmentRef`: attachment identifier, media type, and URI.
- `EnvelopeProof`: signature proof metadata and proof value.
- `CanonicalMessageEnvelope`: aggregate schema + validation + canonical payload serialization.

## Validation Rules
- Envelope type must be `kamn:message:v1`.
- Sender and recipients must be valid `kamn:did:agent:*` identifiers.
- Envelope expiry must be strictly after creation timestamp.
- Nonce must be positive (`nonce > 0`).
- Header message type must be in the allowed canonical set:
  `Request`, `Response`, `Proposal`, `Acceptance`, `Rejection`, `Delegation`,
  `StatusUpdate`, `PaymentOffer`, `PaymentConfirm`, `Heartbeat`, `Revocation`.
- Encryption algorithm must be `X25519-XChaCha20-Poly1305`.
- Recipient key list and body entries must not be empty.
- Proof purpose must be `authentication`.
- Proof verification method must be bound to sender DID (`<from>#...`).

## Canonical Payload
- `canonical_payload()` emits a deterministic serialization for signing/verification paths.
- Recipient DIDs, recipient key refs, and attachments are sorted before emission.
- Body uses `BTreeMap`, guaranteeing deterministic key ordering.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test message_envelope_schema
cargo test -p kamn-core
```

## Notes
This slice is intentionally dependency-light and fast to validate in CI while
providing strict shape and proof-binding checks for message portability.
