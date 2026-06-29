# Restore Workspace Pre-Merge Contract Tests

## Objective

Restore the `Workspace Pre-Merge Gate (PR)` full workspace test contract for PR
#7022 without weakening tests, CI, proof semantics, source-marker assertions, or
governance-ratio policy.

## Inputs/Outputs

Input:

- PR #7022 Workspace Pre-Merge log at head
  `d6ecc87d3130deebbe19fc2c4ed85e8dac272983`.
- Local red reproductions for:
  - `cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance`
  - `cargo test -p kamn-core --test script_surface_index_docs`
  - `cargo test -p kamn-core --test script_surface_reduction_candidates_docs`
  - `cargo test -p kamn-core --test test_file_size_policy`
  - `cargo test -p kamn-crypto --test direct_message_crypto_failure_paths`
  - `cargo test -p kamn-e2e-harness --test command_result_error_path_policy`
  - `cargo test -p kamn-e2e-harness --test r57_high_gap_evidence_matrix_contract`
- `cargo test -p kamn-e2e-harness --lib`
- `cargo test -p kamn-sdk --test live_transport_task_escrow -- --nocapture`
- `cargo test -p kamn-sdk --test service_api_client -- --nocapture`
- `cargo test -p kamn-node --bin kamn-node`
  - `cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::integration_runtime_kolme_live_renders_provider_contract_markers -- --exact --nocapture`
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::lane_set_bundle_contract_tests::lane_id_bundle_contract_tests::functional_live_postgres_topology_lane_id_bundle_rows_are_canonical -- --exact --nocapture`

Output:

- Passing targeted full-test contract lanes.
- Passing `cargo test --workspace --locked --all-features --no-fail-fast`, or
  separately filed evidence for any unrelated remaining failure.
- Passing `make check`.

## Boundaries/Non-goals

- Do not add MVP demo features.
- Do not weaken or delete failing contract assertions.
- Do not skip workspace pre-merge tests.
- Do not change MVP claim, proof-report, settlement, or devnet semantics.
- Do not create fake or in-memory settlement evidence.

## Failure modes

- The PR head remains over the governance-ratio threshold.
- Script inventory docs or baselines drift from tracked filesystem state.
- Source-marker contracts still reference extracted root files instead of the
  current real files that own the tested behavior.
- Full workspace tests reveal an unrelated new failure after this repair.
- MCP tool-call contract tests depend on platform-specific `/bin/true` or
  `/bin/false` paths and report spawn failures instead of the intended
  subprocess exit or invalid-framing failures.
- Presence-mode websocket bridge tests assert before the live forwarded event
  reaches the response buffer under full-suite load.
- Kolme-live provider-marker tests do not pin the complete env-local signer
  preflight matrix, allowing process-global signer env drift from neighboring
  tests to hide the nonce request marker under parallel binary test execution.
- Daemon phase6 topology projections spawn in-process parallel legs while
  relying on env-backed service-api state and relay-spool paths, allowing one
  leg to append or drain another leg's NDJSON spool under parallel test load.
- The daemon projection fixture's env lock must also serialize with service-api
  tests because both surfaces mutate `KAMN_SERVICE_API_STATE_FILE` and
  `KAMN_SERVICE_API_RELAY_SPOOL_FILE`.
- Agent-lib task/escrow route fixture servers may close a connection without a
  response if full-suite load delays request bytes past a short socket timeout.
- Agent-lib list-message route fixture servers may race the SDK request by
  reserving then dropping a port, sleeping for readiness, and closing before
  the response payload is flushed under full-suite load.
- MCP real-backend integration fixture servers may race real backend dispatch
  requests with reserve/drop loopback ports, fixed sleep readiness, unflushed
  responses, and a short server budget under full workspace load.
- SDK live-transport task/escrow fixture servers may race route calls with
  reserve/drop loopback ports, fixed sleep readiness, incomplete
  content-length reads, and unflushed responses under full workspace load.
- SDK service API client route-family fixture servers may stop reading on a
  transient short socket timeout before a complete signed payload arrives,
  causing route validation to fail before any HTTP response is written.
- Shell wrapper parity tests may run nested cold cargo lanes while the
  workspace pre-merge command is still under load; the wrapper tests must use an
  explicit bounded local pre-merge budget without changing script defaults.
- The async runtime live-validation wrapper measured 434s under full workspace
  pre-merge load, so its wrapper parity test needs a bounded 600s local budget
  while preserving the script's 120s default for direct operator use.
- Mailbox relay API fixtures may issue the first request before the spawned
  endpoint is accepting connections unless the fixture waits for readiness.
- Live bridge websocket delivery can observe submit events but miss the
  forwarded event if request-budget shutdown or a short read window races the
  live bridge forward path under full-suite load.
- Live bridge websocket presence delivery can time out while collecting
  devnet-backed forward evidence if the fixture HTTP client waits for
  connection close instead of a complete response under full-suite load.
- Signer private-key parse zeroization performance must remain bounded without
  using a single-machine 2s wall-clock threshold that fails under full-suite
  test contention.
- MCP real-backend content lifecycle dispatch may return a backend-error
  envelope if the fixture stops reading after a transient short socket timeout
  before the signed POST headers are complete under full workspace load.
- Websocket upgrade flow tests may inherit service-api state, relay, audit, TLS,
  auth, or live-Solana env from neighboring tests unless the harness owns a
  scoped service-api env guard for the lifetime of the endpoint thread.
- Auth-scope endpoint fixtures may build a guarded snapshot and then drop the
  service-api env guard before the spawned endpoint reads startup env, causing
  readiness timeouts under full binary-suite parallel execution.
- Snapshot-journal roundtrip tests may share a temp journal path when parallel
  test threads observe the same coarse system-time tick, causing one payload
  assertion to read another test's first record.
- Fast Gate touched-Rust size policy may reject newly touched service API
  client fixture roots that grow past the file budget even when behavior,
  clippy, and workspace tests are green.
- Fast Gate touched-Rust size policy may reject newly touched oversized core
  model files when CI-only strict-clippy fixes touch them; split model/error
  ownership rather than adding touched-size waivers.
- Splitting a service API client test support fixture into one bounded helper
  module increments the deterministic test-file inventory baseline while
  leaving oversized counts unchanged.
- Workspace Pre-Merge CI may run the full workspace tests from a shallow
  `actions/checkout` clone even though branch-head governance tests require the
  moratorium base-to-head commit range. In that state the checker correctly
  fails closed with `governance_commit_subjects_empty` because the commit window
  is invisible.
- Critical-path coverage may invoke stale `--exact` test selectors after
  module splits, producing `running 0 tests`, missing coverage targets, and
  fail-closed `critical_path_coverage_target_missing` / below-threshold policy
  decisions even when the covered behavior still has current tests.
- Critical-path coverage policy may parse both core and node `llvm-cov`
  reports where one report contains zero-percent entries for files owned by
  the other report; merging must preserve the best observed metrics for the
  same normalized filename instead of overwriting healthy coverage with a
  cross-crate zero row.
- Critical-path coverage thresholds may target a split parent module file that
  no longer owns executable coverage. The group-channel threshold must follow
  `crates/kamn-core/src/group_channel_crypto/engine/lifecycle.rs` while keeping
  the existing 55% line/function thresholds.
- Newer strict CI clippy may reject uninlined `format!` arguments or default
  insertion helpers that the local toolchain does not flag yet; the fix must
  satisfy CI without adding allows or weakening `-D warnings`.
- Newer strict CI clippy may also reject MCP JSON-RPC protocol response
  formatting that local cached workspace checks did not revisit before CI; the
  fix must keep the emitted response JSON unchanged.
- Newer strict CI clippy may continue into MCP test fixture response builders
  after the protocol response fix; the fix must keep every fixture payload
  shape unchanged.
- Newer strict CI clippy may also report the same fixture response-builder
  pattern in extracted MCP stdio protocol support and request helpers; the fix
  must keep framed request/response JSON unchanged.
- Remaining MCP test framing helpers and payload-length fixture builders may
  keep the same old-style format shape even when the current local toolchain
  does not flag them; normalize them proactively to avoid another CI-only
  strict-clippy round.
- Newer strict CI clippy may continue into kamn-node extraction-budget contract
  assertions after MCP fixture cleanup; the fix must keep the same budget
  assertion and source-marker proof semantics.
- Newer strict CI clippy may continue into durable cross-node relay slice
  doc-marker contract assertions after the gate reaches later kamn-node test
  targets; the fix must keep the same required marker list and doc proof
  semantics.
- Newer strict CI clippy may continue into signer module extraction budget
  assertions after the gate reaches later kamn-node contract tests; the fix
  must keep the same module, source-marker, and file-budget proof semantics.
- Newer strict CI clippy may continue into kamn-node command-surface and proof
  doc contract assertions after the gate reaches later test targets; the fix
  must keep the same required selectors, doc markers, and reason-code proof
  semantics.
- Newer strict CI clippy may continue into service API endpoint module
  extraction contract assertions after the gate reaches later kamn-node test
  targets; the fix must keep the same module, OpenAPI-marker, and line-budget
  proof semantics.
- Newer strict CI clippy may continue into main-tests surface-budget contract
  reason-code formatting after the gate reaches later kamn-node test targets;
  the fix must keep the same budget thresholds, reason taxonomy, and detail
  payload semantics.
- Newer strict CI clippy may continue into working vertical slice doc/root/test
  marker assertions after the gate reaches later kamn-node contract tests; the
  fix must keep the same local proof markers and integration-test wiring
  semantics.
- Newer strict CI clippy may report uninlined arguments in service API websocket
  mode validation, managed signer key material and response verification, and
  managed signer test helpers; the fix must keep emitted runtime errors and
  fixture command payloads unchanged.
- Newer strict CI clippy may report uninlined arguments in live Postgres
  projection/topology contracts, observability readiness metrics, task escrow
  endpoint shutdown checks, and signer managed-external helpers; the fix must
  keep reason-code, metrics, topology, shutdown, and signer payload semantics
  unchanged.
- Newer strict CI clippy may continue into SDK TCP vertical-slice doc and demo
  script marker assertions after the gate reaches kamn-sdk test targets; the
  fix must keep the same SDK TCP proof markers and script-marker semantics.
- Newer strict CI clippy may continue into SDK service API client
  contract-server support response builders after the gate reaches route-family
  fixture targets; the fix must keep the same JSON payloads, HTTP responses,
  deterministic IDs, and route-family fixture semantics.
- Newer strict CI clippy may continue into SDK live transport agent
  contract-server support response builders after the gate reaches the
  `live_transport_agent` test target; the fix must keep the same channel,
  message, agent registration, and live transport fixture semantics.
- Remaining SDK test/support assertions, HTTP response builders, endpoint
  formatting, temp-path helpers, and TCP handshake fixtures may still use
  old-style formatting that newer strict CI clippy rejects after the currently
  observed targets pass; normalize only format-macro call sites while leaving
  literal `"{}"` signed-payload bodies unchanged.
- Newer strict CI clippy may continue into the shell test surface ratio policy
  test/support modules after SDK cleanup; normalize format and panic call sites
  while preserving reason taxonomy strings, waiver validation messages, schema
  markers, and JSON report rendering.
- The touched-Rust size policy may reject the shell test surface ratio policy
  loader after strict-clippy formatting touches it; keep threshold loading split
  below the touched function limit without changing validation semantics.
- Newer strict CI clippy may continue into the p2p live transport runtime test
  support deadline assertion after shell ratio cleanup; normalize the timeout
  assertion format without changing drain polling or timeout behavior.
- Newer strict CI clippy may continue into the p2p libp2p native adapter
  runtime test support deadline assertion after the live transport cleanup;
  normalize the timeout assertion format without changing drain polling or
  timeout behavior.
- Workspace Pre-Merge may fail deterministic inventory/evidence contracts after
  the strict-clippy repair commits shift the current branch evidence window and
  script wrapper inventory; refresh only the observed branch-head counts and
  generated script-surface candidate counts without weakening the gates.
- Critical-path mutation may invoke stale test selectors after module splits,
  causing `cargo-mutants` slices to run zero tests and report escaped mutants
  even though the runtime contract still has current tests.
- Critical-path mutation may target a stale source line after signer module
  extraction, discovering zero mutants instead of proving the strict signer
  secret-source precedence contract.
- Fast Gate may cancel on elapsed-budget policy before reaching format, strict
  clippy, production panic-surface, or bounded test steps when the CI-tool
  regression bundle runs inside the same 20-minute job on a 969-file branch
  surface and cold Rust cache.
- The fast-mode CI-tool regression command may be CI-green on Ubuntu while
  failing locally on macOS if shell tests depend on GNU-only `date -d`
  behavior.
- Local full-stack dry-run contract tests may hard-code `/tmp/kolme_fork` even
  though the validator intentionally emits the resolved checkout path, which is
  `/private/tmp/kolme_fork` on macOS.
- Fast-mode CI-tool regression shell tests may use GNU `sed -i` fixture
  mutation syntax, which fails locally on BSD/macOS `sed` before the command
  can prove the same policy checks.
- Kolme dispatcher manifest metadata contract tests may use GNU `find -printf`
  to inventory wrapper symlinks, which fails locally on BSD/macOS `find` before
  the contract can prove dispatcher metadata resolution.

## Acceptance criteria

- `cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance` passes.
- `cargo test -p kamn-core --test script_surface_index_docs --test script_surface_reduction_candidates_docs --test test_file_size_policy` passes.
- `cargo test -p kamn-crypto --test direct_message_crypto_failure_paths` passes.
- `cargo test -p kamn-e2e-harness --test command_result_error_path_policy --test r57_high_gap_evidence_matrix_contract` passes.
- `cargo test -p kamn-e2e-harness --lib` passes.
- `cargo test -p kamn-node --bin kamn-node` passes.
- The kolme-live provider-marker contract still proves nonce, submit, and
  finality requests in order without relaxing the request-marker assertions.
- Daemon topology projection tests use isolated state and relay-spool paths for
  each projection and the shared service-api env lock while keeping the phase6
  reason-code assertions unchanged.
- Agent-lib task/escrow route contracts use a pre-bound fixture listener and
  bounded complete-request reads while preserving the four live route
  assertions.
- Agent-lib list-message route contracts use a pre-bound fixture listener and
  flushed response writes while preserving the live SDK route assertion.
- MCP real-backend integration contracts use pre-bound fixture listeners and
  flushed response writes while preserving real dispatch, stdio, content, and
  bridge assertions.
- SDK live-transport task/escrow contracts use pre-bound fixture listeners,
  complete request-body reads, and flushed responses while preserving task,
  content, escrow, malformed-payload, and fail-closed assertions.
- SDK service API client route-family contracts use pre-bound fixture listeners,
  bounded complete request-body reads, and flushed responses while preserving
  task, escrow, bridge, replay, websocket, and TLS assertions.
- Shell wrapper parity contracts use explicit bounded local pre-merge budgets
  for the nested cargo lanes while leaving the script default budget and CI
  command unchanged.
- The async runtime live-validation wrapper parity contract uses a 600s
  all-features pre-merge budget and still requires the pass/fail runtime
  markers in stdout and the JSON report.
- Mailbox relay endpoint fixtures wait for API readiness before issuing signed
  HTTP requests.
- Live bridge websocket delivery still requires a real forwarded event and
  non-placeholder live bridge evidence under a bounded read/request budget.
- Live bridge websocket presence delivery uses a bounded complete-response read
  for the live forward call while still requiring non-placeholder devnet proof
  markers in the forwarded frame.
- Signer private-key parse zeroization remains a bounded performance contract
  under full-suite load.
- MCP real-backend content lifecycle dispatch waits for a complete HTTP header
  before routing the request and still requires the `"ok":true` content
  lifecycle envelopes.
- Websocket upgrade harnesses hold scoped service-api env guards through server
  join while preserving the 101 upgrade and event-frame assertions.
- Auth-scope endpoint fixtures keep the service-api env guard alive through
  endpoint join while preserving route authz, scope rejection, and signature
  binding assertions.
- Snapshot-journal roundtrip tests use collision-resistant temp paths under
  same-process parallel execution while preserving exact payload restore
  assertions.
- Service API client contract-server support stays split below the touched-Rust
  file budget while preserving the same fixture entrypoints and route behavior.
- M4 settlement model and error taxonomy stay split below the touched-Rust file
  budget while preserving public re-exports and error display strings.
- Test-file size policy inventory accounting is refreshed by exactly one added
  bounded support module without increasing soft, severe, or hard oversized
  counts.
- Workspace Pre-Merge CI checkout fetches full history when it runs branch-head
  governance tests, matching the Fast Gate ratio job and preserving the
  fail-closed empty-window semantics.
- Critical-path coverage runner exact selectors match the current split module
  test names and a cheap Rust contract prevents stale selector drift before the
  expensive `cargo llvm-cov` lane.
- Critical-path coverage checker merges duplicate file metrics across core and
  node reports by preserving the maximum observed line/function percentages for
  each normalized filename.
- Critical-path coverage thresholds point at the extracted group-channel
  lifecycle module that owns executable coverage, with the previous 55% line
  and function thresholds unchanged.
- Direct-message crypto AAD, migration, escrow/task lifecycle,
  Postgres-bridge SQL rendering, bridge envelope proof rendering,
  block-pipeline schema errors, zk planning text, and native p2p queue
  insertion are strict-clippy clean under CI without changing emitted strings or
  behavior.
- MCP JSON-RPC success response rendering is strict-clippy clean under CI
  without changing the emitted response envelope.
- MCP tool-dispatch fixture response builders are strict-clippy clean under CI
  without changing structured fixture payloads.
- MCP stdio protocol fixture response builders and request helpers are
  strict-clippy clean under CI without changing framed protocol payloads.
- MCP test framing helpers and payload-length fixture builders are normalized
  to strict-clippy style without changing HTTP, stdio, or fixture JSON payloads.
- Message-store runtime-evidence extraction budget assertions are strict-clippy
  clean under CI without changing line-budget or marker checks.
- Durable cross-node relay slice doc-marker assertions are strict-clippy clean
  under CI without changing the required marker list or doc proof semantics.
- Signer module extraction budget assertions are strict-clippy clean under CI
  without changing module presence, source-marker, or file-budget proof
  semantics.
- Main-tests command-surface parity, runtime proof index, and restart
  persistence proof contract assertions are strict-clippy clean under CI
  without changing required selectors, doc markers, or reason-code proof
  semantics.
- Service API endpoint module extraction assertions are strict-clippy clean
  under CI without changing module declarations, OpenAPI markers, or line-budget
  proof semantics.
- Main-tests surface-budget contract formatting is strict-clippy clean under
  CI without changing budget thresholds, reason taxonomy, or detail payload
  semantics.
- Working vertical slice proof-marker assertions are strict-clippy clean under
  CI without changing the doc markers, root module wiring assertion, or
  integration-test marker semantics.
- Service API websocket mode validation and managed signer runtime errors are
  strict-clippy clean under CI without changing their emitted messages.
- Live Postgres projection/topology, observability readiness, task escrow
  shutdown, and managed signer helper payloads are strict-clippy clean under CI
  without changing their reason codes, metric strings, topology checks, or
  fixture command output.
- SDK TCP vertical-slice doc and script marker assertions are strict-clippy
  clean under CI without changing the required marker lists or local SDK demo
  proof semantics.
- SDK service API client contract-server support response builders are
  strict-clippy clean under CI without changing response payload shapes,
  status codes, deterministic IDs, or route-family behavior.
- SDK live transport agent contract-server support response builders are
  strict-clippy clean under CI without changing response payload shapes,
  registration metadata, or live transport route behavior.
- Remaining SDK test/support format-macro call sites are strict-clippy clean
  under CI without changing assertion meanings, wire payloads, HTTP response
  bytes, temp path uniqueness, endpoint values, or handshake signatures; literal
  `"{}"` signed-payload bodies remain literal.
- Shell test surface ratio policy tests are strict-clippy clean under CI without
  changing reason-code output, waiver parsing, schema marker assertions, current
  surface counting, or JSON report payloads.
- Shell test surface ratio policy support remains below touched-Rust function
  limits while preserving `threshold_value_invalid` validation for negative
  threshold deltas.
- P2P live transport runtime support is strict-clippy clean under CI while
  preserving expected-frame deadline messages, timeout values, and polling
  cadence.
- P2P libp2p native adapter runtime support is strict-clippy clean under CI
  while preserving expected-frame deadline messages, timeout values, and
  polling cadence.
- The current-head governance ratio contract matches the real checker output
  for `HEAD` and remains below the `0.20` ceiling.
- The script-surface reduction candidate doc matches filesystem inventory,
  including the refreshed `scripts/ci` short-wrapper count and total candidate
  marker.
- Test-file size policy inventory accounting is refreshed by exactly one added
  coverage-gate contract test, with oversized counts unchanged.
- Critical-path mutation runner test selectors match the current split module
  test names for group-channel crypto and service API replay rejection.
- Critical-path mutation runner signer selectors target the current signer
  secret-provider implementation instead of stale `signer.rs` line numbers.
- Critical-path mutation runner still expects 10 mutants across 6 slices and
  catches all selected mutants locally or in CI.
- Fast Gate still runs format, strict clippy, production panic-surface, bounded
  tests, runtime fuzz, performance smoke, and budget telemetry in the
  `Fast Gate (PR)` job.
- CI-tool regression coverage remains PR-blocking through a separate
  `CI Tool Regression Gate (PR)` job that runs the same
  `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` entrypoint
  under its own 20-minute budget when selector output requires CI tool checks.
- The `Fast Gate (PR)` job no longer runs the CI-tool regression bundle inside
  its own runtime budget.
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` is locally
  runnable on the shared macOS workspace for its shell-test date fixtures
  without weakening Ubuntu CI behavior.
