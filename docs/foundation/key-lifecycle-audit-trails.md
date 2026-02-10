# Key Lifecycle Tamper-Evident Audit Trails (Issue #158)

This document captures the first implementation slice for tamper-evident key lifecycle audit trails and verification checks.

## Scope Delivered
- Added deterministic audit record construction in `crates/kamn-core/src/key_lifecycle.rs`:
  - `KeyLifecycleAuditRecord` with `sequence`, `event_kind`, `event_payload`, `previous_hash`, and `record_hash`.
  - `KeyLifecycle::audit_records()` to materialize a hash-chained audit trail from lifecycle events.
  - `KeyLifecycle::verify_audit_trail()` and `KeyLifecycle::verify_audit_records(...)` for integrity validation.
  - `KeyLifecycleAuditError` typed failures for empty trails, sequence gaps, broken chain links, and hash mismatches.
- Extended integration tests in `crates/kamn-core/tests/key_lifecycle.rs` for chain construction and tamper detection.

## Tamper-Evident Rules
- The first record must reference the genesis marker `GENESIS`.
- Sequence IDs must be contiguous and start at `1`.
- Each record hash is computed from:
  - sequence
  - event kind
  - canonical event payload
  - previous hash
- Verification fails when sequence continuity, chain links, or record hashes are inconsistent.

## Limitations (First Slice)
- Hashing currently uses a deterministic non-cryptographic fingerprint for low-dependency bootstrap compatibility.
- A future slice can replace the digest with SHA-256/HMAC signing while preserving record format and verification semantics.

## DID Lifecycle Operator-Binding Audit Evidence Contract (Issue #890)
Lifecycle-sensitive DID mutations must carry deterministic operator-binding authorization evidence and auditable export markers before CI policy gates can return `GO`.

- Evidence bundle generator:
  - `bash scripts/did/generate_lifecycle_operator_binding_evidence_bundle.sh --output-file /tmp/lifecycle-operator-binding.json --did kamn:did:agent:agent-001 --actor-did kamn:did:human:operator-001 --required-operator-did kamn:did:human:operator-001 --mutation-action rotate --mutation-nonce 51 --mutation-reason-code did_lifecycle_mutation_allowed --audit-export-id audit-export-001 --audit-record-count 3 --audit-digest sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/did/check_lifecycle_operator_binding_policy.sh --bundle-file /tmp/lifecycle-operator-binding.json`
- PR fast contract lane:
  - `bash scripts/did/run_lifecycle_operator_binding_contract_lane.sh`
- Stable shell wrappers:
  - `scripts/did/generate_lifecycle_operator_binding_evidence_bundle.sh`
  - `scripts/did/check_lifecycle_operator_binding_policy.sh`
- Shared Python implementation:
  - `scripts/did/lifecycle_operator_binding_contract.py`
- Decision key contract:
  - `did_lifecycle_operator_binding_reason_codes:GO:v1`
- Regression policy:
  - missing required evidence fields and final-decision drift force `NO-GO` (`Regression: #890`).

## Secure-Provider Key Rotation/Revocation Evidence Contract (Issue #988)
Secure-provider signer lifecycle events (rotation/revocation) must emit deterministic custody evidence and fail-closed policy outputs before privileged operations proceed.

- Evidence bundle generator:
  - `bash scripts/signer/generate_secure_provider_key_lifecycle_evidence_bundle.sh --output-file /tmp/secure-provider-key-lifecycle.json --secure-key-reference secure:aws-kms:role-operator/key-ops-rotation-988 --provider aws-kms --key-role operator --lifecycle-action rotate --previous-version 8 --target-version 9 --incident-ticket INC-5988 --revocation-reason-code operator-requested --required-approvals 2 --received-approvals 2 --custody-attestation-hash sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --approval-quorum-hash sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/signer/check_secure_provider_key_lifecycle_policy.sh --bundle-file /tmp/secure-provider-key-lifecycle.json`
- PR fast contract lane:
  - `bash scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh`
- Stable shell wrappers:
  - `scripts/signer/generate_secure_provider_key_lifecycle_evidence_bundle.sh`
  - `scripts/signer/check_secure_provider_key_lifecycle_policy.sh`
  - `scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh`
- Shared Python implementation:
  - `scripts/signer/secure_provider_key_lifecycle_contract.py`
- Decision key contract:
  - `secure_provider_key_lifecycle_reason_codes:GO:v1`
- Regression policy:
  - tampered lifecycle decisions and missing `policy_checks` fail closed (`Regression: #988`).

## Local Validation
Run from repository root:

```bash
bash scripts/did/test_generate_lifecycle_operator_binding_evidence_bundle.sh
bash scripts/did/test_run_lifecycle_operator_binding_contract_lane.sh
bash scripts/signer/test_generate_secure_provider_key_lifecycle_evidence_bundle.sh
bash scripts/signer/test_run_secure_provider_key_lifecycle_contract_lane.sh
bash scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh
cargo test -p kamn-core --test key_lifecycle
cargo test -p kamn-core --test key_lifecycle_audit_trails_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
