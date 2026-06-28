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
- Critical-path mutation may invoke stale test selectors after module splits,
  causing `cargo-mutants` slices to run zero tests and report escaped mutants
  even though the runtime contract still has current tests.
- Critical-path mutation may target a stale source line after signer module
  extraction, discovering zero mutants instead of proving the strict signer
  secret-source precedence contract.

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
- Test-file size policy inventory accounting is refreshed by exactly one added
  coverage-gate contract test, with oversized counts unchanged.
- Critical-path mutation runner test selectors match the current split module
  test names for group-channel crypto and service API replay rejection.
- Critical-path mutation runner signer selectors target the current signer
  secret-provider implementation instead of stale `signer.rs` line numbers.
- Critical-path mutation runner still expects 10 mutants across 6 slices and
  catches all selected mutants locally or in CI.
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
```
