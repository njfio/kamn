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

## DIDComm Envelope Compatibility Replay Contract Lane (Issue #892)
Compatibility vectors must replay through a deterministic fixture matrix and fail closed on schema/signature/key-reference drift.

- Replay matrix runner:
  - `python3 scripts/message/run_didcomm_envelope_compatibility_replay.py --fixture fixtures/didcomm_envelope_compatibility/replay_cases.json --output-json /tmp/didcomm-envelope-compatibility-report.json`
- Policy checker:
  - `bash scripts/message/check_didcomm_envelope_compatibility_policy.sh --report-file /tmp/didcomm-envelope-compatibility-report.json`
- Stable shell wrapper:
  - `scripts/message/check_didcomm_envelope_compatibility_policy.sh`
- Shared Python implementation:
  - `scripts/message/didcomm_envelope_compatibility_policy_contract.py`
- PR fast contract lane:
  - `bash scripts/message/run_didcomm_envelope_compatibility_contract_lane.sh`
- Decision key contract:
  - `didcomm_envelope_compatibility_reason_codes:GO:v1`
- Regression policy:
  - schema/signature/recipient-key drift and expected-decision mismatches force `NO-GO` (`Regression: #892`).

## Local Validation
Run from repository root:

```bash
bash scripts/message/test_run_didcomm_envelope_compatibility_replay.sh
bash scripts/message/test_check_didcomm_envelope_compatibility_policy.sh
bash scripts/message/test_run_didcomm_envelope_compatibility_contract_lane.sh
cargo test -p kamn-core --test didcomm_compatibility_profile_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
