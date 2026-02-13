# Kolme Runtime Commit Adapter Contract (Issue #979)

This document captures the adapter-backed runtime commit client that maps
deterministic request payloads into provider calls with explicit typed failure
handling.

## Scope Delivered

- Provider-facing transport/error contracts are canonically owned in
  `kamn-kolme` (`runtime_transport_contracts`) and re-exported through the
  `kamn-core` compatibility surface:
  - `KolmeRuntimeCommitProvider`
  - `KolmeRuntimeCommitProviderOutcome`
  - `KolmeRuntimeCommitProviderReceipt`
  - `KolmeRuntimeCommitProviderError`
  - `KolmeRuntimeCommitProviderTransport`
  - `KolmeRuntimeCommitFinalityTransport`
  - `KolmeRuntimeCommitBlockFallbackTransport`
  - `KolmeRuntimeCommitNotificationsConnection`
  - `KolmeRuntimeCommitNotificationsConnector`
  - `KolmeRuntimeCommitHttpTransport`
- Added adapter-backed runtime commit client in `kamn-core`:
  - `AdapterBackedKolmeRuntimeCommitClient<P>`
- Added typed transport error classification (canonical in `kamn-kolme`,
  compatibility export in `kamn-core`):
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
  - `crates/kamn-kolme/src/http_response_policy.rs` owns HTTP response status/content parsing contracts.
  - `crates/kamn-kolme/src/flat_json_policy.rs` owns flat JSON scalar/object parsing contracts used by broadcast normalization and block-fallback field extraction.
  - `crates/kamn-kolme/src/block_fallback_policy.rs` owns block-fallback response parsing contracts for key/value and flat-JSON payloads.
  - `crates/kamn-kolme/src/transport_request_policy.rs` owns authorization header validation and broadcast submit-path detection contracts.
  - `crates/kamn-kolme/src/provider_outcome_policy.rs` owns live provider outcome parsing and commit-id helper contracts.
  - `crates/kamn-kolme/src/broadcast_payload_policy.rs` owns `/broadcast` payload normalization contracts (direct-signed, signed-envelope, and key/value fallback paths) with deterministic idempotency checks.
  - `crates/kamn-kolme/src/provider_response_policy.rs` owns provider response field parsing contracts (key/value + flat JSON string object formats).
  - `crates/kamn-core/src/kolme_runtime_commit.rs` delegates endpoint normalization through compatibility wrappers.
- Optional auth-aware constructor:
  - `KolmeRuntimeCommitHttpTransport::new_with_authorization(...)`
  - emits `Authorization: <value>` header on submit/finality requests.
- HTTPS/TLS behavior:
  - `https://` requests execute through TLS-backed `openssl s_client` command wiring.
  - `crates/kamn-kolme/src/tls_policy.rs` owns TLS CA-file env parsing and deterministic stderr failure classification contracts.
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
  - canonical pipeline ownership now lives in `crates/kamn-kolme/src/live_provider_pipeline.rs`:
    - `build_runtime_commit_live_provider_config(...)`
    - `build_kolme_fork_broadcast_live_provider_config(...)`
    - `submit_runtime_commit_live_provider_request(...)`
  - `crates/kamn-core/src/kolme_runtime_commit/live_provider.rs` remains a compatibility facade that maps `kamn-kolme` config errors into `KolmeRuntimeCommitError::InvalidRequest`.
  - `KolmeRuntimeCommitLiveProvider::new(...)` preserves legacy submit behavior and request shape.
  - `KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(...)` targets `PUT /broadcast` for `njfio/kolme_fork`.
  - `PUT /broadcast` requests are normalized to JSON using `KolmeApiBroadcastRequest`.
  - txhash-only responses (`{"txhash":"..."}`) map to deterministic commit ids (`kolme-commit:<txhash>`) with default `Pending` finality when backend finality is absent.
  - provider identity for txhash-only responses uses response `provider` when present, otherwise deterministic provider hint from profile construction.