- Local full-stack dry-run contract tests compare the Kolme checkout marker
  against the platform-resolved `/tmp/kolme_fork` default while preserving the
  validator's existing resolved-path output semantics.
- Missing-docs policy shell tests mutate fixtures through a portable in-place
  sed helper that works on both GNU/Linux and BSD/macOS sed.
- Kolme dispatcher manifest metadata contract tests inventory wrapper symlinks
  without GNU-only `find -printf` while preserving the non-empty wrapper
  inventory and manifest metadata assertions.
- `cargo test --workspace --locked --all-features --no-fail-fast` passes locally or remaining unrelated failures are separately filed with evidence.
- `make check` remains green.
- The PR head is governance-ratio compliant.

## Files to touch

Likely:

- `docs/developer/script-surface-index.md`
- `docs/developer/script-surface-reduction-candidates.md`
- `fixtures/ci/test_file_size_policy_baseline.env`
- `crates/kamn-core/tests/script_surface_index_docs.rs`
- `crates/kamn-core/tests/script_surface_reduction_candidates_docs.rs`
- `crates/kamn-core/tests/test_file_size_policy.rs`
- `crates/kamn-crypto/tests/direct_message_crypto_failure_paths.rs`
- `crates/kamn-e2e-harness/tests/command_result_error_path_policy.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent_tests/tool_call_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_projection_contract_tests/live_bridge_presence_stream_contract_tests.rs`
- `crates/kamn-agent-lib/tests/task_and_escrow_service_contract.rs`
- `crates/kamn-core/tests/shell_test_surface_migration_wave1/wrapper_parity_contract_tests/coverage_guided_wrapper_contract_tests.rs`
- `crates/kamn-core/tests/shell_test_surface_migration_wave1/wrapper_parity_contract_tests/runtime_wrapper_contract_tests/runtime_validation_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/support.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests/live_bridge_delivery_contract_tests.rs`
- `crates/kamn-node/src/signer/tests/adapter_contract_tests.rs`
- `crates/kamn-node/tests/signer_module_extraction_contract.rs`
- `crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs`
- `crates/kamn-node/tests/runtime_proof_index_contract.rs`
- `crates/kamn-node/tests/restart_persistence_proof_contract.rs`
- `crates/kamn-node/tests/service_api_endpoint_module_extraction_contract.rs`
- `crates/kamn-node/tests/main_tests_surface_budget_contract.rs`
- `crates/kamn-node/tests/working_vertical_slice_contract.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-node/src/signer/managed_backend/key_material.rs`
- `crates/kamn-node/src/signer/managed_backend/response/verification.rs`
- `crates/kamn-node/src/signer/managed_backend/tests/support.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/projection_taxonomy_contract_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_mapping_contract_tests/host_pair_directionality_contract_tests.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests/endpoint_runtime_contract_tests/readiness_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/support/state_support.rs`
- `crates/kamn-node/src/main_tests/signer_tests/signer_managed_external_contract_tests/support.rs`
- `crates/kamn-sdk/tests/sdk_tcp_vertical_slice_contract.rs`
- `crates/kamn-sdk/tests/service_api_client/support/contract_server_support/agent_content_route_support.rs`
- `crates/kamn-sdk/tests/service_api_client/support/contract_server_support/bridge_route_support.rs`
- `crates/kamn-sdk/tests/service_api_client/support/contract_server_support/public_route_support.rs`
- `crates/kamn-sdk/tests/service_api_client/support/contract_server_support/message_task_route_support/channel_task_route_support.rs`
- `crates/kamn-sdk/tests/service_api_client/support/contract_server_support/message_task_route_support/message_route_support.rs`
- `crates/kamn-sdk/tests/service_api_client/support/contract_server_support/message_task_route_support/task_mutation_route_support.rs`
- `crates/kamn-sdk/tests/live_transport_agent/support/contract_server_support/response_route_support/agent_route_support.rs`
- `crates/kamn-sdk/tests/live_transport_agent/support/contract_server_support/response_route_support/channel_route_support.rs`
- `crates/kamn-sdk/tests/live_transport_agent/support/contract_server_support/response_route_support/message_route_support.rs`
- `crates/kamn-sdk/tests/service_api_client_extraction_contract.rs`
- `crates/kamn-sdk/tests/tcp_module_extraction_contract.rs`
- `crates/kamn-sdk/tests/rust_sdk_alpha_docs.rs`
- `crates/kamn-sdk/tests/tcp_transport_adapter/envelope_validation_contract_tests.rs`
- `crates/kamn-sdk/tests/support/live_transport_contract_server.rs`
- `crates/kamn-sdk/tests/service_api_client/support/env_support.rs`
- `crates/kamn-sdk/tests/live_transport_observability.rs`
- `crates/kamn-sdk/tests/support/tcp_handshake_constant_time_support.rs`
- `crates/kamn-sdk/tests/live_transport_agent/message_contract_tests.rs`
- `crates/kamn-sdk/tests/service_api_client/support/request_parse_support.rs`
- `crates/kamn-sdk/tests/live_transport_agent/support/request_parse_support.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/support/fixtures.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/support/loading.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/support/paths.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/support/report.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/support/current_surface.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/support/evaluation.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/evaluation_gate_contract_tests.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/schema_fixture_contract_tests.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/waiver_contract_tests.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime/support/transport_io_support.rs`
- `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime/support.rs`
- `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance/current_head_status_contract_tests.rs`
- `crates/kamn-core/tests/script_surface_reduction_candidates_docs.rs`
- `docs/developer/script-surface-reduction-candidates.md`
- `crates/kamn-agent-lib/tests/list_messages_service_contract.rs`
- `crates/kamn-mcp-server/tests/real_backend_integration_contract.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow/*`
- `crates/kamn-sdk/tests/service_api_client/*`
- `crates/kamn-sdk/tests/service_api_client/support/contract_server_support/*`
- `crates/kamn-node/src/main_tests/runtime_tests/kolme_live_execution_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures/matrix_profiles/matrix_projection.rs`
- `fixtures/runtime/r57_high_gap_evidence_matrix.txt`
- `.github/workflows/ci-fast-gate.yml`
- `crates/kamn-core/tests/fast_gate_governance_helper_contract.rs`
- `scripts/ci/run_critical_path_coverage_gate.sh`
- `scripts/ci/check_critical_path_coverage.py`
- `scripts/ci/test_check_critical_path_coverage.sh`
- `.ci/critical-path-coverage-thresholds.json`
- `crates/kamn-core/tests/critical_path_coverage_gate_contract.rs`
- `scripts/ci/run_critical_path_mutation_gate.sh`
- `scripts/ci/test_run_critical_path_mutation_gate.sh`
- `crates/kamn-core/tests/critical_path_mutation_gate_contract.rs`
- `crates/kamn-crypto/src/direct_message_crypto/cipher.rs`
- `crates/kamn-core/src/block_pipeline/commit_store.rs`
- `crates/kamn-core/src/bridge_adapter/engine/envelope.rs`
- `crates/kamn-core/src/data_layer_m4_escrow_integration/models/settlement.rs`
- `crates/kamn-core/src/data_layer_m4_escrow_integration/models/settlement_error.rs`
- `crates/kamn-core/src/data_layer_m4_escrow_integration/models.rs`
- `crates/kamn-core/src/data_layer_postgres_repository_bridge/m6_age.rs`
- `crates/kamn-core/src/data_layer_postgres_repository_bridge/m7_timescale.rs`
- `crates/kamn-core/src/escrow.rs`
- `crates/kamn-core/src/migrations.rs`
- `crates/kamn-core/src/p2p_transport/native_runtime.rs`
- `crates/kamn-core/src/task_lifecycle.rs`
- `crates/kamn-core/src/zk_message_proofs/planning/recommendation.rs`
- `crates/kamn-mcp-server/src/protocol.rs`
- `crates/kamn-mcp-server/tests/tool_dispatch_contract.rs`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract/support.rs`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract/tool_dispatch_contract_tests.rs`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract/initialize_inventory_contract_tests.rs`
- `crates/kamn-mcp-server/tests/real_backend_integration_contract.rs`
- `crates/kamn-mcp-server/tests/main_stdio_persistent_contract.rs`
- `crates/kamn-mcp-server/tests/tool_inventory_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env`

Git history may also need a local fold of spec-only checkpoint commits into
mixed implementation commits so the final PR head remains governance compliant.
Any such fold must preserve a backup branch before rewriting local history.

## Error semantics

This issue does not alter runtime error behavior. Contract failures must remain
hard failures with explicit assertion messages.

## Test plan

Red evidence:

```bash
cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance -- --nocapture
cargo test -p kamn-core --test script_surface_index_docs --test script_surface_reduction_candidates_docs --test test_file_size_policy
cargo test -p kamn-crypto --test direct_message_crypto_failure_paths
cargo test -p kamn-e2e-harness --test command_result_error_path_policy --test r57_high_gap_evidence_matrix_contract
cargo test -p kamn-e2e-harness --lib
cargo test -p kamn-agent-lib --test list_messages_service_contract -- --nocapture
cargo test -p kamn-agent-lib --test task_and_escrow_service_contract -- --nocapture
cargo test -p kamn-mcp-server --test real_backend_integration_contract -- --nocapture
cargo test -p kamn-sdk --test live_transport_task_escrow -- --nocapture
cargo test -p kamn-sdk --test service_api_client -- --nocapture
cargo test -p kamn-core --test shell_test_surface_migration_wave1 -- --nocapture
cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::integration_runtime_kolme_live_renders_provider_contract_markers -- --exact --nocapture
cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::lane_set_bundle_contract_tests::lane_id_bundle_contract_tests::functional_live_postgres_topology_lane_id_bundle_rows_are_canonical -- --exact --nocapture
cargo test -p kamn-node --bin kamn-node
cargo test -p kamn-core --test critical_path_coverage_gate_contract -- --nocapture
bash scripts/ci/test_check_critical_path_coverage.sh
gh run view 28321353896 --json status,conclusion,jobs,url,headSha,workflowName
gh api repos/njfio/kamn/actions/jobs/83903742305/logs
gh api repos/njfio/kamn/actions/jobs/83903742298/logs
cargo test -p kamn-core --test critical_path_mutation_gate_contract -- --nocapture
cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender -- --list
cargo mutants -p kamn-core --file crates/kamn-core/src/group_channel_crypto/engine/sealing/encrypt.rs --re 'encrypt\.rs:([0-9]+:[0-9]+): replace == with !=' --output /tmp/kamn-mutants-core-group-channel-crypto --copy-vcs true --cargo-test-arg --lib --cargo-test-arg group_channel_crypto::tests::encrypt_requires_key_agreement_seed --timeout 900
cargo mutants -p kamn-node --file crates/kamn-node/src/service_api_endpoint.rs --re 'ServiceApiReplayGuard::record_nonce_if_fresh -> bool with (true|false)' --output /tmp/kamn-mutants-node-service-api-endpoint --copy-vcs true --cargo-test-arg --bin --cargo-test-arg kamn-node --cargo-test-arg main_tests::service_api_endpoint_tests::regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender --timeout 900
gh api repos/njfio/kamn/actions/jobs/83917258068/logs
gh api repos/njfio/kamn/actions/jobs/83923030478/logs
gh api repos/njfio/kamn/actions/jobs/83925054489/logs
rg -n '"[^"]*\{\}[^"]*",|"[^"]*\{:\?\}[^"]*",|panic!\("[^"]*\{\}[^"]*",|format!\(\s*"[^"]*\{\}[^"]*"' crates/kamn-sdk/tests
gh api repos/njfio/kamn/actions/jobs/83961695012/logs
gh api repos/njfio/kamn/actions/jobs/83964490689/logs
gh api repos/njfio/kamn/actions/jobs/83966724810/logs
gh api repos/njfio/kamn/actions/jobs/83966724799/logs
cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance -- --nocapture
cargo test -p kamn-core --test script_surface_reduction_candidates_docs -- --nocapture
```

Green verification:

```bash
cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance -- --nocapture
cargo test -p kamn-core --test script_surface_index_docs --test script_surface_reduction_candidates_docs --test test_file_size_policy
cargo test -p kamn-crypto --test direct_message_crypto_failure_paths
cargo test -p kamn-e2e-harness --test command_result_error_path_policy --test r57_high_gap_evidence_matrix_contract
cargo test -p kamn-e2e-harness --lib
cargo test -p kamn-agent-lib --test list_messages_service_contract -- --nocapture
cargo test -p kamn-agent-lib --test task_and_escrow_service_contract -- --nocapture
cargo test -p kamn-mcp-server --test real_backend_integration_contract -- --nocapture
cargo test -p kamn-sdk --test live_transport_task_escrow -- --nocapture
cargo test -p kamn-sdk --test service_api_client -- --nocapture
cargo test -p kamn-core --test shell_test_surface_migration_wave1 -- --nocapture
cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::integration_runtime_kolme_live_renders_provider_contract_markers -- --exact --nocapture
cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::lane_set_bundle_contract_tests::lane_id_bundle_contract_tests::functional_live_postgres_topology_lane_id_bundle_rows_are_canonical -- --exact --nocapture
cargo test -p kamn-node --bin kamn-node
cargo test -p kamn-core --test critical_path_coverage_gate_contract -- --nocapture
bash scripts/ci/test_check_critical_path_coverage.sh
cargo test -p kamn-core --test critical_path_mutation_gate_contract -- --nocapture
bash scripts/ci/test_run_critical_path_mutation_gate.sh
bash scripts/ci/run_critical_path_mutation_gate.sh --output-json /tmp/kamn-critical-path-mutation-7035-final.json --timeout-seconds 900
cargo clippy -p kamn-crypto --all-targets --all-features -- -D warnings
cargo clippy -p kamn-mcp-server --all-targets --all-features -- -D warnings
cargo test -p kamn-mcp-server --test tool_dispatch_contract -- --nocapture
cargo test -p kamn-mcp-server --test stdio_protocol_contract -- --nocapture
cargo test -p kamn-sdk --test service_api_client_extraction_contract -- --nocapture
cargo test -p kamn-sdk --test tcp_module_extraction_contract -- --nocapture
cargo test -p kamn-sdk --test rust_sdk_alpha_docs -- --nocapture
cargo test -p kamn-sdk --test tcp_transport_adapter -- --nocapture
cargo test -p kamn-sdk --test service_api_client -- --nocapture
cargo test -p kamn-sdk --test live_transport_agent -- --nocapture
cargo test -p kamn-sdk --test live_transport_observability -- --nocapture
cargo clippy -p kamn-sdk --all-targets --all-features -- -D warnings
cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture
cargo clippy -p kamn-core --test shell_test_surface_ratio_policy --all-features -- -D warnings
cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture
cargo clippy -p kamn-core --test p2p_live_transport_runtime --all-features -- -D warnings
cargo test -p kamn-core --test p2p_libp2p_native_adapter_runtime -- --nocapture
cargo clippy -p kamn-core --test p2p_libp2p_native_adapter_runtime --all-features -- -D warnings
cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance -- --nocapture
cargo test -p kamn-core --test script_surface_reduction_candidates_docs -- --nocapture
LLVM_COV=/Users/n/.rustup/toolchains/1.90.0-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-cov LLVM_PROFDATA=/Users/n/.rustup/toolchains/1.90.0-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata bash scripts/ci/run_critical_path_coverage_gate.sh --threshold-file .ci/critical-path-coverage-thresholds.json --core-json /tmp/kamn-critical-path-core-7035-selector-final.json --node-json /tmp/kamn-critical-path-node-7035-selector-final.json --output-json /tmp/kamn-critical-path-policy-7035-selector-final.json
cargo test --workspace --locked --all-features --no-fail-fast
make check
```

Closeout evidence captured on 2026-06-28:

```bash
cargo test -p kamn-core --test critical_path_mutation_gate_contract -- --nocapture
# 4 passed

