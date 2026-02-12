# Kolme Runtime Commit Extraction Plan

## Problem Statement
`crates/kamn-core/src/kolme_runtime_commit.rs` still concentrates transport wiring, finality polling, block-fallback reconciliation, lifecycle orchestration, and adapter glue in one module. This plan defines a staged extraction path into `kamn-kolme` while preserving behavior and keeping validation fast and cost-aware.

## Scope Boundary
In scope:
- move transport-facing parsing/validation helpers from `kamn-core` runtime-commit path into `kamn-kolme`,
- move finality and block-fallback integration helpers into `kamn-kolme` with stable contracts,
- keep `kamn-core` focused on runtime orchestration and product-level error typing.

Out of scope:
- changing public runtime-commit API semantics,
- introducing new network protocols,
- expanding CI runtime with expensive live-network jobs.

## Target Module Boundaries
- `kamn-kolme::transport`: endpoint parsing, request/response framing, websocket handshake/frame parsing.
- `kamn-kolme::finality`: provider-response field extraction and finality alias normalization.
- `kamn-kolme::block_fallback`: block-path rendering, fallback payload parsing, lookup-window and block-identity validation.
- `kamn-core::kolme_runtime_commit`: orchestration, lifecycle transitions, adapter policy, and fail-closed domain errors.

## Phase 1 - Transport and endpoint parsing extraction
- expose transport parsing contracts from `kamn-kolme` and replace remaining core-local transport wrappers,
- keep call-site behavior unchanged with extraction-boundary tests,
- gate completion on strict `clippy` and runtime-commit transport regression tests.

## Phase 1 Progress
- #1820: extracted transport IO classification contract to `kamn-kolme` (`classify_transport_io_error`) and removed core-local transport IO classification ownership.

## Phase 2 - Finality and block-fallback extraction
- move finality parsing and block-fallback conversion contracts behind `kamn-kolme` interfaces,
- keep provider mismatch and commit-id mismatch fail-closed behavior in `kamn-core`,
- gate completion on finality and block-fallback regression suites.