- Fork finality profile:
  - `KolmeRuntimeCommitForkFinalityResolver` composes websocket notifications (`/notifications`) with bounded block fallback scans (`/block/{height}`).
  - websocket handshake/frame parsing contracts are sourced from `kamn-kolme` (`find_http_header_boundary`, `validate_websocket_handshake_response`, `try_take_websocket_frame`) and mapped through `kamn-core` compatibility wrappers.
  - notification variant parsing contracts are sourced from `kamn-kolme` (`parse_notification_event`) and mapped through `kamn-core` compatibility wrappers.
  - finality alias parsing and block-scan policy contracts are sourced from `kamn-kolme` (`parse_receipt_finality`, `parse_commit_receipt_finality`, `validate_lookup_window`, `validate_block_identity`, `render_block_path`, `parse_fork_block_txhash`) so extraction boundaries stay explicit while `kamn-core` adapters are migrated.
  - receipt-finality to commit-finality mapping contracts are sourced from `kamn-kolme` (`commit_finality_from_receipt_finality`) and mapped through `kamn-core` compatibility wrappers.
  - runtime lifecycle/finality projection contracts are sourced from `kamn-kolme` (`lifecycle_state_for_finality`, `lifecycle_state_label`, `commit_finality_label`) and mapped through `kamn-core` compatibility wrappers.
  - provider response field parsing contracts are sourced from `kamn-kolme` (`parse_provider_response_fields`, `parse_provider_key_value_fields`) and mapped through `kamn-core` compatibility wrappers.
  - flat JSON scalar field parsing contracts are sourced from `kamn-kolme` (`parse_flat_json_value_fields`, `required_json_string_field`, `required_positive_u64_json_field`) and mapped through `kamn-core` compatibility wrappers.
  - block-fallback response parsing contracts are sourced from `kamn-kolme` (`parse_block_fallback_response`, `parse_fork_block_fallback_response`) and mapped through `kamn-core` compatibility wrappers.
  - provider outcome and commit-id helper contracts are sourced from `kamn-kolme` (`parse_live_provider_outcome`, `required_provider_response_field`, `parse_commit_id_from_response_fields`, `deterministic_backend_commit_id`, `txhash_from_commit_id`) and mapped through `kamn-core` compatibility wrappers.
  - broadcast payload normalization contracts are sourced from `kamn-kolme` (`normalize_broadcast_payload`) and mapped through `kamn-core` compatibility wrappers.
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
- Deterministic request identity contracts are sourced from `kamn-kolme`
  (`deterministic_runtime_commit_idempotency_key`, `deterministic_runtime_commit_id`)
  and mapped through `kamn-core` compatibility wrappers.
- Deterministic JSON string escaping contracts for signed-envelope serialization are
  sourced from `kamn-kolme` (`escape_json_string`) and mapped through
  `kamn-core` compatibility wrappers.
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

## Decomposition Parity Matrix (Task #2124)

- Deterministic parity artifact:
  - `fixtures/kolme_compatibility/runtime_commit_decomposition_parity_matrix.json`
  - schema: `kamn.kolme.runtime-commit-decomposition-parity-matrix.v1`
- Deterministic parity checker:
  - `python3 scripts/kolme/check_runtime_commit_decomposition_parity_matrix.py check --matrix-file fixtures/kolme_compatibility/runtime_commit_decomposition_parity_matrix.json --output-json /tmp/runtime-commit-decomposition-parity-policy.json`
  - policy schema: `kamn.kolme.runtime-commit-decomposition-parity-policy.v1`
- Covered decomposition scenarios:
  - `submit_http_round_trip` (transport + endpoint/response policies)
  - `submit_fork_broadcast_round_trip` (broadcast payload + provider outcome policies)
  - `finality_notification_resolution` (notification/finality receipt policies)
  - `finality_block_fallback_resolution` (block scan/fallback + lifecycle policies)