bash scripts/ci/test_run_critical_path_mutation_gate.sh
# critical-path mutation gate script tests passed.

bash -n scripts/ci/run_critical_path_mutation_gate.sh scripts/ci/test_run_critical_path_mutation_gate.sh
# passed

bash scripts/ci/run_critical_path_mutation_gate.sh --output-json /tmp/kamn-critical-path-mutation-7035-final.json --timeout-seconds 900
# status=ok
# final_decision=GO
# reason_codes_csv=none
# slice_count=6
# tested_mutants=10
# caught_mutants=10
# missed_mutants=0
# unviable_mutants=0
# timeout_mutants=0

gh api repos/njfio/kamn/actions/jobs/83917258068/logs
# Fast Gate strict clippy exited 101 on CI-only diagnostics in kamn-core:
# uninlined format args and or_insert_with default insertion.

cargo fmt --check
# passed after applying rustfmt

cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed locally

make check
# passed locally

cargo test -p kamn-core --lib
# 463 passed, 5 ignored

wc -l crates/kamn-core/src/data_layer_m4_escrow_integration/models/settlement.rs crates/kamn-core/src/data_layer_m4_escrow_integration/models/settlement_error.rs
# 76 settlement.rs
# 181 settlement_error.rs

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-settlement-split.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-settlement-function-split.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

