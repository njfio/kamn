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
