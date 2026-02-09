# DID Method and Canonical DID Document Schema (Issues #108, #109)

This document captures the first implementation slice for KAMN DID method handling and canonical DID document construction.
For DID Core 1.1 conformance mapping of `kamn:did`, see `docs/foundation/did-core-conformance-kamn-method.md`.

## Scope Delivered
- Added `crates/kamn-core/src/did.rs` with:
  - `AgentDid` parser for `kamn:did:agent:<method-specific-id>`
  - `AgentDidMetadata` model
  - `DidDocument`, `DidVerificationMethod`, and `DidService` models
  - `canonical_did_document(...)` builder
  - typed parsing/build errors (`AgentDidError`, `DidDocumentError`)
- Added integration tests in `crates/kamn-core/tests/did_method.rs`.

## DID Validation Rules
- DID must start with `kamn:did:agent:`.
- Method-specific ID must be non-empty.
- Method-specific ID must use lowercase alphanumeric, `_`, or `-`.

## Canonical DID Document Rules
- Contexts:
  - `https://www.w3.org/ns/did/v1.1`
  - `https://kamn.network/context/v1`
- Controller equals DID ID.
- Default verification key id: `<did>#keys-1`.
- Default service id: `<did>#messaging`.
- Default service endpoint: `kamn://messaging/<method-specific-id>`.
- Public key and capability entries must be non-empty.

## Federated DID Handshake Evidence Contract (Issue #752)
Cross-network DID trust handshakes must emit deterministic evidence before release approval.

- Evidence bundle generator:
  - `bash scripts/did/generate_federated_did_handshake_evidence_bundle.sh --output-file /tmp/federated-did-handshake.json --handshake-id federated-go-001 --subject-did kamn:did:agent:federated-worker-1 --local-network kolme-mainnet-a --remote-network kolme-mainnet-b --resolver-cache-hit true --resolver-version resolver-v1 --signature-policy PASS --nonce-monotonic true --downgrade-detected false --partition-sequence-monotonic true --required-quorum 2 --received-quorum 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/did/check_federated_did_handshake_policy.sh --bundle-file /tmp/federated-did-handshake.json`
- PR fast contract lane:
  - `bash scripts/did/run_federated_did_handshake_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/did/run_federated_did_handshake_deep_lane.sh --output-json federated-did-handshake-report.json`
- Partition replay matrix runner:
  - `python3 scripts/did/run_federated_did_handshake_matrix.py --fixture fixtures/federated_did_handshake/partition_replay_cases.json --output-json federated-did-handshake-report.json`
- Regression policy:
  - replay/downgrade/tamper attempts force `NO-GO` (`Regression: #734`).

## Local Validation
Run from repository root:

```bash
bash scripts/did/test_generate_federated_did_handshake_evidence_bundle.sh
bash scripts/did/test_run_federated_did_handshake_contract_lane.sh
bash scripts/did/test_run_federated_did_handshake_matrix.sh
bash scripts/did/test_run_federated_did_handshake_deep_lane.sh
cargo test -p kamn-core --test did_method
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