cargo test -p kamn-core --lib data_layer_m4_escrow_integration -- --nocapture
# 2 passed

gh api repos/njfio/kamn/actions/jobs/83923030478/logs
# Fast Gate strict clippy exited 101 on `crates/kamn-mcp-server/src/protocol.rs:389`
# with `clippy::uninlined_format_args`; the same Fast Gate run also exceeded
# its elapsed budget after the failed lint lane.

cargo fmt --check
# passed

cargo test -p kamn-mcp-server --lib protocol -- --nocapture
# 8 passed

cargo clippy -p kamn-mcp-server --all-targets --all-features -- -D warnings
# passed

cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-mcp-protocol-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83925054489/logs
# Fast Gate strict clippy exited 101 on
# `crates/kamn-mcp-server/tests/tool_dispatch_contract.rs` fixture response
# builders with `clippy::uninlined_format_args`.

cargo test -p kamn-mcp-server --test tool_dispatch_contract -- --nocapture
# 9 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-mcp-server --all-targets --all-features -- -D warnings
# passed after `cargo clean -p kamn-mcp-server`

cargo fmt --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-mcp-tool-dispatch-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83927107079/logs
# Fast Gate strict clippy reported `clippy::uninlined_format_args` in
# `crates/kamn-mcp-server/tests/stdio_protocol_contract/support.rs` and
# `crates/kamn-mcp-server/tests/stdio_protocol_contract/tool_dispatch_contract_tests.rs`.
# The same run exceeded the fast-gate elapsed budget after lint cancellation.