## Phase 2 Progress
- #1826: extracted finality response-to-receipt parser contract to `kamn-kolme` (`parse_provider_finality_receipt`) and rewired `kamn-core` finality checker to consume the extracted contract.
- #1836: extracted provider-aware block-fallback parse-selection contract to `kamn-kolme` (`parse_provider_block_fallback_response`) and rewired `kamn-core` block fallback reconciler delegation.
- #1838: extracted notification receipt txhash-correlation helper to `kamn-kolme` (`require_commit_id_matches_expected_txhash`) and rewired `kamn-core` fork finality resolver mismatch checking.
- #1840: extracted latest-block upper-bound selection helper to `kamn-kolme` (`resolve_lookup_upper_bound`) and rewired `kamn-core` fork finality resolver block fallback bound selection.
- #1842: extracted adapter receipt provider/commit-id identity validator to `kamn-kolme` (`validate_provider_receipt_identity`) and rewired `kamn-core` adapter receipt mapping checks.
- #1844: extracted adapter non-final receipt guard to `kamn-kolme` (`require_final_receipt_finality`) and rewired `kamn-core` adapter receipt finality enforcement.
- #1846: extracted live provider outcome finality normalization to `kamn-kolme` (`parse_live_runtime_provider_outcome`) and rewired `kamn-core` live provider parsing delegation.
- #1848: extracted notification event-to-receipt projection to `kamn-kolme` (`notification_event_to_receipt`) and rewired `kamn-core` notification receipt conversion delegation.
- #1850: extracted TLS CA env-result resolver to `kamn-kolme` (`resolve_tls_ca_file_env_result`) and rewired `kamn-core` TLS CA configuration lookup delegation.
- #1852: extracted provider-scoped notification receipt projection to `kamn-kolme` (`notification_event_to_provider_receipt`) and rewired `kamn-core` notification conversion provider normalization + receipt assembly delegation.
- #1854: extracted block-fallback txhash-match receipt projection to `kamn-kolme` (`project_finalized_block_txhash_receipt` / `project_failed_block_txhash_receipt`) and rewired `kamn-core` fallback reconciler receipt projection delegation.
- #1856: extracted fallback txhash request validation + unresolved-reason composition to `kamn-kolme` (`validate_lookup_txhash` / `compose_block_fallback_unresolved_reason`) and rewired `kamn-core` fallback reconciler delegation.
- #1858: extracted terminal receipt-finality gate to `kamn-kolme` (`is_terminal_receipt_finality`) and rewired `kamn-core` finality poller convergence gating delegation.
- #1860: extracted poll-attempt budget validation to `kamn-kolme` (`is_valid_poll_attempt_budget`) and rewired `kamn-core` finality poller budget guard delegation.
- #1862: extracted finality check commit-id request guard to `kamn-kolme` (`is_valid_runtime_commit_id_request`) and rewired `kamn-core` finality checker input validation delegation.
- #1864: extracted finality checker constructor endpoint input guards to `kamn-kolme` (`is_valid_finality_base_url_input` / `is_valid_finality_status_path_input`) and rewired `kamn-core` constructor guard delegation.
- #1866: extracted block-fallback constructor input guards to `kamn-kolme` (`is_valid_block_fallback_base_url_input` / `is_valid_block_fallback_provider_input` / `is_valid_block_fallback_lookup_budget`) and rewired `kamn-core` block-fallback constructor guard delegation.
- #1868: extracted notifications consumer constructor guard contracts to `kamn-kolme` (`is_valid_notifications_provider_input` / `is_valid_notifications_reconnect_budget`) and rewired `kamn-core` notifications consumer guard delegation.
- #1870: extracted websocket connector timeout guard contract to `kamn-kolme` (`is_valid_websocket_timeout_seconds`) and rewired `kamn-core` websocket connector constructor guard delegation.
- #1872: extracted live provider constructor endpoint guard contracts to `kamn-kolme` (`is_valid_live_provider_base_url_input` / `is_valid_live_provider_submit_path_input`) and rewired `kamn-core` live provider constructor guard delegation.
- #1874: extracted HTTP transport timeout guard contract to `kamn-kolme` (`is_valid_http_transport_timeout_seconds`) and rewired `kamn-core` HTTP transport constructor guard delegation.
- #1876: extracted HTTP block-fetch height guard contract to `kamn-kolme` (`is_valid_block_lookup_height`) and rewired `kamn-core` block-fetch transport height validation delegation.
- #1878: extracted adapter expected-provider guard contract to `kamn-kolme` (`is_valid_expected_provider_input`) and rewired `kamn-core` adapter-backed client constructor guard delegation.
- #1880: extracted in-memory provider guard contract to `kamn-kolme` (`is_valid_runtime_provider_input`) and rewired `kamn-core` in-memory client constructor guard delegation.
- #1882: extracted fork provider-hint guard contract to `kamn-kolme` (`is_valid_provider_hint_input`) and rewired `kamn-core` fork broadcast profile provider-hint guard delegation.
- #1884: extracted transport idempotency-key guard contract to `kamn-kolme` (`is_valid_transport_idempotency_key_input`) and rewired `kamn-core` HTTP transport submit-path idempotency guards.
- #1886: extracted transport wire-payload guard contract to `kamn-kolme` (`is_valid_transport_wire_payload_input`) and rewired `kamn-core` HTTP transport submit guard delegation.
- #1888: extracted broadcast submit-path normalization contract to `kamn-kolme` (`normalize_broadcast_submit_path_input`) and rewired `kamn-core` broadcast helper submit-path normalization delegation.
- #1890: extracted receipt-finality update input guards to `kamn-kolme` (`is_valid_receipt_provider_input` / `is_valid_receipt_commit_id_input`) and rewired `kamn-core` runtime pipeline guard delegation.
- #1892: extracted runtime-commit request field guards to `kamn-kolme` (`is_valid_runtime_operation_id_input` / `is_valid_runtime_state_root_input` / `is_valid_runtime_payload_hash_input`) and rewired `kamn-core` request validation delegation.
- #1894: extracted runtime-commit request single-line field guard to `kamn-kolme` (`are_runtime_commit_request_fields_single_line`) and rewired `kamn-core` request validation newline checks delegation.
- #1896: extracted signed-envelope field guards to `kamn-kolme` (`is_valid_signed_envelope_signer_key_id_input` / `is_valid_signed_envelope_message_input` / `is_valid_signed_envelope_signature_input`) and rewired `kamn-core` signed-envelope constructor guard delegation.
- #1898: extracted HTTPS transport stdout non-empty guard to `kamn-kolme` (`is_valid_http_response_bytes_input`) and rewired `kamn-core` TLS response-byte validation delegation.
- #1900: extracted runtime request nonce guard to `kamn-kolme` (`is_valid_runtime_nonce_input`) and rewired `kamn-core` request validation nonce delegation.
- #1902: extracted canonical signed-message match guard to `kamn-kolme` (`is_canonical_runtime_commit_signed_message`) and rewired `kamn-core` signed-envelope canonical payload match validation delegation.
- #1904: extracted signed-envelope field normalization contract to `kamn-kolme` (`normalize_runtime_commit_signed_envelope_fields`) and rewired `kamn-core` signed-envelope construction normalization delegation.
- #1906: extracted runtime-commit canonical wire-payload renderer to `kamn-kolme` (`render_runtime_commit_wire_payload`) and rewired `kamn-core` runtime request payload serialization delegation.
- #1908: extracted runtime request field normalization contract to `kamn-kolme` (`normalize_runtime_commit_request_fields`) and rewired `kamn-core` deterministic request construction normalization delegation.
- #1910: extracted signed-envelope wire payload renderer to `kamn-kolme` (`render_signed_envelope_wire_payload`) and rewired `kamn-core` signed-envelope wire rendering delegation.
- #1912: extracted transport idempotency-key normalization contract to `kamn-kolme` (`normalize_transport_idempotency_key_input`) and rewired `kamn-core` HTTP broadcast submission normalization delegation.
- #1914: extracted provider-hint normalization contract to `kamn-kolme` (`normalize_provider_hint_input`) and rewired `kamn-core` fork broadcast provider construction normalization delegation.
- #1916: extracted notifications provider normalization contract to `kamn-kolme` (`normalize_notifications_provider_input`) and rewired `kamn-core` notifications consumer construction normalization delegation.
- #1918: extracted finality endpoint normalization contract to `kamn-kolme` (`normalize_finality_endpoint_inputs`) and rewired `kamn-core` finality checker constructor normalization delegation.
- #1920: extracted live provider endpoint normalization contract to `kamn-kolme` (`normalize_live_provider_endpoint_inputs`) and rewired `kamn-core` live provider constructor normalization delegation.
- #1922: extracted block-fallback constructor normalization contract to `kamn-kolme` (`normalize_block_fallback_constructor_inputs`) and rewired `kamn-core` block-fallback reconciler constructor normalization delegation.
- #1924: extracted reconnect exhaustion reason composition contract to `kamn-kolme` (`compose_notifications_reconnect_exhausted_reason`) and rewired `kamn-core` notifications consumer reconnect exhaustion errors to delegate text composition.
- #1926: removed local notification parse wrapper glue from `kamn-core` by introducing direct `KamnKolmeNotificationEvent` -> `KolmeRuntimeCommitNotificationEvent` conversion and inlining delegated parse contract mapping in notifications consumer flow.
- #1928: removed local TLS CA env wrapper glue from `kamn-core` by inlining delegated `resolve_tls_ca_file_env_result` contract mapping in HTTPS transport setup and deleting `configured_tls_ca_file` helper ownership.
- #1930: removed local live-provider parse wrapper glue from `kamn-core` by introducing direct `KamnKolmeRuntimeProviderOutcome` -> `KolmeRuntimeCommitProviderOutcome` conversion and inlining delegated parse mapping in live-provider submission flow.
- #1932: removed local lifecycle-record helper glue from `kamn-core` by inlining deterministic lifecycle-record construction in runtime pipeline submit path and deleting `lifecycle_record_from_outcome` helper ownership.
- #1934: removed websocket header-boundary helper glue from `kamn-core` by inlining deterministic handshake header-boundary read loop in connector `connect` flow and deleting `read_http_header_boundary` helper ownership.
- #1936: extracted runtime pipeline lifecycle record/projection ownership into `kolme_runtime_commit/runtime_pipeline.rs` and rewired `kolme_runtime_commit.rs` to re-export pipeline types from the dedicated submodule.
- #1938: extracted in-memory runtime client ownership into `kolme_runtime_commit/in_memory_client.rs` and rewired `kolme_runtime_commit.rs` to re-export `InMemoryKolmeRuntimeCommitClient` from the dedicated submodule.
- #1940: extracted websocket notifications transport ownership into `kolme_runtime_commit/notifications_websocket.rs` and rewired `kolme_runtime_commit.rs` to re-export connector/connection types from the dedicated submodule.
- #1942: extracted notifications consumer ownership into `kolme_runtime_commit/notifications_consumer.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitNotificationsConsumer` from the dedicated submodule.
- #1944: extracted fork finality resolver ownership into `kolme_runtime_commit/fork_finality_resolver.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitForkFinalityResolver` from the dedicated submodule.
- #1946: extracted finality checker ownership into `kolme_runtime_commit/finality_checker.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitFinalityChecker` from the dedicated submodule.
- #1948: extracted block fallback reconciler ownership into `kolme_runtime_commit/block_fallback_reconciler.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitBlockFallbackReconciler` from the dedicated submodule.
- #1950: extracted adapter-backed client ownership into `kolme_runtime_commit/adapter_backed_client.rs` and rewired `kolme_runtime_commit.rs` to re-export `AdapterBackedKolmeRuntimeCommitClient` from the dedicated submodule.
- #1952: extracted live provider ownership into `kolme_runtime_commit/live_provider.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitLiveProvider` from the dedicated submodule.
- #1954: extracted HTTP transport ownership into `kolme_runtime_commit/http_transport.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitHttpTransport` from the dedicated submodule.
- #1956: extracted API codec ownership into `kolme_runtime_commit/api_codec.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeApiNextNonce*`/`KolmeApiBroadcast*` types from the dedicated submodule.
- #1958: extracted runtime request/signed-envelope ownership into `kolme_runtime_commit/request_envelope.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitRequest` and `KolmeRuntimeCommitSignedBroadcastEnvelope` from the dedicated submodule.
- #1960: extracted runtime error ownership into `kolme_runtime_commit/errors.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitTransportErrorKind`, `KolmeRuntimeCommitProviderError`, and `KolmeRuntimeCommitError` from the dedicated submodule.
- #1962: extracted runtime receipt/outcome/notification ownership into `kolme_runtime_commit/outcomes.rs` and rewired `kolme_runtime_commit.rs` to re-export `KolmeRuntimeCommitReceipt`, `KolmeRuntimeCommitOutcome`, `KolmeRuntimeCommitProviderReceipt`, `KolmeRuntimeCommitProviderOutcome`, and `KolmeRuntimeCommitNotificationEvent` from the dedicated submodule.
- #1964: extracted runtime client/provider/transport interface ownership into `kolme_runtime_commit/interfaces.rs` and rewired `kolme_runtime_commit.rs` to re-export runtime trait interfaces from the dedicated submodule.

