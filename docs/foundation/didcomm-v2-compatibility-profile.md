# KAMN to DIDComm v2 Compatibility Profile (Issues #178, #179)

This document defines the first compatibility profile slice between the KAMN canonical message envelope and DIDComm v2-compatible message structures.

## Field-Level Mapping
| KAMN Canonical Field | DIDComm v2 Field | Compatibility Rule |
|---|---|---|
| envelope.id | id | Preserve as-is. |
| envelope.from | from | Preserve as DID string. |
| envelope.to[] | to[] | Preserve recipient DID list order. |
| header.message_type | type | Map Request/Response/Event to profile-specific DIDComm message types. |
| body.message | body | Serialize as JSON body payload. |
| proof.verification_method | from_prior/metadata | Preserve verification reference in compatibility metadata. |

## Crypto and Key Handling Expectations
- Ed25519 verification methods remain authoritative for signature validation.
- X25519 key agreement references must map to recipient key IDs.
- Unsupported algorithm negotiation results in compatibility rejection.
- Missing recipient key reference produces deterministic validation failure.

## Deterministic Compatibility Vectors
- Vector-S1: canonical request envelope maps to DIDComm plaintext message.
- Vector-S2: canonical response envelope maps to DIDComm signed response.
- Vector-F1: missing recipient key reference is rejected.
- Vector-F2: unsupported attachment mapping is rejected.

## Limitations and Constraints
- This profile does not define transport binding; transport remains implementation-specific.
- Cross-profile attachment translation is limited to canonical JSON attachments.
- Unsupported attachment translation decision: reject.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test didcomm_compatibility_profile_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