- Contract-lane integration:
  - `scripts/kolme/contracts/runtime_commit_contract_lane.py` runs the parity checker before runtime commit cargo tests.
  - `scripts/kolme/test_check_runtime_commit_decomposition_parity_matrix.sh` enforces fail-closed schema/parity drift checks.

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
cargo test -p kamn-kolme --test http_response_policy_contracts
cargo test -p kamn-kolme --test tls_policy_contracts
cargo test -p kamn-kolme --test provider_response_policy_contracts
cargo test -p kamn-kolme --test flat_json_policy_contracts
cargo test -p kamn-kolme --test block_fallback_policy_contracts
cargo test -p kamn-kolme --test provider_outcome_policy_contracts
cargo test -p kamn-kolme --test transport_request_policy_contracts
cargo test -p kamn-kolme --test broadcast_payload_policy_contracts
cargo test -p kamn-kolme --test runtime_lifecycle_policy_contracts
cargo test -p kamn-kolme --test runtime_request_identity_policy_contracts
cargo test -p kamn-kolme --test receipt_to_commit_finality_policy_contracts
cargo test -p kamn-kolme --test json_escape_policy_contracts
cargo test -p kamn-kolme --test commit_finality_parse_policy_contracts
cargo test -p kamn-core --test kolme_runtime_commit_http_transport integration_http_transport_fetch_next_nonce_query_and_parse -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport integration_http_transport_submit_broadcast_request_put_and_parse_txhash -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_http_transport_submit_broadcast_request_rejects_malformed_txhash_response -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport functional_https_transport_submit_with_trusted_ca_succeeds -- --exact
cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_https_transport_maps_certificate_errors_to_unavailable -- --exact
bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode dry-run --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt
python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/kolme-local-runtime-commit-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-runtime-commit-live-policy.json
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --base-url http://127.0.0.1:3000 --provider-hint kolme-fork-local --max-seconds 90 --preflight-max-seconds 10 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt
bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json /tmp/kolme-local-runtime-commit-live-summary.json --policy-output-json /tmp/kolme-local-runtime-commit-live-policy.json
bash scripts/kolme/run_runtime_commit_contract_lane.sh
```

## Local Orchestration Chain (Task #1966)

- Local-only heavy execution remains opt-in and bounded:
  - all non-dry-run orchestration lanes require `KAMN_KOLME_LOCAL_HEAVY=1`.
  - CI fast-gate remains dry-run/contract focused; local run-mode is intentionally opt-in to control cost.
- `kamn-node` live-provider runtime profile (CI-safe configuration validation):
  - `cargo run -p kamn-node -- --role processor --runtime-mode kolme-live --kolme-live-base-url http://127.0.0.1:3000 --kolme-live-provider-hint kolme-fork-local --kolme-live-signing-profile kolme-fork-secp256k1-v1 --output json`
  - expected deterministic report markers:
    - `kolme_live_provider_client_contract=KolmeRuntimeCommitLiveProvider`
    - `kolme_live_signing_profile=kolme-fork-secp256k1-v1`
    - `kolme_live_execution_status=provider-configured`
  - live submit payload composition:
    - runtime request intent is rendered into native direct-signed Kolme message payload (`pubkey`, `nonce`, `created`, `messages`) before `/broadcast` submission.
    - nonce source contract: `GET /get-next-nonce?pubkey=...` through `KolmeRuntimeCommitHttpTransport`.
    - signer profile selector contract: `KAMN_KOLME_LIVE_SIGNER_PROFILE` with supported values `ops-primary` (default) and `ops-secondary`; unsupported values fail closed.
    - private key source contracts: `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX` for `ops-primary`, `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY` for `ops-secondary` (required for selected profile; no fallback private key path).
    - signer adapter contract: `KolmeForkSecp256k1SignerAdapter` owns secp256k1 key decode, recoverable signing, and sign-then-verify compatibility checks against the selected signer key.
    - malformed signature bytes, invalid recovery id, or recovered-key mismatch are rejected fail-closed before runtime submit dispatch.
    - fallback private key marker contract: `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK` must remain unset; policy fails closed with `fallback_signer_secret_present_violation` when present.
    - synthetic fallback signature material (`signature=<idempotency_key>`) is rejected by runtime tests/policies.
  - in-memory fallback markers and non-fork signing profiles are rejected fail-closed by typed node config errors.
