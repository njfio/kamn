# Kolme Runtime Commit Adapter Contract (Issue #979)

This document captures the adapter-backed runtime commit client that maps
deterministic request payloads into provider calls with explicit typed failure
handling.

## Scope Delivered

- Added provider-facing adapter interfaces in `kamn-core`:
  - `KolmeRuntimeCommitProvider`
  - `KolmeRuntimeCommitProviderOutcome`
  - `KolmeRuntimeCommitProviderReceipt`
  - `KolmeRuntimeCommitProviderError`
  - `KolmeRuntimeCommitHttpTransport`
- Added adapter-backed runtime commit client:
  - `AdapterBackedKolmeRuntimeCommitClient<P>`
- Added typed transport error classification:
  - `KolmeRuntimeCommitTransportErrorKind::{Timeout, Unavailable, MalformedResponse}`
- Extended runtime commit error contracts:
  - `KolmeRuntimeCommitError::ProviderTransport`
  - `KolmeRuntimeCommitError::ProviderMismatch`
  - `KolmeRuntimeCommitError::NonFinalReceipt`
- Added adapter integration coverage in:
  - `crates/kamn-core/tests/kolme_runtime_commit_client.rs`
  - `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`

## Concrete HTTP Transport

- `KolmeRuntimeCommitHttpTransport` provides deterministic `http://` and `https://` transport paths for:
  - `KolmeRuntimeCommitProviderTransport::submit_runtime_commit(...)`
  - `KolmeRuntimeCommitFinalityTransport::fetch_runtime_commit_finality(...)`
  - `KolmeRuntimeCommitHttpTransport::fetch_next_nonce(...)`
  - `KolmeRuntimeCommitHttpTransport::submit_broadcast_request(...)`
- Endpoint normalization boundary:
  - `crates/kamn-kolme/src/endpoint_policy.rs` owns HTTP/WebSocket endpoint parsing and URL composition contracts.
  - `crates/kamn-core/src/kolme_runtime_commit.rs` delegates endpoint normalization through compatibility wrappers.
- Optional auth-aware constructor:
  - `KolmeRuntimeCommitHttpTransport::new_with_authorization(...)`
  - emits `Authorization: <value>` header on submit/finality requests.
- HTTPS/TLS behavior:
  - `https://` requests execute through TLS-backed `openssl s_client` command wiring.
  - optional custom CA trust file is read from `KAMN_KOLME_TLS_CA_FILE`.
  - deterministic TLS failure mapping:
    - certificate verification failures => `Unavailable("tls certificate verification failed")`
    - handshake/protocol failures => `Unavailable("tls handshake failed")`
- Deterministic runtime behavior:
  - query parameter encoding for `commit_id` in finality polling
  - timeout mapping to `KolmeRuntimeCommitProviderError::Timeout`
  - network and protocol failures mapped fail-closed to provider transport errors
  - deterministic 4xx classification:
    - `401`/`403` => authorization failure (`Unavailable`)
    - `400`/`404`/`409`/`422` => invalid request (`MalformedResponse`)
    - `429` => rate limited (`Unavailable`)
- Live submit profiles:
  - `KolmeRuntimeCommitLiveProvider::new(...)` preserves legacy submit behavior and request shape.
  - `KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(...)` targets `PUT /broadcast` for `njfio/kolme_fork`.
  - `PUT /broadcast` requests are normalized to JSON using `KolmeApiBroadcastRequest`.
  - txhash-only responses (`{"txhash":"..."}`) map to deterministic commit ids (`kolme-commit:<txhash>`) with default `Pending` finality when backend finality is absent.
  - provider identity for txhash-only responses uses response `provider` when present, otherwise deterministic provider hint from profile construction.
- Fork finality profile:
  - `KolmeRuntimeCommitForkFinalityResolver` composes websocket notifications (`/notifications`) with bounded block fallback scans (`/block/{height}`).
  - websocket handshake/frame parsing contracts are sourced from `kamn-kolme` (`find_http_header_boundary`, `validate_websocket_handshake_response`, `try_take_websocket_frame`) and mapped through `kamn-core` compatibility wrappers.
  - notification variant parsing contracts are sourced from `kamn-kolme` (`parse_notification_event`) and mapped through `kamn-core` compatibility wrappers.
  - finality alias parsing and block-scan policy contracts are sourced from `kamn-kolme` (`parse_receipt_finality`, `validate_lookup_window`, `validate_block_identity`, `render_block_path`, `parse_fork_block_txhash`) so extraction boundaries stay explicit while `kamn-core` adapters are migrated.
  - resolver consumes one notification event first:
    - txhash-bearing `NewBlock` / `FailedTransaction` events map directly to receipts.
    - `LatestBlock` (or `NewBlock` payloads that carry height but no txhash) trigger bounded `/block/{height}` fallback reconciliation.
  - transport unavailability/timeout also falls back to bounded block reconciliation.
  - malformed notification payloads and txhash mismatches remain fail-closed and do not trigger fallback.
  - fork profile does not require `/runtime-commit/status` or `/commit/finality` endpoints.
  - block fallback accepts both synthetic parity fixtures and real fork block payloads that include top-level `txhash` plus nested `block`/`logs` JSON.

## Typed Kolme Nonce/Broadcast Codecs

- Added typed Kolme API codec contracts in `kamn-core` (compatibility wrappers) with canonical ownership in `kamn-kolme`:
  - `KolmeApiNextNonceRequest`
  - `KolmeApiNextNonceResponse`
  - `KolmeApiBroadcastRequest`
  - `KolmeApiBroadcastResponse`