cargo test -p kamn-mcp-server --test stdio_protocol_contract --test tool_dispatch_contract --test tool_inventory_contract --test main_stdio_persistent_contract --test real_backend_integration_contract -- --nocapture
# 35 passed

cargo clean -p kamn-mcp-server
CARGO_INCREMENTAL=0 cargo clippy -p kamn-mcp-server --all-targets --all-features -- -D warnings
# passed

cargo fmt --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-mcp-stdio-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83932840209/logs
# Fast Gate attempt 2 reached strict clippy after Workspace Pre-Merge passed and
# reported `clippy::uninlined_format_args` in
# `crates/kamn-node/tests/message_store_runtime_evidence_extraction_contract.rs`.

cargo test -p kamn-node --test message_store_runtime_evidence_extraction_contract -- --nocapture
# 3 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --all-targets --all-features -- -D warnings
# passed

cargo fmt --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-node-runtime-evidence-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract -- --nocapture
# red before workflow split: fast_gate_job_must_not_run_ci_tool_regression_bundle
# and workflow job section missing: ci-tool-regression-gate

bash scripts/kolme/test_dispatcher_manifest_metadata_contract.sh
# red before portable find fix:
# find: -printf: unknown primary or operator
# expected run and contract wrapper symlink inventory to be non-empty

cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract -- --nocapture
# 10 passed

bash scripts/kolme/test_dispatcher_manifest_metadata_contract.sh
# Kolme dispatcher manifest metadata contract tests passed.

uv run --python 3.11 --with cryptography /tmp/kamn-bash5-bin/bash -lc 'PATH=/tmp/kamn-bash5-bin:$PATH; set -o pipefail; KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh 2>&1 | tee /tmp/kamn-ci-tools-fast-mode-uv-py311-bash5-pipefail-final.log'
# Fast-mode CI tool regression tests passed.

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-final-fast-gate-split.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83944719211/logs
# Fast Gate reached Format check and then failed Lint (strict) on
# `crates/kamn-node/tests/durable_cross_node_relay_slice_contract.rs` with
# `clippy::uninlined_format_args`.

cargo test -p kamn-node --test durable_cross_node_relay_slice_contract -- --nocapture
# 1 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --test durable_cross_node_relay_slice_contract --all-features -- -D warnings
# passed

cargo fmt --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-durable-cross-node-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83945902499/logs
# Fast Gate reached Format check and then failed Lint (strict) on
# `crates/kamn-node/tests/signer_module_extraction_contract.rs` with
# `clippy::uninlined_format_args`.

cargo test -p kamn-node --test signer_module_extraction_contract -- --nocapture
# 1 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --test signer_module_extraction_contract --all-features -- -D warnings
# passed