- Runtime live lane (submit + optional finality):
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --base-url http://127.0.0.1:3000 --provider-hint kolme-fork-local --max-seconds 90 --preflight-max-seconds 10 --finality-command "printf 'finality=final\n'" --finality-max-seconds 15 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt --finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt`
  - `bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json /tmp/kolme-local-runtime-commit-live-summary.json --policy-output-json /tmp/kolme-local-runtime-commit-live-policy.json`
  - evidence markers `submit_evidence_marker_present` and `finality_evidence_marker_present` must both pass for GO decisions in run mode.
  - request/finality linkage markers `request_payload_evidence_marker_present`, `request_payload_evidence_artifact_path`, `submit_evidence_artifact_path`, `finality_evidence_artifact_path`, `request_finality_evidence_contract_version`, and `request_finality_evidence_linked` must pass for GO decisions in run mode.
  - linkage drift fails closed with deterministic reason codes: `request_payload_evidence_marker_missing`, `finality_evidence_artifact_path_missing`, `request_finality_evidence_linkage_missing`.
  - native payload markers `native_payload_pubkey_marker_present`, `native_payload_nonce_marker_present`, and `native_payload_messages_marker_present` must pass when strict real-node evidence checks are enabled.
  - default live command composition emits `KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1` and policy fails closed when the signing-profile marker is absent.
  - provider hint contract remains live-only for this lane (`kolme-fork-local`); in-memory provider references fail closed with:
    - `provider_hint_in_memory_provider_reference_detected`
    - `live_command_in_memory_provider_reference_detected`
  - summary emits synthetic-command classification markers: `live_command_synthetic`, `finality_command_synthetic`, and `synthetic_evidence_classification_version=v1`.
  - use `--require-non-synthetic-run-evidence` in `check_local_runtime_commit_live_evidence_policy.py` when validating real-node run evidence to fail closed on marker-only command paths.
  - use `--require-native-payload-evidence` in `check_local_runtime_commit_live_evidence_policy.py` when validating real-node run evidence to fail closed on missing native payload markers.
- Local KAMN runtime integration lane:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-commit-finality-command "printf 'finality=final\n'" --runtime-commit-finality-max-seconds 15 --runtime-commit-finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
  - runner default profile is `real-node`; passing `--runtime-profile real-node` explicitly keeps command-surface contracts deterministic.
  - simulation-mode constraint matrix:
    - `runtime-profile=standard` is dry-run only; run mode fails closed before execution.
    - `runtime-profile=real-node` is required for run mode and production/integration evidence lanes.
  - runtime step composes through `run_local_runtime_commit_live_finality_evidence_contract_lane.sh` and captures nested runtime evidence policy artifacts.
  - integration summary emits deterministic runtime evidence profile markers:
    - `runtime_commit_command_profile`
    - `runtime_commit_policy_command_profile`
    - `runtime_commit_command_profile_version`
    - `runtime_signing_profile`
    - `runtime_signer_key_source_contract_version`
    - `runtime_signer_key_source`
  - real-node summary/profile checker contract requires:
    - `runtime_commit_command_profile=real-node-non-synthetic-v1`
    - `runtime_commit_policy_command_profile=real-node-non-synthetic-v1`
    - `runtime_commit_command_profile_version=v1`
    - `runtime_signer_key_source_contract_version=v1`
    - `runtime_signer_key_source=env-local`
  - marker/profile drift is fail-closed in `check_local_kamn_live_runtime_real_node_profile_policy.py`.
  - strict real-node checker additionally fails closed when runtime command surfaces omit fork-aligned signing profile marker:
    - `runtime_commit_real_signing_profile_marker_missing`
  - strict real-node checker also fails closed when simulated/non-secp signing-profile values appear in runtime command surfaces:
    - `runtime_commit_signing_profile_value_disallowed`
    - `runtime_commit_simulated_signing_profile_detected`
  - real-node runner composition rejects in-memory fallback references:
    - `runtime-commit-command must not reference InMemoryKolmeRuntimeCommitClient when runtime-profile=real-node`
  - real-node checker fails closed when in-memory provider references appear in strict command/policy surfaces:
    - `runtime_commit_in_memory_provider_reference_detected`
    - `runtime_commit_policy_check_in_memory_provider_reference_detected`
  - runtime failure taxonomy contract (summary + policy, version `v1`):
    - `runtime_commit_nested_reason_code` captures nested runtime lane `reason_code` (or deterministic `report_missing` / `report_invalid_json` / `reason_code_missing` diagnostics).
    - `runtime_commit_failure_taxonomy` and `runtime_commit_failure_diagnostic_hint` provide deterministic operator-triage surfaces.
    - policy checker `check_local_kamn_live_runtime_integration_policy.py` fails closed on taxonomy drift (`runtime_commit_failure_taxonomy_mismatch:<expected>`).
  - runtime failure taxonomy/remediation map:
    - `transport.preflight.timeout`: verify `--base-url` reachability and preflight endpoint latency.
    - `transport.preflight.failed`: inspect preflight health output and provider-hint wiring.
    - `transport.submit.timeout`: increase `--runtime-commit-max-seconds` or reduce submit command latency.
    - `transport.submit.failed`: inspect runtime submit command stderr and live output artifact.
    - `finality.timeout`: increase `--runtime-commit-finality-max-seconds`; validate notifications/block fallback endpoint responsiveness.
    - `finality.failed`: inspect finality command output and endpoint contract markers.
    - `policy.rejected`: inspect nested runtime policy report `final_decision` and `reason_codes`.
    - `budget.exceeded`: increase `--max-seconds` or reduce local-heavy prerequisite/runtime cost.
    - `runtime.summary.unavailable`: ensure nested runtime summary artifact exists and contains a valid `reason_code`.
    - `runtime.unknown`: inspect nested reason + endpoint/finality artifact logs for unmapped failure signatures.