- Canonical extraction boundary:
  - `crates/kamn-kolme/src/api_codec.rs` owns deterministic nonce/broadcast codec constructors, query/payload serializers, and JSON parse contracts.
  - `crates/kamn-core/src/kolme_runtime_commit.rs` delegates codec behavior through compatibility wrappers to preserve existing core API surface.
  - direct signed transaction message shape validation (`validate_direct_signed_transaction_message`) is sourced from `kamn-kolme` codec contracts.
- Deterministic nonce request behavior:
  - `KolmeApiNextNonceRequest::query_path(...)` percent-encodes the `pubkey` query
    value and preserves deterministic path composition.
- Deterministic response parsing behavior:
  - `KolmeApiNextNonceResponse::parse_json(...)` requires positive `next_nonce`
    and supports nullable `account_id`.
  - `KolmeApiBroadcastResponse::parse_json(...)` requires non-empty `txhash`.
- Deterministic broadcast payload behavior:
  - `KolmeApiBroadcastRequest::to_json_payload()` emits canonical JSON field order
    (`message`, `signature`, `recovery_id`) and applies JSON string escaping.

## Deterministic Request Normalization Rules

- Adapter submissions call provider transport with:
  - canonical request payload from `KolmeRuntimeCommitRequest::to_wire_payload()`
  - deterministic idempotency key from `KolmeRuntimeCommitRequest::idempotency_key()`
- The adapter preserves validation semantics from
  `KolmeRuntimeCommitRequest::validate()` before provider dispatch.
- Signed translation model:
  - `KolmeRuntimeCommitRequest::translate_to_signed_broadcast_envelope(...)` defines the custody boundary between runtime intent and externally signed payload.
  - `signed_message` must exactly match canonical `to_wire_payload()` output.
  - `KolmeRuntimeCommitSignedBroadcastEnvelope` requires non-empty `signer_key_id`, `message`, and `signature`.
  - fork `/broadcast` normalization accepts signed envelope wire payloads only when `signer_key_id` is present and envelope message idempotency key matches the transport idempotency key.
  - fork `/broadcast` normalization also accepts direct pre-signed transaction JSON payloads (`message`, `signature`, `recovery_id`) when:
    - `message` is a JSON object string.
    - `message` contains required Kolme transaction fields: `pubkey`, `nonce`, `created`, `messages`.
    - optional JSON `idempotency_key` matches the transport idempotency key.
    - malformed/non-JSON direct payload messages fail closed.

## Provider and Finality Policy Rules

- `expected_provider` is mandatory and must be non-empty at client construction.
- Provider response must return matching `receipt.provider`.
- Provider response `receipt.commit_id` must be non-empty.
- Adapter mode requires `receipt.finality == Final`.
- `Pending` or `Failed` receipt finality is rejected as `NonFinalReceipt`.

## Typed Failure Mapping

- Provider timeout:
  - `KolmeRuntimeCommitProviderError::Timeout`
  - mapped to `KolmeRuntimeCommitError::ProviderTransport { kind: Timeout, ... }`
- Provider channel unavailable:
  - `KolmeRuntimeCommitProviderError::Unavailable { reason }`
  - mapped to `KolmeRuntimeCommitError::ProviderTransport { kind: Unavailable, ... }`
- Provider malformed response:
  - `KolmeRuntimeCommitProviderError::MalformedResponse { reason }`
  - mapped to `KolmeRuntimeCommitError::ProviderTransport { kind: MalformedResponse, ... }`
- Provider identity mismatch:
  - mapped to `KolmeRuntimeCommitError::ProviderMismatch`
- Non-final provider receipt:
  - mapped to `KolmeRuntimeCommitError::NonFinalReceipt`

## Validation Commands

Run targeted checks first:

```bash
cargo test -p kamn-core --test kolme_runtime_commit_client
cargo test -p kamn-core --test kolme_runtime_commit_finality
cargo test -p kamn-core --test kolme_runtime_commit_block_fallback
cargo test -p kamn-core --test kolme_runtime_commit_fork_finality_resolver
cargo test -p kamn-kolme --test api_codec_contracts
cargo test -p kamn-kolme --test finality_block_scan_contracts
cargo test -p kamn-kolme --test endpoint_policy_contracts
cargo test -p kamn-kolme --test websocket_policy_contracts
cargo test -p kamn-core --test kolme_runtime_commit_http_transport integration_http_transport_fetch_next_nonce_query_and_parse -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport integration_http_transport_submit_broadcast_request_put_and_parse_txhash -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_http_transport_submit_broadcast_request_rejects_malformed_txhash_response -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport functional_https_transport_submit_with_trusted_ca_succeeds -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_https_transport_maps_certificate_errors_to_unavailable -- --exact
bash scripts/kolme/run_runtime_commit_contract_lane.sh
```

Then run broader regression:

```bash
cargo test -p kamn-core
```

## Regression Markers

- Mutated invalid runtime commit requests remain fail-closed (`Regression: #825`).
- Replay/tamper mismatch policy remains fail-closed (`Regression: #827`).
- Adapter provider mismatch/non-final receipts remain fail-closed (`Regression: #979`).
- HTTPS TLS certificate/handshake drift remains fail-closed (`Regression: #1471`).
- `kolme_fork` submit-profile drift for `PUT /broadcast` and txhash-only response mapping remains fail-closed (`Regression: #1502`).
- `kolme_fork` finality resolution drift for notifications + block fallback remains fail-closed (`Regression: #1503`).
- signed translation envelope and signer custody precondition drift remains fail-closed (`Regression: #1506`).
- direct signed transaction payload normalization drift remains fail-closed (`Regression: #1516`).
- direct signed transaction required-field validation drift remains fail-closed (`Regression: #1519`).
- typed nonce/broadcast HTTP helper mapping drift remains fail-closed (`Regression: #1533`).