## Phase 3 - Adapter and lifecycle orchestration extraction
- split remaining adapter/transport bridge glue into dedicated modules (or subcrate) with explicit ownership,
- preserve lifecycle-state and idempotency semantics in `kamn-core`,
- gate completion on end-to-end runtime pipeline tests and extraction-boundary guard tests.

## Risk Register
- coupling drift between `kamn-core` error variants and extracted `kamn-kolme` contracts,
- silent behavior drift during wrapper removal or inline mapping updates,
- extraction churn increasing review burden unless changes remain small and issue-scoped.

## Validation Matrix
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo test -p kamn-core --test kolme_runtime_commit_extraction_boundary`
- `cargo test -p kamn-core --test kolme_runtime_commit_finality`
- `cargo test -p kamn-core --test kolme_runtime_commit_client`
- `cargo test -p kamn-core --test kolme_runtime_commit_client_docs`

Execution policy:
- keep local runtime-commit test runs as the default validation loop,
- keep CI focused on fast, selector-based checks until extraction stabilizes,
- reserve expensive live-network or forked-backend exercises for opt-in local validation lanes.

## Exit Criteria
- extraction-boundary test no longer detects legacy wrappers for extracted responsibilities,
- runtime-commit behavior remains fail-closed across provider mismatch, malformed response, and timeout paths,
- module ownership is explicit enough to split `kolme_runtime_commit` into maintainable submodules without behavioral regressions.

Regression: #1814