- Local fork process lifecycle lane:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --integration-runtime-commit-finality-command "printf 'finality=final\n'" --integration-runtime-commit-finality-max-seconds 15 --integration-runtime-commit-finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Real-process wrapper lane with lifecycle run intent:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh --mode run --checkout-path /tmp/kolme_fork --lifecycle-mode run --lifecycle-runtime-commit-finality-command "printf 'finality=final\n'" --lifecycle-runtime-commit-finality-max-seconds 15 --lifecycle-runtime-commit-finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt --output-json /tmp/kolme-local-fork-real-process-summary.json`

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
- local live-node provider smoke preflight and ignored-test dispatch remain fail-closed (`Regression: #1829`).
- local runtime-commit live evidence policy markers for `KolmeRuntimeCommitLiveProvider` path remain fail-closed (`Regression: #2095`).
- `kamn-node` kolme-live runtime profile guardrails reject in-memory provider-hint fallback and invalid signing-profile drift (`Regression: #2175`).
- local runtime-commit submit/finality evidence marker policy and contract lane parity remain fail-closed (`Regression: #2099`).
- local KAMN live runtime integration runtime-step contract composition remains fail-closed for missing runtime policy evidence artifacts (`Regression: #2101`).
- typed nonce/broadcast HTTP helper mapping drift remains fail-closed (`Regression: #1533`).
- provider response field parser extraction parity drift remains fail-closed (`Regression: #1745`).
- flat JSON parser extraction parity drift remains fail-closed (`Regression: #1747`).
- provider outcome and commit-id parser extraction parity drift remains fail-closed (`Regression: #1749`).
- block-fallback parser extraction parity drift remains fail-closed (`Regression: #1751`).
- provider helper reuse extraction parity drift remains fail-closed (`Regression: #1753`).
- transport request helper extraction parity drift remains fail-closed (`Regression: #1755`).
- broadcast payload normalization extraction parity drift remains fail-closed (`Regression: #1757`).
- runtime lifecycle/finality projection extraction parity drift remains fail-closed (`Regression: #1775`).
- runtime request identity extraction parity drift remains fail-closed (`Regression: #1777`).
- receipt-finality mapping extraction parity drift remains fail-closed (`Regression: #1779`).
- JSON escape helper extraction parity drift remains fail-closed (`Regression: #1781`).
- commit-finality parse helper extraction parity drift remains fail-closed (`Regression: #1783`).
- local orchestration chain markers for integration/process/wrapper lifecycle mode and finality pass-through remain fail-closed (`Regression: #1979`).