rg -n "\{\}" crates/kamn-node/tests/signer_module_extraction_contract.rs
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-signer-extraction-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83947261892/logs
# Fast Gate reached late Lint (strict) after runtime contract lanes and failed
# on `clippy::uninlined_format_args` in:
# - `crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs`
# - `crates/kamn-node/tests/runtime_proof_index_contract.rs`
# - `crates/kamn-node/tests/restart_persistence_proof_contract.rs`

cargo test -p kamn-node --test main_tests_command_surface_parity_contract --test runtime_proof_index_contract --test restart_persistence_proof_contract -- --nocapture
# 5 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --test main_tests_command_surface_parity_contract --test runtime_proof_index_contract --test restart_persistence_proof_contract --all-features -- -D warnings
# passed

rg -n "\{\}" crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs crates/kamn-node/tests/runtime_proof_index_contract.rs crates/kamn-node/tests/restart_persistence_proof_contract.rs
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-command-proof-doc-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83948726724/logs
# Fast Gate reached late Lint (strict) and failed on
# `crates/kamn-node/tests/service_api_endpoint_module_extraction_contract.rs`
# with `clippy::uninlined_format_args`.

cargo test -p kamn-node --test service_api_endpoint_module_extraction_contract -- --nocapture
# 5 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --test service_api_endpoint_module_extraction_contract --all-features -- -D warnings
# passed

rg -n "\{\}" crates/kamn-node/tests/service_api_endpoint_module_extraction_contract.rs
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-service-api-endpoint-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed

gh api repos/njfio/kamn/actions/jobs/83949991287/logs
# Fast Gate reached late Lint (strict) and failed on
# `crates/kamn-node/tests/main_tests_surface_budget_contract.rs` with
# `clippy::uninlined_format_args`.

cargo test -p kamn-node --test main_tests_surface_budget_contract -- --nocapture
# 1 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --test main_tests_surface_budget_contract --all-features -- -D warnings
# passed

rg -n "\{\}" crates/kamn-node/tests/main_tests_surface_budget_contract.rs
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-main-tests-surface-budget-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-29 for head
`c20b03b670622820b1dd8386772a178a84be2407` Fast Gate follow-up:

```bash
gh api repos/njfio/kamn/actions/jobs/83964490689/logs
# Fast Gate reached late Lint (strict) and failed on
# `clippy::uninlined_format_args` in
# `crates/kamn-core/tests/p2p_live_transport_runtime/support/transport_io_support.rs`.

rg -n '"[^"]*\{\}[^"]*",|"[^"]*\{:\?\}[^"]*",|panic!\("[^"]*\{\}[^"]*",|format!\(\s*"[^"]*\{\}[^"]*"|format!\(\s*"[^"]*\{:\.[0-9]+\}[^"]*"' crates/kamn-core/tests/p2p_live_transport_runtime
# no matches

cargo fmt --check
# passed

git diff --check
# passed

cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture
# 14 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-core --test p2p_live_transport_runtime --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-core --test p2p_live_transport_runtime --all-features -- -D warnings
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-p2p-runtime-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy -p kamn-core --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-core --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-29 for head
`fea3bdb4fc35f8908f347b2c79c85790a313ee8d` Fast Gate follow-up:

```bash
gh api repos/njfio/kamn/actions/jobs/83961695012/logs
# Fast Gate reached late Lint (strict) and failed on
# `clippy::uninlined_format_args` in
# `crates/kamn-core/tests/shell_test_surface_ratio_policy` support,
# report, path, loading, and waiver assertion helpers.

rg -n '"[^"]*\{\}[^"]*",|"[^"]*\{:\?\}[^"]*",|panic!\("[^"]*\{\}[^"]*",|format!\(\s*"[^"]*\{\}[^"]*"|format!\(\s*"[^"]*\{:\.[0-9]+\}[^"]*"' crates/kamn-core/tests/shell_test_surface_ratio_policy
# no matches

cargo fmt --check
# passed

git diff --check
# passed

cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture
# 4 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-core --test shell_test_surface_ratio_policy --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-core --test shell_test_surface_ratio_policy --all-features -- -D warnings
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-shell-ratio-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy -p kamn-core --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-core --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-29 for the remaining SDK
test/support strict-clippy formatting sweep:

```bash
rg -n '"[^"]*\{\}[^"]*",|"[^"]*\{:\?\}[^"]*",|panic!\("[^"]*\{\}[^"]*",|format!\(\s*"[^"]*\{\}[^"]*"' crates/kamn-sdk/tests
# only literal `"{}"` signed-payload body arguments to `auth_with_scope(...)`
# remain; no format-macro call sites matched.

cargo fmt --check
# passed

git diff --check
# passed

cargo test -p kamn-sdk --test service_api_client_extraction_contract --test tcp_module_extraction_contract --test rust_sdk_alpha_docs --test tcp_transport_adapter --test service_api_client --test live_transport_agent --test live_transport_observability -- --nocapture
# 53 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-sdk --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-sdk --all-targets --all-features -- -D warnings
# passed

cargo test -p kamn-sdk --all-features -- --nocapture
# passed; 1 ignored; doc-tests: 0

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-sdk-remaining-format-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-28 for head
`f7b596910e93552f8115d86be5f3b6c05c74c332` Fast Gate follow-up:

```bash
gh api repos/njfio/kamn/actions/jobs/83957969743/logs
# Fast Gate reached late Lint (strict) and failed on
# `clippy::uninlined_format_args` in SDK live transport agent
# contract-server support response builders:
# - `response_route_support/channel_route_support.rs`
# - `response_route_support/message_route_support.rs`

cargo test -p kamn-sdk --test live_transport_agent -- --nocapture
# 16 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-sdk --test live_transport_agent --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-sdk --test live_transport_agent --all-features -- -D warnings
# passed

rg -n '"[^\"]*\{\}[^\"]*",|"[^\"]*\{:\?\}[^\"]*",|panic!\("[^\"]*\{\}[^\"]*",|format!\(\s*"[^\"]*\{\}[^\"]*"' crates/kamn-sdk/tests/live_transport_agent/support/contract_server_support
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-sdk-live-transport-agent-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-28 for head
`650191f8c2346b527865e0d7bd8bf6983205ab55` Fast Gate follow-up:

```bash
gh api repos/njfio/kamn/actions/jobs/83956010808/logs
# Fast Gate reached late Lint (strict) and failed on
# `clippy::uninlined_format_args` in SDK service API client contract-server
# support response builders:
# - `agent_content_route_support.rs`
# - `bridge_route_support.rs`
# - `message_task_route_support/channel_task_route_support.rs`
# - `message_task_route_support/message_route_support.rs`
# - `message_task_route_support/task_mutation_route_support.rs`

cargo test -p kamn-sdk --test service_api_client -- --nocapture
# 17 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-sdk --test service_api_client --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-sdk --test service_api_client --all-features -- -D warnings
# passed

rg -n '"[^\"]*\{\}[^\"]*",|"[^\"]*\{:\?\}[^\"]*",|panic!\("[^\"]*\{\}[^\"]*",|format!\(\s*"[^\"]*\{\}[^\"]*"' crates/kamn-sdk/tests/service_api_client/support/contract_server_support
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-sdk-service-api-fixture-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-28 for head
`9f872e86509c8554cd8ddbf77c0a31329354c6bc` Fast Gate follow-up:

```bash
gh api repos/njfio/kamn/actions/jobs/83954008054/logs
# Fast Gate reached late Lint (strict) and failed on
# `crates/kamn-sdk/tests/sdk_tcp_vertical_slice_contract.rs` with
# `clippy::uninlined_format_args`.

cargo test -p kamn-sdk --test sdk_tcp_vertical_slice_contract -- --nocapture
# 2 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-sdk --test sdk_tcp_vertical_slice_contract --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-sdk --test sdk_tcp_vertical_slice_contract --all-features -- -D warnings
# passed

rg -n '"[^\"]*\{\}[^\"]*",|"[^\"]*\{:\?\}[^\"]*",|panic!\("[^\"]*\{\}[^\"]*",|format!\(\s*"[^\"]*\{\}[^\"]*"' crates/kamn-sdk/tests/sdk_tcp_vertical_slice_contract.rs
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-sdk-tcp-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-28 for head
`289ac953ba8f571be4c773c4d606e1deb858bb51` Fast Gate follow-up:

```bash
gh api repos/njfio/kamn/actions/jobs/83951144588/logs
# Fast Gate reached late Lint (strict) and failed on
# `clippy::uninlined_format_args` in:
# - `crates/kamn-node/tests/working_vertical_slice_contract.rs`
# - `crates/kamn-node/src/service_api_endpoint/websocket.rs`
# - `crates/kamn-node/src/signer/managed_backend/key_material.rs`
# - `crates/kamn-node/src/signer/managed_backend/response/verification.rs`
# - `crates/kamn-node/src/signer/managed_backend/tests/support.rs`
# - live Postgres, observability, task-escrow, and signer main-test helpers.

cargo test -p kamn-node --test working_vertical_slice_contract -- --nocapture
# 3 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --test working_vertical_slice_contract --all-features -- -D warnings
# passed

cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical -- --exact --nocapture
# 1 passed

cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::host_pair_directionality_contract_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_contract_is_canonical -- --exact --nocapture
# 1 passed

cargo test -p kamn-node --bin kamn-node main_tests::observability_endpoint_tests::endpoint_runtime_contract_tests::readiness_contract_tests::functional_observability_endpoint_projects_readiness_reason_code_parity_across_endpoint_surfaces -- --exact --nocapture
# 1 passed

cargo test -p kamn-node --bin kamn-node main_tests::signer_tests::signer_managed_external_contract_tests::backend_provenance_contract_tests::regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch -- --exact --nocapture
# 1 passed

cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_routes_contract_tests::integration_service_api_endpoint_persists_task_and_escrow_state_across_routes -- --exact --nocapture
# 1 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-node --all-targets --all-features -- -D warnings
# passed

rustup run stable rustc --version
# rustc 1.89.0 (29483883e 2025-08-04)

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-node --all-targets --all-features -- -D warnings
# passed

rg -n '"[^\"]*\{\}[^\"]*",|"[^\"]*\{:\?\}[^\"]*",|panic!\("[^\"]*\{\}[^\"]*",|format!\(\s*"[^\"]*\{\}[^\"]*"' \
  crates/kamn-node/tests/working_vertical_slice_contract.rs \
  crates/kamn-node/src/service_api_endpoint/websocket.rs \
  crates/kamn-node/src/signer/managed_backend/key_material.rs \
  crates/kamn-node/src/signer/managed_backend/response/verification.rs \
  crates/kamn-node/src/signer/managed_backend/tests/support.rs \
  crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/projection_taxonomy_contract_tests.rs \
  crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_mapping_contract_tests/host_pair_directionality_contract_tests.rs \
  crates/kamn-node/src/main_tests/observability_endpoint_tests/endpoint_runtime_contract_tests/readiness_contract_tests.rs \
  crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/support/state_support.rs \
  crates/kamn-node/src/main_tests/signer_tests/signer_managed_external_contract_tests/support.rs
# no matches

cargo fmt --check
# passed

git diff --check
# passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-node-runtime-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

make check
# passed
```

Closeout evidence captured on 2026-06-29 for head
`2e1bb6d3d23808ce8059dffca9b6f93af623dc7c` Fast Gate and Workspace Pre-Merge follow-up:

```bash
gh api repos/njfio/kamn/actions/jobs/83966724810/logs
# Fast Gate strict clippy failed on
# `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime/support.rs`
# with `clippy::uninlined_format_args` in the expected-frame timeout message.

gh api repos/njfio/kamn/actions/jobs/83966724799/logs
# Workspace Pre-Merge full tests failed deterministic current-head evidence:
# governance ratio current-head counts drifted from 10/40 to 8/42 while staying
# `status=ok`, and script-surface candidate docs expected 63 total candidates
# while filesystem inventory reported 62.

rg -n '"[^"]*\{\}[^"]*",|"[^"]*\{:\?\}[^"]*"' crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime
# no matches

cargo fmt --check
# passed

git diff --check
# passed

cargo test -p kamn-core --test p2p_libp2p_native_adapter_runtime --all-features -- --nocapture
# 10 passed

CARGO_INCREMENTAL=0 cargo clippy -p kamn-core --test p2p_libp2p_native_adapter_runtime --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-core --test p2p_libp2p_native_adapter_runtime --all-features -- -D warnings
# passed

python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha HEAD --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/kamn-governance-ratio-current-head.json
# status=ok
# input_non_merge_commit_total=984
# non_merge_commit_total=50
# governance_commit_count=8
# feature_commit_count=42
# governance_ratio=0.16
# feature_ratio=0.84

cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance -- --nocapture
# 28 passed

cargo test -p kamn-core --test script_surface_reduction_candidates_docs -- --nocapture
# 1 passed

bash scripts/ci/check_touched_rust_size_policy.sh --base-ref main --threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json --baseline-file fixtures/ci/touched_rust_size_policy_baseline.json --output-json /tmp/kamn-touched-rust-size-policy-7035-p2p-libp2p-clippy.json
# status=pass
# policy_decision=GO
# offending_files=none
# offending_functions=none

CARGO_INCREMENTAL=0 cargo clippy -p kamn-core --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy -p kamn-core --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

CARGO_INCREMENTAL=0 rustup run stable cargo clippy --workspace --all-targets --all-features -- -D warnings
# passed

cargo test --workspace --locked --all-features --no-fail-fast
# passed

make check
# passed
```
