# Node Configuration Layering

This document defines the deterministic configuration contracts for `kamn-node`.

## Scope

Phase 6.2 implementation adds:

- `--config-file <path>` support using a deterministic `key=value` format.
- Environment-variable overrides for selected node settings.
- Strict validation with fail-closed behavior on malformed config or invalid override values.

## Shell JSON Helper Contract (Issue #4879)

Shell scripts that emit JSON artifacts must route writes through the shared helper:

- `scripts/lib/write_json_file.sh`
- `write_json_file()` / `write_json_object()` from `scripts/lib/common.sh` when already sourced.

Operator-facing and CI contract scripts should not construct JSON files with direct `cat <<JSON` redirection.
The migration contract is validated by:

- `bash scripts/lib/test_json_write_helper_migration_contract.sh`

## Precedence

`kamn-node` resolves settings in this order (low to high):

1. Built-in defaults
2. Profile defaults (`--profile`)
3. Config-file entries (when `--config-file` or `KAMN_NODE_CONFIG_FILE` is set)
4. `KAMN_NODE_*` environment overrides
5. Explicit CLI flags

## Config File Format

- File format is line-oriented `key=value`.
- Empty lines and lines starting with `#` are ignored.
- Unknown keys fail closed.
- Boolean keys must use `true` or `false`.

Example:

```text
# node-runtime.conf
role=listener
chain_id=kamn-localnet
chain_version=v0.2.0
storage_dir=./data/listener
enable_gossip=false
sync_mode=archive
output=json
diagnostics=snapshot
```

## Supported Keys

Core keys:

- `profile`, `role`, `chain_id`, `chain_version`, `storage_dir`, `enable_gossip`, `sync_mode`
- `runtime_mode`, `expected_state_version`, `expected_state_hash`
- `proposal`, `rejoin_attempt`
- `output`, `diagnostics`

Runtime/API keys:

- `daemon_max_ticks`, `daemon_tick_interval_ms`, `daemon_shutdown_signal_tick`
- `daemon_shutdown_os_signals`, `daemon_shutdown_drain_ticks`, `daemon_shutdown_timeout_ticks`
- `daemon_peer_id`, `daemon_lifecycle_event`
- `api_bind`, `api_max_requests`, `api_idle_timeout_ms`
- `observability_endpoint_bind`, `observability_endpoint_metrics_path`
- `observability_endpoint_health_path`, `observability_endpoint_max_requests`
- `observability_endpoint_idle_timeout_ms`

Shutdown signal failure matrix:

- `SIGINT` and `SIGTERM` trigger graceful shutdown when `--daemon-shutdown-os-signals` is enabled.
- Timeout transitions must project
  `graceful-shutdown-timeout:signal@<tick>;drain_ticks=<n>;timeout_ticks=<n>;ignored_signals=<n>`.
- Graceful completion reasons fail closed when `drain_ticks > timeout_ticks` with
  `full_supervisor_stop_graceful_drain_timeout_contract_mismatch`.
- Invalid numeric shutdown metadata fails closed with:
  - `full_supervisor_stop_invalid_drain_ticks`
  - `full_supervisor_stop_invalid_timeout_ticks`
  - `full_supervisor_stop_invalid_ignored_signals`
- Shutdown checkpoint reconciliation drift fails closed under
  `runtime_shutdown_invariant_violation:<reason_code>`.
- Shutdown checkpoint reconciliation reason taxonomy marker:
  - `shutdown_checkpoint_reconciliation_reason_taxonomy_version=kamn.runtime.shutdown-checkpoint-reconciliation-reason-taxonomy.v1`
- Required reconciliation reason markers:
  - `shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch`
  - `shutdown_checkpoint_reconciliation_timeout_checkpoint_mismatch`
  - `shutdown_checkpoint_reconciliation_graceful_reason_code_mismatch`
  - `shutdown_checkpoint_reconciliation_graceful_checkpoint_mismatch`
  - `shutdown_checkpoint_reconciliation_not_signaled_reason_code_mismatch`
  - `shutdown_checkpoint_reconciliation_not_signaled_checkpoint_mismatch`
- Regression markers:
- `Regression: #4332`
- `Regression: #4333`
- See `docs/ops/runbooks/shutdown.md` for deterministic reason-shape and operator validation steps.

## Full-Stack Harness Marker Completeness and Parity Mismatch Controls (Issue #4195)

Full I/O scenario matrix policy checks fail closed when required harness markers drift or
dry-run parity contracts are violated.

Deterministic harness mismatch controls:

- `full_io_harness_marker_completeness_status=verified`
- `full_io_harness_marker_parity_status=verified`
- `full_io_harness_policy_reason_taxonomy_version=kamn.runtime.full-io-scenario-matrix-policy-reason-taxonomy.v1`
- `full_io_harness_policy_reason_codes_csv=full_io_scenario_matrix_policy_process_harness_mismatch,full_io_scenario_matrix_policy_api_route_matrix_mismatch,full_io_scenario_matrix_policy_auth_failure_matrix_mismatch,full_io_scenario_matrix_policy_websocket_matrix_mismatch,full_io_scenario_matrix_policy_multinode_propagation_mismatch,full_io_scenario_matrix_policy_dry_run_command_count_mismatch,full_io_scenario_matrix_policy_dry_run_command_status_mismatch`

Deterministic fail-closed mismatch reasons:

- `full_io_scenario_matrix_policy_process_harness_mismatch`
- `full_io_scenario_matrix_policy_api_route_matrix_mismatch`
- `full_io_scenario_matrix_policy_auth_failure_matrix_mismatch`
- `full_io_scenario_matrix_policy_websocket_matrix_mismatch`
- `full_io_scenario_matrix_policy_multinode_propagation_mismatch`
- `full_io_scenario_matrix_policy_dry_run_command_count_mismatch`
- `full_io_scenario_matrix_policy_dry_run_command_status_mismatch`

Validation commands:

- `bash scripts/runtime/test_check_full_io_scenario_matrix_live_policy.sh`
- `bash scripts/runtime/test_validate_full_io_scenario_matrix_live_contract_lane.sh`

Regression marker:

- `Regression: #4195`

Kolme-live keys:

- `kolme_live_base_url`, `kolme_live_provider_hint`, `kolme_live_signing_profile`
- `kolme_live_strict_signer_contracts`, `kolme_live_signer_profile`, `kolme_live_signer_key_source`

## Signer Material Validation and Fallback Prohibition Contracts (Issues #4167, #4168)

Kolme live signer configuration must remain explicit and fail closed for missing signer material,
invalid signer secret hex, and fallback secret reintroduction.

Deterministic signer-config policy markers:

- `signer_config_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-signer-config-reason-taxonomy.v1`
- `signer_config_reason_codes_csv=signer_secret_missing,signer_secret_invalid_hex,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing`
- `signer_config_reason_codes_value=none|<csv>`

Deterministic fail-closed signer-config reasons:

- `signer_secret_missing`
- `signer_secret_invalid_hex`
- `fallback_signer_secret_present_violation`
- `fallback_signer_secret_checkpoint_reason_mismatch`
- `fallback_signer_secret_remediation_missing`

Runtime signer key-source policy taxonomy markers:

- `runtime_signer_key_source_policy_reason_codes_csv=production_signer_key_source_env_local_forbidden,fallback_signer_secret_present_violation`
- `production_signer_key_source_env_local_forbidden`
- `fallback_signer_secret_present_violation`

Managed signer provenance taxonomy markers:

- `managed_signer_provenance_reason_codes_csv=managed_signer_backend_response_provenance_missing,managed_signer_backend_response_provenance_malformed,managed_signer_backend_response_provenance_mismatch`
- `managed_signer_backend_response_provenance_missing`
- `managed_signer_backend_response_provenance_malformed`
- `managed_signer_backend_response_provenance_mismatch`

Managed key-source adapter provenance mapping markers:

- `managed_key_source_adapter_provenance_status=verified`
- `managed_key_source_adapter_provenance_fields_csv=profile,key_source,key_reference_env,signer_public_key_hex`
- `managed_key_source_adapter_provenance_reason_codes_csv=managed_signer_provenance_marker_profile_mismatch,managed_signer_provenance_marker_key_source_mismatch,managed_signer_provenance_marker_key_reference_env_mismatch,managed_signer_provenance_marker_public_key_missing`
- `managed_signer_provenance_marker_profile_mismatch`
- `managed_signer_provenance_marker_key_source_mismatch`
- `managed_signer_provenance_marker_key_reference_env_mismatch`
- `managed_signer_provenance_marker_public_key_missing`

Deterministic operator-facing remediation/error expectations:

- missing signer material rejects with message `signer secret env is required for selected profile`.
- fallback signer secret rejects with message `fallback signer secret env must not be set` and remediation
  marker `remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`.
- runtime signer key-source policy rejects fallback signer secret env before signer preflight execution
  with remediation `unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`.

Validation commands:

- `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh`
- `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::regression_kolme_live_signer_key_source_policy_rejects_fallback_secret_path_with_deterministic_reason_code -- --exact --nocapture`
- `cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture`
- `cargo test -p kamn-node signer::managed_backend::tests::unit_managed_key_source_adapter_emits_deterministic_provenance_marker -- --exact`
- `cargo test -p kamn-node signer::tests::regression_managed_key_source_provenance_marker_profile_mismatch_fails_closed -- --exact`
- `python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json`

Regression markers:

- `Regression: #4167`
- `Regression: #4168`
- `Regression: #3955`

## Signer Secret Decode Buffer Zeroization Controls (Issues #4165, #4166)

Signer secret ingestion and parse paths must explicitly scrub transient buffers on strict-policy
rejections and parse completion paths.

Deterministic zeroization markers:

- `signer_secret_source_precedence_zeroization_status=verified`
- `signer_private_key_parse_zeroization_status=verified`
- `signer_transient_key_material_zeroization_status=verified`
- `signer_secret_source_precedence_violation`
- `managed_signer_private_key_adapter_unsupported`

Validation commands:

- `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact`
- `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact`
- `cargo test -p kamn-node regression_signer_secret_source_precedence_path_requires_zeroize_markers`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_signer_secret_zeroization_controls -- --exact`

Regression markers:

- `Regression: #4165`
- `Regression: #4166`

## Async API Backpressure Failure Modes (Issue #4315)

`kamn-node` async API ingress limits remain fail closed under bounded-concurrency pressure.

Deterministic taxonomy markers:

- `service_api_backpressure_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1`
- `service_api_lifecycle_rejection_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1`
- `service_api_lifecycle_rejection_reason_codes_csv=service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid`
- `async_lifecycle_backpressure_projection_status=verified`
- `admission_inflight_budget_status=verified`
- `admission_queue_budget_status=verified`
- `admission_inflight_budget_limit=32`
- `admission_queue_budget_limit=1`
- `admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1`
- `admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch`
- `admission_decision_taxonomy_status=verified`
- `admission_decision_accept_status=verified`
- `admission_decision_defer_status=verified`
- `admission_decision_reject_status=verified`
- `admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1`
- `admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject`
- `reason_codes_value=none|service_api_axum_policy_*`

Backpressure reason markers:

- `service_api_ingress_concurrency_limit_exceeded`
- `service_api_ingress_rate_limit_exceeded`
- `service_api_ingress_sender_rate_limit_exceeded`
- `service_api_axum_policy_admission_inflight_budget_limit_mismatch`
- `service_api_axum_policy_admission_queue_budget_limit_mismatch`
- `service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch`
- `service_api_axum_policy_admission_decision_reason_codes_csv_mismatch`

fail-closed response contract:

- backpressure limiter rejections emit `HTTP 429` with `error=too-many-requests`
- concurrency saturation maps to `outcome=concurrency-limit`
- ingress rate pressure maps to `outcome=rate-limit`
- sender admission anti-spam pressure maps to `outcome=anti-spam`

Validation commands:

- `cargo test -p kamn-node integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded -- --exact`
- `cargo test -p kamn-node regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds -- --exact`
- `cargo test -p kamn-node functional_service_api_endpoint_backpressure_projection_covers_reason_codes -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_async_backpressure_failure_modes -- --exact`
- `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `bash scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`

Regression marker:

- `Regression: #4315`

## Service API Request-Path Authz Matrix and Docs Parity Contract (Issue #4057)

Deterministic route-level request authorization markers:

- `service_api_request_path_authz_reason_taxonomy_version=kamn.runtime.service-api-auth-reason-taxonomy.v1`
- `service_api_request_path_authz_reason_codes_csv=service_api_auth_sender_did_header_missing,service_api_auth_sender_did_invalid,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected`
- `service_api_request_path_authz_public_routes_csv=GET:/healthz,GET:/metrics`
- `service_api_request_path_authz_protected_routes_csv=POST:/v1/messages/send,POST:/v1/channels/create,POST:/v1/tasks/create,GET:/v1/messages/{message_id},GET:/v1/channels/{channel_id}/messages,GET:/v1/tasks/{task_id},GET:/v1/agents/{agent_did},GET:/v1/events/ws`
- `service_api_request_path_authz_missing_header_reason_code=service_api_auth_sender_did_header_missing`

Deterministic remediation markers:

- `service_api_request_path_authz_remediation_map_version=v1`
- `service_api_request_path_authz_remediation.service_api_auth_sender_did_header_missing=add x-kamn-sender-did header with a valid kamn DID before calling protected routes`
- `service_api_request_path_authz_remediation.service_api_auth_sender_did_invalid=fix sender DID to kamn:did:<scope>:<id> format`
- `service_api_request_path_authz_remediation.service_api_auth_nonce_header_missing=add x-kamn-request-nonce header with a positive integer`
- `service_api_request_path_authz_remediation.service_api_auth_nonce_invalid=use a base-10 u64 nonce value in x-kamn-request-nonce`
- `service_api_request_path_authz_remediation.service_api_auth_nonce_non_positive=increment nonce to a value greater than zero`
- `service_api_request_path_authz_remediation.service_api_auth_signature_header_missing=add x-kamn-request-signature over sender_did+nonce+state_hash+body`
- `service_api_request_path_authz_remediation.service_api_auth_signature_verification_failed=recompute signature with the supported profile and current state hash`
- `service_api_request_path_authz_remediation.service_api_auth_replay_nonce_detected=use a fresh nonce per sender DID and avoid replaying accepted envelopes`

Validation commands:

- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::unit_service_api_route_authz_matrix_matches_protected_and_public_paths -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_request_path_authz_docs_parity_matches_source_taxonomy -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_request_path_authz_remediation_markers_cover_reason_codes -- --exact`

Regression marker:

- `Regression: #4057`

## Service API Scope Policy Checker Contract (Issue #4056)

Deterministic scope-policy markers:

- `service_api_scope_policy_reason_taxonomy_version=kamn.runtime.service-api-scope-policy-reason-taxonomy.v1`
- `service_api_scope_policy_reason_codes_csv=service_api_auth_scope_header_missing,service_api_auth_scope_invalid,service_api_auth_scope_route_mismatch`
- `service_api_scope_policy_fixture_schema_version=kamn.runtime.service-api-scope-policy-fixture-matrix.v1`
- `service_api_scope_policy_fixture_path=fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`

Deterministic remediation markers:

- `service_api_scope_policy_remediation_map_version=v1`
- `service_api_scope_policy_remediation.service_api_auth_scope_header_missing=add x-kamn-authz-scope with the route-required scope value`
- `service_api_scope_policy_remediation.service_api_auth_scope_invalid=use one of messages:write|messages:read|channels:write|channels:read|tasks:write|tasks:read|agents:read|events:read|protected:unknown`
- `service_api_scope_policy_remediation.service_api_auth_scope_route_mismatch=align x-kamn-authz-scope to the required scope for method/path`

Validation commands:

- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::unit_service_api_scope_policy_fixture_parser_contract -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_scope_policy_docs_parity_matches_source_taxonomy -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_scope_policy_remediation_markers_cover_reason_codes -- --exact`

Regression marker:

- `Regression: #4056`

## Service API Tenant-Isolation Matrix Contract (Issue #4058)

Deterministic tenant-isolation matrix markers:

- `service_api_tenant_isolation_matrix_reason_taxonomy_version=kamn.runtime.service-api-tenant-isolation-matrix-policy-reason-taxonomy.v1`
- `service_api_tenant_isolation_matrix_reason_codes_csv=ci_fast_gate_failed,service_api_tenant_isolation_policy_schema_mismatch,service_api_tenant_isolation_policy_status_invalid,service_api_tenant_isolation_policy_final_decision_invalid,service_api_tenant_isolation_policy_final_decision_mismatch,service_api_tenant_isolation_policy_lane_mode_invalid,service_api_tenant_isolation_policy_matrix_schema_mismatch,service_api_tenant_isolation_policy_matrix_rows_invalid,service_api_tenant_isolation_policy_matrix_row_count_mismatch,service_api_tenant_isolation_policy_matrix_row_duplicate,service_api_tenant_isolation_policy_matrix_row_id_invalid,service_api_tenant_isolation_policy_matrix_row_missing,service_api_tenant_isolation_policy_matrix_row_status_mismatch,service_api_tenant_isolation_policy_matrix_row_leakage_result_mismatch,service_api_tenant_isolation_policy_matrix_row_reason_code_mismatch,service_api_tenant_isolation_policy_matrix_row_selector_mismatch,service_api_tenant_isolation_policy_marker_missing,service_api_tenant_isolation_policy_execution_reason_code_mismatch,service_api_tenant_isolation_policy_command_count_invalid,service_api_tenant_isolation_policy_command_count_mismatch,service_api_tenant_isolation_policy_elapsed_seconds_invalid,service_api_tenant_isolation_policy_max_seconds_invalid,service_api_tenant_isolation_policy_runtime_budget_exceeded,service_api_tenant_isolation_policy_docs_marker_missing`
- `service_api_tenant_isolation_matrix_matrix_schema_version=kamn.runtime.service-api-tenant-isolation-matrix.v1`
- `service_api_tenant_isolation_matrix_required_row_ids_csv=m2_abac_cross_tenant_visibility_denied,m8_cross_owner_retention_and_shred_denied,m9_cross_owner_dispatch_and_presence_denied,m9_gateway_cross_owner_presence_denied`
- `service_api_tenant_isolation_matrix_strategy_doc_path=docs/ci/strategy.md`
- `service_api_tenant_isolation_matrix_ops_doc_path=docs/ops/configuration.md`

Deterministic leakage rejection reason markers:

- `m2_abac_scope_denied`
- `m8_compliance_owner_scope_denied`
- `m9_realtime_owner_scope_denied`
- `service_api_tenant_isolation_policy_matrix_row_status_mismatch`

Validation commands:

- `bash scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh --mode dry-run --output-json /tmp/service-api-tenant-isolation-matrix-live-summary.json`
- `KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_OPT_IN=1 bash scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh --mode run --output-json /tmp/service-api-tenant-isolation-matrix-live-summary.json`
- `bash scripts/runtime/check_service_api_tenant_isolation_matrix_live_policy.sh --report-file /tmp/service-api-tenant-isolation-matrix-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-tenant-isolation-matrix-policy.json`
- `bash scripts/runtime/validate_service_api_tenant_isolation_matrix_live_contract_lane.sh --output-json /tmp/service-api-tenant-isolation-matrix-contract-lane-report.json --policy-output-json /tmp/service-api-tenant-isolation-matrix-policy.json`
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract integration_tenant_isolation_matrix_contract_lane_composes_lane_policy_and_docs_parity -- --exact`
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract regression_tenant_isolation_matrix_policy_rejects_tampered_leakage_marker -- --exact`

Regression marker:

- `Regression: #4058`

## API Version-Policy Contract (Issue #4041)

Deterministic API version-policy markers:

- `api_version_policy_reason_taxonomy_version=kamn.runtime.api-version-policy-reason-taxonomy.v1`
- `api_version_policy_reason_codes_csv=ci_fast_gate_failed,api_version_policy_schema_mismatch,api_version_policy_status_invalid,api_version_policy_final_decision_invalid,api_version_policy_final_decision_mismatch,api_version_policy_lane_mode_invalid,api_version_policy_fixture_schema_mismatch,api_version_policy_fixture_rows_invalid,api_version_policy_fixture_row_count_mismatch,api_version_policy_fixture_row_duplicate,api_version_policy_fixture_row_id_invalid,api_version_policy_fixture_row_missing,api_version_policy_fixture_row_status_mismatch,api_version_policy_fixture_row_decision_mismatch,api_version_policy_fixture_row_reason_code_mismatch,api_version_policy_fixture_row_version_mismatch,api_version_policy_fixture_row_window_mismatch,api_version_policy_marker_missing,api_version_policy_execution_reason_code_mismatch,api_version_policy_command_count_invalid,api_version_policy_command_count_mismatch,api_version_policy_elapsed_seconds_invalid,api_version_policy_max_seconds_invalid,api_version_policy_runtime_budget_exceeded,api_version_policy_docs_marker_missing`
- `api_version_policy_fixture_schema_version=kamn.runtime.api-version-policy-fixture-matrix.v1`
- `api_version_policy_fixture_path=fixtures/runtime/api_version_policy_fixture_matrix.txt`
- `api_version_policy_required_row_ids_csv=v1_messages_send,v2_channels_create,v0_messages_send,v3_future_route`
- `api_version_policy_strategy_doc_path=docs/ci/strategy.md`
- `api_version_policy_ops_doc_path=docs/ops/configuration.md`

Deterministic fail-closed reason markers:

- `api_version_unsupported_window`
- `api_version_policy_fixture_row_status_mismatch`

Validation commands:

- `bash scripts/runtime/validate_api_version_policy_live.sh --mode dry-run --output-json /tmp/api-version-policy-live-summary.json`
- `KAMN_API_VERSION_POLICY_OPT_IN=1 bash scripts/runtime/validate_api_version_policy_live.sh --mode run --output-json /tmp/api-version-policy-live-summary.json`
- `bash scripts/runtime/check_api_version_policy_live_policy.sh --report-file /tmp/api-version-policy-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/api-version-policy-live-policy.json`
- `bash scripts/runtime/validate_api_version_policy_live_contract_lane.sh --output-json /tmp/api-version-policy-contract-lane-report.json --policy-output-json /tmp/api-version-policy-live-policy.json`
- `cargo test -p kamn-core --test api_version_policy_contract functional_api_version_policy_checker_accepts_valid_report -- --exact`
- `cargo test -p kamn-core --test api_version_policy_contract regression_api_version_policy_checker_rejects_tampered_marker -- --exact`

Regression marker:

- `Regression: #4041`

## Request-Response Schema Compatibility Contract (Issue #4042)

Deterministic request-response schema compatibility markers:

- `request_response_schema_compatibility_reason_taxonomy_version=kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1`
- `request_response_schema_compatibility_reason_codes_csv=ci_fast_gate_failed,request_response_schema_compatibility_schema_mismatch,request_response_schema_compatibility_status_invalid,request_response_schema_compatibility_final_decision_invalid,request_response_schema_compatibility_final_decision_mismatch,request_response_schema_compatibility_lane_mode_invalid,request_response_schema_compatibility_fixture_schema_mismatch,request_response_schema_compatibility_fixture_rows_invalid,request_response_schema_compatibility_fixture_row_count_mismatch,request_response_schema_compatibility_fixture_row_duplicate,request_response_schema_compatibility_fixture_row_id_invalid,request_response_schema_compatibility_fixture_row_missing,request_response_schema_compatibility_fixture_row_status_mismatch,request_response_schema_compatibility_fixture_row_decision_mismatch,request_response_schema_compatibility_fixture_row_reason_code_mismatch,request_response_schema_compatibility_fixture_row_version_pair_mismatch,request_response_schema_compatibility_fixture_row_change_class_mismatch,request_response_schema_compatibility_marker_missing,request_response_schema_compatibility_execution_reason_code_mismatch,request_response_schema_compatibility_command_count_invalid,request_response_schema_compatibility_command_count_mismatch,request_response_schema_compatibility_elapsed_seconds_invalid,request_response_schema_compatibility_max_seconds_invalid,request_response_schema_compatibility_runtime_budget_exceeded,request_response_schema_compatibility_docs_marker_missing`
- `request_response_schema_compatibility_fixture_schema_version=kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1`
- `request_response_schema_compatibility_fixture_path=fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt`
- `request_response_schema_compatibility_required_row_ids_csv=v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_messages_get_required_response_removal,v1_to_v2_tasks_create_required_request_removal`
- `request_response_schema_compatibility_strategy_doc_path=docs/ci/strategy.md`
- `request_response_schema_compatibility_ops_doc_path=docs/ops/configuration.md`

Deterministic fail-closed reason markers:

- `schema_pair_breaking_change_detected`
- `request_response_schema_compatibility_fixture_row_status_mismatch`

Validation commands:

- `bash scripts/runtime/validate_request_response_schema_compatibility_live.sh --mode dry-run --output-json /tmp/request-response-schema-compatibility-live-summary.json`
- `KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_OPT_IN=1 bash scripts/runtime/validate_request_response_schema_compatibility_live.sh --mode run --output-json /tmp/request-response-schema-compatibility-live-summary.json`
- `bash scripts/runtime/check_request_response_schema_compatibility_live_policy.sh --report-file /tmp/request-response-schema-compatibility-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/request-response-schema-compatibility-live-policy.json`
- `bash scripts/runtime/validate_request_response_schema_compatibility_live_contract_lane.sh --output-json /tmp/request-response-schema-compatibility-contract-lane-report.json --policy-output-json /tmp/request-response-schema-compatibility-live-policy.json`
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract functional_request_response_schema_compatibility_checker_accepts_valid_report -- --exact`
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract regression_request_response_schema_compatibility_checker_rejects_tampered_marker -- --exact`

Regression marker:

- `Regression: #4042`

## API Compatibility Matrix Local-Heavy Contract (Issue #4043)

Deterministic local-heavy compatibility matrix markers:

- `api_compatibility_matrix_local_heavy_reason_taxonomy_version=kamn.runtime.api-compatibility-matrix-local-heavy-policy-reason-taxonomy.v1`
- `api_compatibility_matrix_local_heavy_reason_codes_csv=ci_fast_gate_failed,api_compatibility_matrix_local_heavy_policy_schema_mismatch,api_compatibility_matrix_local_heavy_policy_status_invalid,api_compatibility_matrix_local_heavy_policy_final_decision_invalid,api_compatibility_matrix_local_heavy_policy_final_decision_mismatch,api_compatibility_matrix_local_heavy_policy_lane_mode_invalid,api_compatibility_matrix_local_heavy_policy_artifact_schema_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_schema_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid,api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_duplicate,api_compatibility_matrix_local_heavy_policy_fixture_row_id_invalid,api_compatibility_matrix_local_heavy_policy_fixture_row_missing,api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_decision_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_reason_code_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_version_pair_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_route_selector_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_change_class_mismatch,api_compatibility_matrix_local_heavy_policy_marker_missing,api_compatibility_matrix_local_heavy_policy_execution_reason_code_mismatch,api_compatibility_matrix_local_heavy_policy_command_count_invalid,api_compatibility_matrix_local_heavy_policy_command_count_mismatch,api_compatibility_matrix_local_heavy_policy_elapsed_seconds_invalid,api_compatibility_matrix_local_heavy_policy_max_seconds_invalid,api_compatibility_matrix_local_heavy_policy_runtime_budget_exceeded,api_compatibility_matrix_local_heavy_policy_local_heavy_opt_in_required,api_compatibility_matrix_local_heavy_policy_local_heavy_scope_mismatch,api_compatibility_matrix_local_heavy_policy_docs_marker_missing`
- `api_compatibility_matrix_local_heavy_fixture_schema_version=kamn.runtime.api-compatibility-matrix-local-heavy-fixture-matrix.v1`
- `api_compatibility_matrix_local_heavy_fixture_path=fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt`
- `api_compatibility_matrix_local_heavy_required_row_ids_csv=v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_tasks_create_required_request_removal,v1_to_v2_messages_get_required_response_removal,v1_to_v2_messages_send_enum_variant_removal`
- `api_compatibility_matrix_local_heavy_strategy_doc_path=docs/ci/strategy.md`
- `api_compatibility_matrix_local_heavy_ops_doc_path=docs/ops/configuration.md`

Deterministic incompatibility and fail-closed markers:

- `incompatible_request_breaking_change`
- `incompatible_response_breaking_change`
- `incompatible_enum_breaking_change`
- `api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch`

Validation commands:

- `bash scripts/runtime/validate_api_compatibility_matrix_local_heavy_live.sh --mode dry-run --ci-fast-gate PASS --output-json /tmp/api-compatibility-matrix-local-heavy-summary.json`
- `KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_OPT_IN=1 bash scripts/runtime/validate_api_compatibility_matrix_local_heavy_live.sh --mode run --ci-fast-gate FAIL --output-json /tmp/api-compatibility-matrix-local-heavy-summary.json`
- `bash scripts/runtime/check_api_compatibility_matrix_local_heavy_live_policy.sh --report-file /tmp/api-compatibility-matrix-local-heavy-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/api-compatibility-matrix-local-heavy-policy.json`
- `bash scripts/runtime/validate_api_compatibility_matrix_local_heavy_live_contract_lane.sh --output-json /tmp/api-compatibility-matrix-local-heavy-contract-lane-report.json --policy-output-json /tmp/api-compatibility-matrix-local-heavy-policy.json`
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract functional_api_compatibility_matrix_local_heavy_policy_accepts_valid_report -- --exact`
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract regression_api_compatibility_matrix_local_heavy_policy_rejects_tampered_matrix_marker -- --exact`

Regression marker:

- `Regression: #4043`

## API Compatibility CI Dry-Run Governance Contract (Issue #4044)

Deterministic checker markers:

- `compatibility_ci_dry_run_reason_taxonomy_version=kamn.ci.api-compatibility-ci-dry-run-governance-reason-taxonomy.v1`
- `compatibility_ci_dry_run_reason_codes_csv=compatibility_ci_dry_run_argument_invalid,compatibility_ci_dry_run_threshold_contract_violation,compatibility_ci_dry_run_report_contract_violation,compatibility_ci_dry_run_runtime_budget_exceeded,compatibility_ci_dry_run_fast_mode_selector_drift,compatibility_ci_dry_run_workflow_exclusion_drift,compatibility_ci_dry_run_docs_marker_parity_drift,compatibility_ci_dry_run_docs_remediation_marker_missing`
- `compatibility_ci_dry_run_threshold_fixture_path=fixtures/ci/api_compatibility_ci_dry_run_governance_thresholds.env`
- `compatibility_ci_dry_run_max_seconds=120`
- `compatibility_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract -- --nocapture`
- `compatibility_ci_dry_run_fast_mode_forbidden_entry=bash "$ROOT_DIR/scripts/runtime/validate_api_compatibility_matrix_local_heavy_live.sh" --mode run`
- `compatibility_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/validate_api_compatibility_matrix_local_heavy_live.sh --mode run`
- `compatibility_ci_dry_run_remediation_map_version=v1`

Validation commands:

- `bash scripts/runtime/validate_api_version_policy_live.sh --mode dry-run --output-json /tmp/api-version-policy-live-summary.json`
- `bash scripts/runtime/validate_request_response_schema_compatibility_live.sh --mode dry-run --output-json /tmp/request-response-schema-compatibility-live-summary.json`
- `bash scripts/runtime/validate_api_compatibility_matrix_local_heavy_live.sh --mode dry-run --ci-fast-gate PASS --output-json /tmp/api-compatibility-matrix-local-heavy-summary.json`
- `python3 scripts/ci/check_api_compatibility_ci_dry_run_governance.py --api-version-policy-report-file /tmp/api-version-policy-live-summary.json --request-response-schema-compatibility-report-file /tmp/request-response-schema-compatibility-live-summary.json --api-compatibility-matrix-local-heavy-report-file /tmp/api-compatibility-matrix-local-heavy-summary.json --threshold-file fixtures/ci/api_compatibility_ci_dry_run_governance_thresholds.env --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --output-json /tmp/api-compatibility-ci-dry-run-governance-report.json`
- `cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract -- --nocapture`

Deterministic remediation markers:

- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_argument_invalid=fix checker invocation flags and required file arguments`
- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_threshold_contract_violation=restore required keys/values in fixtures/ci/api_compatibility_ci_dry_run_governance_thresholds.env`
- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_report_contract_violation=regenerate dry-run compatibility reports and restore schema/taxonomy markers`
- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_runtime_budget_exceeded=reduce compatibility checker/report overhead or adjust threshold fixture with explicit review evidence`
- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_fast_mode_selector_drift=restore required ci-tools fast-mode checker entry and remove local-heavy run leakage`
- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_workflow_exclusion_drift=remove local-heavy run-mode command from .github/workflows/ci-fast-gate.yml`
- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_docs_marker_parity_drift=realign docs/ci/strategy.md and docs/ops/configuration.md compatibility marker blocks with fixture/checker contracts`
- `compatibility_ci_dry_run_remediation.compatibility_ci_dry_run_docs_remediation_marker_missing=add missing compatibility_ci_dry_run_remediation.<reason>= marker entries to strategy and ops docs`

Regression marker:

- `Regression: #4044`

## Realtime Presence Mode Gateway and Guardrail Contracts (Issues #5279, #5281, #5283)

Phase-5 realtime gateway integration requires deterministic presence-mode websocket contracts and
bounded guardrail validation for backpressure/anti-spam/replay safety paths.

Deterministic presence-mode markers:

- `service_api_ws_presence_mode_status=verified`
- `service_api_ws_events_mode_header=x-kamn-events-mode`
- `service_api_ws_events_mode_presence_value=presence`
- `service_api_ws_presence_required_headers_csv=x-kamn-presence-owner-did,x-kamn-presence-target-agent-did,x-kamn-requester-agent-did`
- `service_api_ws_presence_optional_headers_csv=x-kamn-presence-target-owner-did,x-kamn-presence-gateway-node,x-kamn-presence-connected-since,x-kamn-presence-last-heartbeat,x-kamn-presence-capabilities`
- `service_api_ws_presence_fail_closed_reason_codes_csv=service_api_ws_events_mode_invalid,service_api_ws_presence_owner_did_header_missing,service_api_ws_presence_target_agent_did_header_missing,service_api_ws_presence_requester_agent_did_header_missing,m9_realtime_owner_scope_denied,m9_realtime_presence_visibility_denied`
- `service_api_ws_presence_event_type=m9.presence.snapshot`
- `service_api_ws_presence_transport_profile=websocket`

Realtime guardrail validation markers:

- `realtime_guardrail_burst_validation_status=verified`
- `replay_duplicate_reason_ordering_status=verified`

Validation commands:

- `cargo test -p kamn-node integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event -- --exact`
- `cargo test -p kamn-node integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic -- --exact`
- `cargo test -p kamn-node integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts -- --exact`
- `cargo test -p kamn-node regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_realtime_presence_mode_and_guardrail_markers -- --exact`

Regression marker:

- `Regression: #5283`

## Phase-6 Archival Failure-Retry Policy Contracts (Issues #5285, #5287)

Phase-6 retention/archival execution requires deterministic recovery projection when archival export
attempts fail.

Deterministic retry-policy markers:

- `archival_retry_policy_status=verified`
- `archival_retry_reason_taxonomy_version=kamn.runtime.data-layer-m10-archival-retry-reason-taxonomy.v1`
- `archival_retry_reason_codes_csv=m10_archival_retry_scheduled,m10_archival_retry_exhausted,m10_archival_failure_permanent,m10_archival_retry_policy_invalid,m10_archival_retry_attempt_invalid`
- `archival_retry_policy_contract=max_attempts>=1;base_backoff_seconds>=1;max_backoff_seconds>=base_backoff_seconds`

Deterministic fail-closed and recoverable branches:

- `m10_archival_retry_scheduled`
- `m10_archival_retry_exhausted`
- `m10_archival_failure_permanent`
- `m10_archival_retry_policy_invalid`
- `m10_archival_retry_attempt_invalid`

Validation commands:

- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c12_transient_archival_failure_projects_deterministic_retry_window -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c15_archival_retry_policy_and_attempt_validation_fail_closed -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_phase6_archival_retry_policy_markers -- --exact`

Regression marker:

- `Regression: #5287`

## Phase-6 Retention+Archival Execution Tick Contracts (Issue #5289)

Phase-6 compliance automation now includes a single deterministic execution-tick boundary composing
M8 retention due lookup, crypto-shredding execution, M10 shred-completeness projection, and M10
archival due evaluation.

Deterministic orchestration markers:

- `phase6_execution_tick_status=verified`
- `phase6_execution_tick_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-execution-reason-taxonomy.v1`
- `phase6_execution_tick_reason_codes_csv=m10_phase6_execution_applied,m10_phase6_execution_owner_scope_denied,m10_phase6_execution_legal_hold_active,m10_phase6_execution_input_invalid,m10_phase6_execution_projection_input_invalid,m10_phase6_execution_projection_failed`
- `phase6_execution_tick_contract=retention_due_lookup->crypto_shred->partition_projection->archive_due`

Validation commands:

- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c17_phase6_orchestration_tick_orders_outputs_deterministically -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_phase6_execution_tick_orchestration_markers -- --exact`

Regression marker:

- `Regression: #5289`

## Phase-6 Execution Tick Budget Guardrail Contracts (Issue #5291)

Phase-6 orchestration tick output must be bounded by deterministic per-tick workload ceilings
before scheduler wiring is promoted.

Deterministic budget markers:

- `phase6_execution_tick_budget_status=verified`
- `phase6_execution_tick_budget_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-execution-budget-reason-taxonomy.v1`
- `phase6_execution_tick_budget_reason_codes_csv=m10_phase6_execution_budget_within_limit,m10_phase6_execution_budget_due_candidates_exceeded,m10_phase6_execution_budget_shredded_messages_exceeded,m10_phase6_execution_budget_projections_exceeded,m10_phase6_execution_budget_archive_entries_exceeded,m10_phase6_execution_budget_invalid`
- `phase6_execution_tick_budget_contract=due_candidates<=max_due_candidates;shredded_messages<=max_shredded_messages;projection_reports<=max_projection_reports;archived_entries<=max_archived_entries`

Validation commands:

- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_phase6_execution_tick_budget_markers -- --exact`

Regression marker:

- `Regression: #5291`

## Phase-6 Scheduler Cycle Trigger and Guarded Execution Contracts (Issue #5293)

Phase-6 runtime wiring now has a deterministic scheduler-cycle boundary that composes trigger
evaluation, preflight budget admission, execution tick orchestration, and budget evidence emission.

Deterministic scheduler markers:

- `phase6_scheduler_cycle_status=verified`
- `phase6_scheduler_trigger_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-trigger-reason-taxonomy.v1`
- `phase6_scheduler_trigger_reason_codes_csv=m10_phase6_scheduler_trigger_deferred,m10_phase6_scheduler_trigger_due_threshold,m10_phase6_scheduler_trigger_interval_elapsed`
- `phase6_scheduler_cycle_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-cycle-reason-taxonomy.v1`
- `phase6_scheduler_cycle_reason_codes_csv=m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_policy_invalid,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded,m10_phase6_execution_budget_shredded_messages_exceeded,m10_phase6_execution_budget_projections_exceeded,m10_phase6_execution_budget_archive_entries_exceeded`
- `phase6_scheduler_cycle_contract=trigger_decision->preflight_budget_admission->phase6_execution_tick->budget_evidence`

Validation commands:

- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_phase6_scheduler_cycle_markers -- --exact`

Regression marker:

- `Regression: #5293`

## Phase-6 Stateful Scheduler Runtime Checkpoint Contracts (Issue #5295)

Phase-6 scheduler execution now includes a stateful runtime contract that tracks deterministic
cycle counters, checkpoint continuity, and monotonic scheduler clock behavior across repeated
cycles.

Deterministic scheduler runtime markers:

- `phase6_scheduler_runtime_checkpoint_status=verified`
- `phase6_scheduler_runtime_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-runtime-reason-taxonomy.v1`
- `phase6_scheduler_runtime_reason_codes_csv=m10_phase6_scheduler_runtime_initialized,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded`
- `phase6_scheduler_runtime_state_contract=total_cycles=executed_cycles+deferred_cycles+fail_closed_cycles;last_successful_tick_epoch_seconds_updates_on_applied_only;last_observed_now_epoch_seconds_monotonic_non_decreasing`

Validation commands:

- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_phase6_scheduler_runtime_checkpoint_markers -- --exact`

Regression marker:

- `Regression: #5295`

## Phase-6 Runtime Evidence Bundle Projection Contracts (Issue #5297)

Phase-6 scheduler execution now projects a canonical runtime evidence bundle combining one
scheduler-cycle report with the persisted runtime checkpoint state.

Deterministic runtime evidence markers:

- `phase6_runtime_evidence_bundle_status=verified`
- `phase6_runtime_evidence_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-runtime-evidence-reason-taxonomy.v1`
- `phase6_runtime_evidence_reason_codes_csv=m10_phase6_runtime_evidence_applied,m10_phase6_runtime_evidence_deferred,m10_phase6_runtime_evidence_input_invalid`
- `phase6_runtime_evidence_bundle_contract=cycle_report+runtime_state->canonical_evidence_bundle;applied_requires_execution_and_budget_payload;deferred_requires_empty_execution_payload`

Validation commands:

- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete -- --exact`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_phase6_runtime_evidence_bundle_markers -- --exact`

Regression marker:

- `Regression: #5297`

## Phase-6 Daemon Runtime Integration Contracts (Issue #5299)

Daemon runtime orchestration now executes a deterministic Phase-6 scheduler runtime path and emits
structured completion markers for applied/deferred/fail-closed reason classes.

Deterministic daemon Phase-6 runtime markers:

- `phase6_daemon_runtime_contract_status=verified`
- `phase6_daemon_runtime_reason_taxonomy_version=kamn.runtime.daemon.phase6.reason-taxonomy.v1`
- `phase6_daemon_runtime_reason_codes_csv=m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded`
- `phase6_daemon_runtime_contract=daemon_tick_executes_m10_scheduler_runtime;report_projects_phase6_reason_and_counters;clock_regression_fails_closed`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_deferred_runtime_markers_when_shutdown_signals_are_present -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_daemon_phase6_runtime_projection_fail_closed_reason_is_stable_on_clock_regression -- --exact`

Regression marker:

- `Regression: #5299`

## PostgreSQL Live Integration + Daemon Runtime Validation Slice (Issue #5338)

This env-gated slice validates that a configured live PostgreSQL adapter path and daemon Phase-6
runtime projection markers can be exercised in one deterministic integration lane.

Deterministic live-postgres daemon slice markers:

- `phase6_live_postgres_daemon_runtime_slice_status=verified`
- `phase6_live_postgres_daemon_runtime_slice_env_gate=KAMN_TEST_POSTGRES_URL|DATABASE_URL`
- `phase6_live_postgres_daemon_runtime_slice_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres.reason-taxonomy.v1`
- `phase6_live_postgres_daemon_runtime_slice_reason_codes_csv=live_postgres_env_unset,live_postgres_adapter_connected,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred`
- `phase6_live_postgres_daemon_runtime_slice_contract=live_postgres_env_gate->adapter_connect_and_migrate->daemon_phase6_runtime_projection`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice -- --exact`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter spec_c01_and_c03_live_adapter_executes_insert_and_lookup_with_session_context -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_validation_slice_markers -- --exact`

Regression marker:

- `Regression: #5338`

### Gate and Deferred Path Hardening (Issue #5340)

The live-postgres daemon validation slice additionally enforces deterministic env-gate reason
resolution and an explicit deferred-path validation lane when daemon shutdown signals are active.

Deterministic gate/deferred markers:

- `phase6_live_postgres_daemon_runtime_gate_reason_contract=env_unset->skip_with_reason;env_set->adapter_connect_and_migrate`
- `phase6_live_postgres_daemon_runtime_deferred_contract=live_postgres_adapter_connected+shutdown_signal->m10_phase6_scheduler_cycle_deferred`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_gate_and_deferred_markers -- --exact`

Regression marker:

- `Regression: #5340`

### Scenario Matrix Stability (Issue #5342)

The live-postgres daemon validation slice now includes a deterministic scenario matrix and repeated
run stability checks to ensure reason-code projections remain stable for env-unset, applied, and
deferred paths.

Deterministic matrix/stability markers:

- `phase6_live_postgres_daemon_runtime_matrix_contract=env_unset->live_postgres_env_unset;env_set_no_shutdown->m10_phase6_scheduler_cycle_applied;env_set_shutdown->m10_phase6_scheduler_cycle_deferred`
- `phase6_live_postgres_daemon_runtime_stability_contract=repeated_runs_preserve_reason_code_per_matrix_scenario`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_stability_markers -- --exact`

Regression marker:

- `Regression: #5342`

### Matrix Taxonomy and Canonical Ordering (Issue #5344)

The live-postgres matrix contracts now include explicit reason-taxonomy versioning and canonical
scenario ordering markers to prevent drift between matrix semantics and docs.

Deterministic taxonomy/ordering markers:

- `phase6_live_postgres_daemon_runtime_matrix_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1`
- `phase6_live_postgres_daemon_runtime_matrix_reason_codes_csv=live_postgres_env_unset,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred`
- `phase6_live_postgres_daemon_runtime_matrix_scenarios_csv=env_unset,env_set_no_shutdown,env_set_shutdown`
- `phase6_live_postgres_daemon_runtime_matrix_order_contract=matrix_rows_order=env_unset->env_set_no_shutdown->env_set_shutdown;reason_codes_align_with_scenarios`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_ordering_markers -- --exact`

Regression marker:

- `Regression: #5344`

### Runtime-to-Matrix Taxonomy Bridge (Issue #5346)

The live-postgres validation matrix now explicitly bridges daemon runtime taxonomy markers to
matrix taxonomy semantics so applied/deferred runtime reason outputs remain taxonomy-consistent.

Deterministic taxonomy-bridge markers:

- `phase6_live_postgres_daemon_runtime_reason_taxonomy_version=kamn.runtime.daemon.phase6.reason-taxonomy.v1`
- `phase6_live_postgres_daemon_runtime_matrix_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1`
- `phase6_live_postgres_daemon_runtime_taxonomy_bridge_contract=runtime_reason_taxonomy_v1->matrix_scenario_taxonomy_v1;applied_and_deferred_reasons_must_align`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_taxonomy_bridge_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_taxonomy_versions_are_stable_across_repeated_runs -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_bridge_markers -- --exact`

Regression marker:

- `Regression: #5346`

### Bounded Load-Profile Matrix (Issue #5348)

The live-postgres matrix slice now enforces deterministic bounded load-profile contracts so
reason/taxonomy projections remain stable across applied and deferred runtime profiles.

Deterministic load-profile markers:

- `phase6_live_postgres_daemon_runtime_matrix_load_profile_ids_csv=applied_t3_i10,applied_t5_i25,applied_t9_i40,deferred_t5_i25_s3_d2_to4,deferred_t7_i25_s3_d2_to4,deferred_t9_i40_s3_d2_to4`
- `phase6_live_postgres_daemon_runtime_matrix_load_profile_contract=applied_profiles->m10_phase6_scheduler_cycle_applied;deferred_profiles->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_load_profile_markers -- --exact`

Regression marker:

- `Regression: #5348`

### Role-Profile Matrix Determinism (Issue #5350)

The live-postgres matrix contracts now enforce deterministic role-profile behavior across
`processor`, `listener`, and `approver` applied/deferred runtime variants.

Deterministic role-profile markers:

- `phase6_live_postgres_daemon_runtime_matrix_role_profile_ids_csv=processor_applied,processor_deferred,listener_applied,listener_deferred,approver_applied,approver_deferred`
- `phase6_live_postgres_daemon_runtime_matrix_role_profile_contract=processor|listener|approver_applied->m10_phase6_scheduler_cycle_applied;processor|listener|approver_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_profile_matrix_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_profile_markers -- --exact`

Regression marker:

- `Regression: #5350`

### Two-Node Role-Pair Matrix (Issue #5352)

The live-postgres matrix contracts now enforce deterministic ordered two-node role-pair behavior as
a bounded distributed-lane precursor across processor/listener/approver handoff-style runs.

Deterministic role-pair markers:

- `phase6_live_postgres_daemon_runtime_matrix_role_pair_ids_csv=processor_to_listener_applied,processor_to_listener_deferred,listener_to_approver_applied,listener_to_approver_deferred,approver_to_processor_applied,approver_to_processor_deferred`
- `phase6_live_postgres_daemon_runtime_matrix_role_pair_contract=role_pair_leg_a_applied->m10_phase6_scheduler_cycle_applied;role_pair_leg_b_applied->m10_phase6_scheduler_cycle_applied;role_pair_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;role_pair_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_pair_matrix_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_pair_markers -- --exact`

Regression marker:

- `Regression: #5352`

### Bounded Parallel Role-Pair Lane Matrix (Issue #5354)

The live-postgres matrix contracts now include bounded same-host parallel role-pair lanes to
enforce deterministic reason/taxonomy behavior under concurrent two-leg execution.

Deterministic bounded parallel lane markers:

- `phase6_live_postgres_daemon_runtime_matrix_parallel_role_pair_lane_ids_csv=processor_listener_parallel_applied,processor_listener_parallel_deferred,listener_approver_parallel_applied,listener_approver_parallel_deferred`
- `phase6_live_postgres_daemon_runtime_matrix_parallel_role_pair_contract=parallel_lane_leg_a_applied->m10_phase6_scheduler_cycle_applied;parallel_lane_leg_b_applied->m10_phase6_scheduler_cycle_applied;parallel_lane_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;parallel_lane_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_role_pair_lane_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_role_pair_lane_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_role_pair_markers -- --exact`

Regression marker:

- `Regression: #5354`

### Asymmetric Parallel Lane Matrix (Issue #5356)

The live-postgres matrix contracts now include bounded asymmetric parallel lanes so mixed-cadence
concurrent role-pair legs remain deterministic for reason/taxonomy outputs.

Deterministic asymmetric lane markers:

- `phase6_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_ids_csv=processor_listener_asymmetric_parallel_applied,processor_listener_asymmetric_parallel_deferred,listener_approver_asymmetric_parallel_applied,listener_approver_asymmetric_parallel_deferred`
- `phase6_live_postgres_daemon_runtime_matrix_asymmetric_parallel_contract=asymmetric_parallel_leg_a_applied->m10_phase6_scheduler_cycle_applied;asymmetric_parallel_leg_b_applied->m10_phase6_scheduler_cycle_applied;asymmetric_parallel_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;asymmetric_parallel_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_asymmetric_parallel_lane_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_asymmetric_parallel_lane_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_markers -- --exact`

Regression marker:

- `Regression: #5356`

### Parallel Lane Order-Invariance Matrix (Issue #5358)

The live-postgres matrix contracts now enforce order-invariance so bounded same-host parallel lane
outcomes remain equivalent even when lane execution order is permuted.

Deterministic order-invariance markers:

- `phase6_live_postgres_daemon_runtime_matrix_order_invariance_contract=baseline_and_permuted_lane_orders_must_produce_equivalent_sorted_reason_taxonomy_fingerprints`
- `phase6_live_postgres_daemon_runtime_matrix_order_invariance_lane_sets_csv=symmetric_parallel,asymmetric_parallel`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_order_invariance_markers -- --exact`

Regression marker:

- `Regression: #5358`

### Parallel Lane Permutation-Invariance Matrix (Issue #5360)

The live-postgres matrix contracts now enforce deterministic invariance across multiple canonical
lane permutations beyond reverse-order checks.

Deterministic permutation markers:

- `phase6_live_postgres_daemon_runtime_matrix_permutation_ids_csv=baseline,reverse,rotate_left_1,interleaved_even_then_odd`
- `phase6_live_postgres_daemon_runtime_matrix_permutation_invariance_contract=deterministic_permutations_must_preserve_sorted_lane_reason_taxonomy_fingerprints`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_permutation_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_permutation_invariance_markers -- --exact`

Regression marker:

- `Regression: #5360`

### Parallel Lane Fingerprint Schema Contracts (Issue #5362)

The live-postgres matrix contracts now codify explicit fingerprint schema semantics so parallel
lane projections fail closed on field-order or delimiter drift.

Deterministic fingerprint schema markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_fingerprint_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-fingerprint.v1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_fingerprint_field_order_csv=lane_id,leg_a_reason,leg_a_taxonomy,leg_b_reason,leg_b_taxonomy`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_fingerprint_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_fingerprint_schema_markers -- --exact`

Regression marker:

- `Regression: #5362`

### Parallel Lane Topology-Scope Contracts (Issue #5364)

The live-postgres matrix contracts now codify topology scope semantics so same-host and
distributed-label parallel lane bundles remain explicit and deterministic under repeated runs.

Deterministic topology markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology.v1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_ids_csv=same_host_parallel,distributed_label_parallel`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_contract=topology_labels_must_preserve_sorted_lane_reason_taxonomy_fingerprints_under_repeated_runs`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_scope_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_scope_markers -- --exact`

Regression marker:

- `Regression: #5364`

### Parallel Lane Topology Permutation-Invariance Contracts (Issue #5366)

The live-postgres matrix contracts now enforce deterministic invariance when topology profiles are
permuted, so topology-order drift cannot alter sorted topology fingerprint bundles.

Deterministic topology permutation markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_permutation_ids_csv=baseline,reverse,rotate_left_1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_permutation_contract=deterministic_topology_profile_permutations_must_preserve_sorted_topology_fingerprint_bundles`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_permutation_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_permutations_are_invariant -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_permutation_markers -- --exact`

Regression marker:

- `Regression: #5366`

### Parallel Lane Topology Host-Pair Contracts (Issue #5368)

The live-postgres matrix contracts now codify host-pair semantics so same-host and
distributed-labeled topology fingerprints fail closed on host-pair drift.

Deterministic host-pair markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_required_host_pair_ids_csv=node_alpha->node_alpha,node_alpha->node_beta`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_contract=host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pairs_are_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_markers -- --exact`

Regression marker:

- `Regression: #5368`

### Parallel Lane Topology Host-Pair Directionality Contracts (Issue #5370)

The live-postgres matrix contracts now codify host-pair directionality semantics so host-pair
extraction remains non-commutative (`host_a->host_b`) and fails closed on reversed pairs.

Deterministic host-pair directionality markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_extraction_rule=host_a_to_host_b_arrow_notation_non_commutative`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_forbidden_reverse_pairs_csv=node_beta->node_alpha`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_directionality_markers -- --exact`

Regression marker:

- `Regression: #5370`

### Parallel Lane Topology Host-Pair Mapping Contracts (Issue #5372)

The live-postgres matrix contracts now codify explicit topology-id to host-pair mapping rows so
topology labels cannot drift to the wrong host-pair association.

Deterministic topology-id host-pair mapping markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-mapping.v1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_rows_csv=same_host_parallel->node_alpha->node_alpha,distributed_label_parallel->node_alpha->node_beta`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_contract=topology_id_to_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_mapping_markers -- --exact`

Regression marker:

- `Regression: #5372`

### Parallel Lane Topology Lane-Set Mapping Contracts (Issue #5374)

The live-postgres matrix contracts now codify explicit topology-id to lane-set mapping rows so
topology labels cannot drift to the wrong parallel lane-set class.

Deterministic topology-id lane-set mapping markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_set_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-set-mapping.v1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_set_mapping_rows_csv=same_host_parallel->symmetric_parallel,distributed_label_parallel->asymmetric_parallel`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_set_mapping_contract=topology_id_to_lane_set_rows_must_remain_stable_under_repeated_runs_and_permutations`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_lane_set_mapping_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_lane_set_mapping_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_lane_set_mapping_markers -- --exact`

Regression marker:

- `Regression: #5374`

### Parallel Lane Topology Lane-Count Mapping Contracts (Issue #5376)

The live-postgres matrix contracts now codify explicit topology-id to lane-count mapping rows so
topology labels cannot drift to incorrect lane-cardinality expectations.

Deterministic topology-id lane-count mapping markers:

- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_count_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-count-mapping.v1`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_count_mapping_rows_csv=same_host_parallel->4,distributed_label_parallel->4`
- `phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_count_mapping_contract=topology_id_to_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_lane_count_mapping_markers -- --exact`

Regression marker:

- `Regression: #5376`

## Convergence Promotion Marker Contracts (Issue #5301)

Daemon runtime report output now projects deterministic convergence promotion markers that summarize
schema, error-path, concurrency, performance-budget, and cost-budget gates.

Deterministic convergence markers:

- `convergence_promotion_contract_status=verified`
- `convergence_reason_taxonomy_version=kamn.runtime.daemon.convergence.reason-taxonomy.v1`
- `convergence_reason_codes_csv=convergence_promotion_gate_go,convergence_schema_drift_detected,convergence_error_path_drift_detected,convergence_concurrency_drift_detected,convergence_performance_budget_exceeded,convergence_cost_budget_exceeded`
- `convergence_promotion_contract=schema+error_path+concurrency+performance+cost->decision;any_failed_gate=no_go`

Validation commands:

- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_daemon_convergence_projection_fail_closed_reason_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_convergence_promotion_marker_contracts -- --exact`

Regression marker:

- `Regression: #5301`

## Quota Policy Fixture Matrix and Parser Helper Contracts (Issue #4090)

Per-scope quota checker development relies on a deterministic fixture matrix so parser/helper
contracts stay fail closed before runtime checker wiring is expanded in follow-up work.

Deterministic fixture markers:

- `quota_policy_fixture_matrix_path=fixtures/runtime/quota_policy_fixture_matrix.txt`
- `quota_policy_fixture_matrix_schema_version=kamn.runtime.quota-policy-fixture-matrix.v1`
- `quota_policy_reason_taxonomy_version=kamn.runtime.quota-policy-reason-taxonomy.v1`
- `quota_policy_reason_codes_csv=quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive`
- `quota_policy_fixture_columns=case_id|scope|window_seconds|limit|expected_status|expected_reason_code`

Deterministic fail-closed fixture reasons:

- `quota_scope_unknown`
- `quota_window_non_positive`
- `quota_limit_non_positive`

Validation command:

- `cargo test -p kamn-core --test quota_policy_fixture_parser_contract`

Regression marker:

- `Regression: #4090`

## Fairness Starvation Fixture and Checker Contracts (Issue #4092)

Fairness-starvation governance requires deterministic fixture coverage so checker behavior stays
stable across overload policy changes.

Deterministic fixture markers:

- `fairness_fixture_matrix_path=fixtures/runtime/starvation_fairness_fixture_matrix.txt`
- `fairness_fixture_matrix_schema_version=kamn.runtime.fairness-fixture-matrix.v1`
- `fairness_reason_taxonomy_version=kamn.runtime.fairness-policy-reason-taxonomy.v1`
- `fairness_reason_codes_csv=fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap`
- `fairness_fixture_columns=case_id|scope|window_seconds|active_weighted_share|max_weighted_share_gap|expected_status|expected_reason_code`

Deterministic fail-closed fairness reasons:

- `fairness_scope_unknown`
- `fairness_window_non_positive`
- `fairness_max_gap_non_positive`
- `fairness_weighted_share_exceeds_gap`

Validation command:

- `cargo test -p kamn-core --test fairness_policy_checker_contract`

Regression marker:

- `Regression: #4092`

## Daemon OS-Signal Stress Matrix Overload Profiles (Issue #4094)

Daemon OS-signal local-heavy stress evidence must remain deterministic so
degradation and recovery claims are auditable and fail closed.

Deterministic stress matrix markers:

- `daemon_os_signal_stress_matrix_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1`
- `daemon_os_signal_stress_profile_baseline_reason_code=stable_success`
- `daemon_os_signal_stress_profile_injected_overload_reason_code=matrix_failure_threshold_exceeded`
- `daemon_os_signal_stress_profile_recovery_reason_code=stable_success_with_quarantine_followup`
- `daemon_os_signal_stress_profile_runtime_budget_reason_code=runtime_budget_exceeded`
- `daemon_os_signal_stress_profile_quarantine_reason_code=quarantine_reference_present_without_followup`

Docs parity and remediation markers:

- `overload_docs_parity_reason_taxonomy_version=kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1`
- `overload_docs_parity_reason_codes_csv=runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,quarantine_reference_present_without_followup,matrix_failures_within_threshold,stable_success_with_quarantine_followup,stable_success`
- `overload_docs_parity_remediation_map_version=v1`
- `overload_docs_parity_remediation.runtime_budget_exceeded=reduce iterations or increase max-seconds budget after validating reproducer runtime`
- `overload_docs_parity_remediation.matrix_failure_threshold_exceeded=triage failing iteration artifacts and rerun reproducer before promotion`
- `overload_docs_parity_remediation.quarantine_registry_missing=restore .ci/flaky-tests.txt or pass an explicit --registry-file`
- `overload_docs_parity_remediation.quarantine_reference_present_without_followup=add --quarantine-followup-issue #<id> or retire stale quarantine entries`
- `overload_docs_parity_remediation.matrix_failures_within_threshold=track flaky rows and keep threshold + waiver evidence attached to release review`
- `overload_docs_parity_remediation.stable_success_with_quarantine_followup=keep follow-up issue open until quarantine references are retired`
- `overload_docs_parity_remediation.stable_success=no action required; retain report artifact link in release checklist`

Validation commands:

- `bash scripts/ci/run_daemon_os_signal_stress_matrix.sh --iterations 10 --attempts-per-iteration 1 --max-seconds 600 --reproducer-max-seconds 180 --failure-threshold 0 --artifact-dir /tmp/daemon-os-signal-stress-matrix-artifacts --output-json /tmp/daemon-os-signal-stress-matrix-report.json`
- `bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh`

Regression marker:

- `Regression: #4094`
- `Regression: #4097`

## Upgrade Compatibility Marker Matrix Controls (Issue #4181)

Compatibility promotion checks require deterministic matrix validation across version-report and
fork-policy schema/taxonomy markers.

Matrix policy checker command:

- `python3 scripts/kolme/check_upgrade_compatibility_marker_matrix_policy.py --version-report-file /tmp/kolme-version-report.json --fork-policy-report-file /tmp/kolme-fork-compatibility-policy-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-upgrade-compatibility-marker-matrix-policy-report.json`

Deterministic matrix checker markers:

- `reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1`
- `reason_codes_csv=version_report_missing,fork_policy_report_missing,version_report_schema_mismatch,version_report_reason_taxonomy_mismatch,version_report_reason_codes_csv_mismatch,version_report_rehearsal_bypass_guard_status_mismatch,version_report_rehearsal_output_normalization_status_mismatch,fork_policy_report_schema_mismatch,fork_policy_report_reason_taxonomy_mismatch,fork_policy_report_reason_codes_csv_mismatch,fork_policy_report_rehearsal_bypass_guard_status_mismatch,fork_policy_report_rehearsal_output_normalization_status_mismatch,expected_final_decision_mismatch,ci_fast_gate_failed`
- `reason_codes_value=none|<csv>`

Deterministic fail-closed mismatch reasons:

- `version_report_schema_mismatch`
- `version_report_reason_taxonomy_mismatch`
- `version_report_reason_codes_csv_mismatch`
- `fork_policy_report_reason_taxonomy_mismatch`
- `fork_policy_report_reason_codes_csv_mismatch`
- `fork_policy_report_rehearsal_bypass_guard_status_mismatch`
- `expected_final_decision_mismatch`
- `ci_fast_gate_failed`

Regression markers:

- `Regression: #4180`
- `Regression: #4181`

## API Protocol Compliance Mismatch Reason Mapping (Issues #4266, #4270, #4271)

Service API axum ingress protocol compliance checker outputs remain deterministic for promotion/runbook decisions.

Deterministic protocol mismatch mapping markers:

- `service_api_axum_protocol_mismatch_reason_mapping_status=verified`
- `service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1`
- `service_api_axum_protocol_mismatch_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation`
- `service_api_axum_protocol_mismatch_reason_code=none|<reason>`

Deterministic mismatch reason classes:

- `service_api_axum_policy_required_field_missing`
- `service_api_axum_policy_marker_missing`
- `service_api_axum_policy_protocol_taxonomy_mismatch`
- `service_api_axum_policy_limit_contract_mismatch`
- `ci_fast_gate_failed`
- `service_api_axum_policy_expected_decision_mismatch`
- `service_api_axum_policy_violation`

Validation commands:

- `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `bash scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_protocol_mismatch_reason_mapping_controls -- --exact`

Regression markers:

- `Regression: #4270`
- `Regression: #4271`

## TLS Runtime Transport Behavior Contracts

Runtime-commit HTTPS execution in `kolme-live` mode uses an in-process rustls
client transport. Subprocess fallback is not allowed in runtime request paths.

TLS trust-root override:

- `KAMN_KOLME_TLS_CA_FILE` (optional custom CA bundle for runtime-commit HTTPS)

Deterministic fail-closed TLS reason markers:

- `tls certificate verification failed`
- `tls handshake failed`

Validation commands:

- `cargo test -p kamn-core --test kolme_runtime_commit_http_transport functional_https_transport_submit_with_trusted_ca_succeeds -- --exact`
- `cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_https_transport_maps_certificate_errors_to_unavailable -- --exact`
- `cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_https_transport_maps_tls_handshake_failures_to_unavailable -- --exact`
- `cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_https_transport_does_not_use_openssl_subprocess -- --exact`

Regression markers:

- `Regression: #4106`

## Audit Integrity Go/No-Go Policy Controls (Issue #4465)

Release go/no-go validation supports an optional audit-integrity evidence gate using sqlite
crash-recovery policy output as the source report.

Generator controls:

- `--audit-integrity-report-file <path>`
- `--audit-integrity-max-age-seconds <seconds>`

Deterministic audit-integrity taxonomy marker:

- `audit_integrity_reason_taxonomy_version=kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1`

Fail-closed mismatch reasons:

- `gonogo_audit_integrity_reason_taxonomy_version_mismatch`
- `gonogo_audit_integrity_reason_codes_csv_mismatch`

Tamper convergence contract:

- checker must fail closed on `audit integrity gate convergence mismatch` when bundled
  audit-integrity payload markers drift from deterministic rebuild.

Validation commands:

- `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`

Regression marker:

- `Regression: #4465`

## Journal Append/Checkpoint Integrity Controls (Issues #4236, #4240, #4241)

Sqlite crash-recovery durability checks project deterministic append/checkpoint integrity markers for
operations and promotion policy consumers.

Deterministic append/checkpoint markers:

- `append_checkpoint_integrity_status=verified`
- `append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1`
- `append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch`

Deterministic fail-closed mismatch reasons:

- `sqlite_crash_recovery_policy_wal_append_status_mismatch`
- `sqlite_crash_recovery_policy_wal_checkpoint_status_mismatch`
- `sqlite_crash_recovery_policy_append_checkpoint_integrity_status_mismatch`
- `sqlite_crash_recovery_policy_append_checkpoint_reason_taxonomy_version_mismatch`
- `sqlite_crash_recovery_policy_append_checkpoint_reason_codes_csv_mismatch`
- `sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch`

Validation commands:

- `bash scripts/runtime/test_check_sqlite_crash_recovery_live_policy.sh`
- `bash scripts/runtime/test_validate_sqlite_crash_recovery_live.sh`
- `bash scripts/runtime/test_validate_sqlite_crash_recovery_live_contract_lane.sh`

Regression markers:

- `Regression: #4240`
- `Regression: #4241`

## Structured Logging Bootstrap Contracts

`kamn-node` logging bootstrap remains deterministic for all runtime modes.

Environment controls:

- `KAMN_NODE_LOG_LEVEL` -> `error|warn|info|debug|trace` (trimmed, case-insensitive)
- `KAMN_NODE_LOG_FORMAT` -> `text|json` (trimmed, case-insensitive)

Deterministic defaults:

- Level defaults to `info` when unset.
- Format defaults to `text` when unset.
- Structured event fields project fallback markers when omitted:
  - `correlation_id=none`
  - `reason_code=none`

Validation commands:

- `cargo test -p kamn-node regression_log_renderer_projects_default_correlation_and_reason_fields_when_missing -- --nocapture`
- `cargo test -p kamn-node regression_log_renderer_text_projects_default_correlation_and_reason_fields_when_missing -- --nocapture`
- `cargo test -p kamn-node unit_log_config_parses_bootstrap_level_with_whitespace_and_case_insensitive_inputs -- --nocapture`

Regression marker:

- `Regression: #4120`

## Runtime Output Emission Contracts

Critical runtime and signer paths avoid ad-hoc stdio macros and keep output
behavior deterministic.

Output policy:

- `src/main.rs` must not use `println!` or `eprintln!` for runtime report/error output.
- Runtime report output is emitted through bounded stdio writer helpers.
- Failure paths continue to emit structured error events (`node.runtime.execute.failed`)
  with deterministic `reason_code` projection.

Validation commands:

- `cargo test -p kamn-node --test runtime_output_contract integration_runtime_output_contract_enforces_main_entrypoint_path -- --nocapture`
- `cargo test -p kamn-node --test runtime_output_contract -- --nocapture`

Regression marker:

- `Regression: #4122`

## Environment Override Contracts

Environment override names map to the same key contracts regardless of config-file usage.

Examples:

- `KAMN_NODE_CHAIN_ID` -> `chain_id`
- `KAMN_NODE_SYNC_MODE` -> `sync_mode`
- `KAMN_NODE_DAEMON_MAX_TICKS` -> `daemon_max_ticks`
- `KAMN_NODE_DAEMON_TICK_INTERVAL_MS` -> `daemon_tick_interval_ms`
- `KAMN_NODE_ENABLE_GOSSIP` -> `enable_gossip`
- `KAMN_NODE_API_BIND` -> `api_bind`
- `KAMN_NODE_KOLME_LIVE_BASE_URL` -> `kolme_live_base_url`
- `KAMN_NODE_OUTPUT` -> `output`

Invalid override values fail closed with typed `ConfigError` variants.

## Runtime Commit Submit/Finality Policy Controls

The local Kolme runtime-commit live validation lane exposes bounded submit/finality
controls that must stay deterministic for release gating.

Primary controls:

- `--finality-max-seconds`
- `--finality-retry-max-attempts`
- `--finality-retry-backoff-seconds`
- `--max-seconds`
- `--skip-preflight` (explicit override; run mode is still local-only gated)

Policy checker:

- `python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/kolme-local-runtime-commit-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-runtime-commit-live-policy.json`

Deterministic submit/finality reason taxonomy markers:

- `submit_finality_reason_taxonomy_version=kamn.kolme.local-runtime-commit-submit-finality-reason-taxonomy.v1`
- `submit_finality_reason_codes_csv=submit_finality_reason_mismatch_for_finality_enabled_run,submit_finality_reason_mismatch_for_submit_only_run`
- `submit_finality_reason_codes_value=none|submit_finality_reason_mismatch_for_finality_enabled_run|submit_finality_reason_mismatch_for_submit_only_run`

Fail-closed mismatch reasons:

- `submit_finality_reason_mismatch_for_finality_enabled_run`
- `submit_finality_reason_mismatch_for_submit_only_run`

## Production-Mode In-Memory Provider Rejection Controls (Issue #4371)

Production-mode runtime integration must fail closed if command surfaces drift back to in-memory
provider references.

Deterministic rejection markers:

- `runtime_commit_in_memory_provider_reference_detected`
- `runtime_commit_policy_check_in_memory_provider_reference_detected`

In-memory provider marker that must never appear in production command surfaces:

- `InMemoryKolmeRuntimeCommitClient`

Validation commands:

- `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
- `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh`

Regression marker:

- `Regression: #4371`

## Multi-Signer Profile and Quorum Signature-Decision Controls (Issue #4357)

Real-node runtime policy evidence must include deterministic signature-decision markers for
profile/quorum checks so reviewer tooling can distinguish stable signer-governance failures.

Deterministic signature-decision taxonomy markers:

- `signature_decision_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signature-decision-reason-taxonomy.v1`
- `signature_decision_reason_codes_csv=runtime_signer_profile_missing,runtime_signer_profile_invalid,runtime_signer_previous_profile_missing,runtime_signer_previous_profile_invalid,runtime_signer_failover_profile_unchanged,runtime_signer_profile_changed_without_failover,runtime_signer_rotation_epoch_stale,runtime_signer_rotation_epoch_regressed,runtime_signer_attestation_schema_invalid,runtime_signer_attestation_required_approvals_invalid,runtime_signer_attestation_approved_signers_invalid,runtime_signer_attestation_approved_signers_not_unique,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_quorum_linkage_contract_version_invalid,runtime_signer_quorum_linkage_contract_version_mismatch,runtime_signer_quorum_required_approvals_invalid,runtime_signer_quorum_required_approvals_mismatch,runtime_signer_quorum_approved_signers_count_invalid,runtime_signer_quorum_approved_signers_count_mismatch,runtime_signer_quorum_profile_linked_invalid,runtime_signer_quorum_profile_linked_mismatch,runtime_signer_quorum_satisfied_invalid,runtime_signer_quorum_satisfied_mismatch,runtime_signer_quorum_linked_invalid,runtime_signer_quorum_linkage_drift,runtime_signer_quorum_linkage_violation,runtime_signer_failover_attestation_required_approvals_insufficient,runtime_signer_failover_attestation_previous_profile_not_approved`
- `signature_decision_reason_codes_value=none|<csv>`

Key fail-closed quorum/profile reasons:

- `runtime_signer_attestation_quorum_shortfall`
- `runtime_signer_quorum_linkage_drift`
- `runtime_signer_quorum_linkage_violation`
- `runtime_signer_failover_attestation_required_approvals_insufficient`

Validation commands:

- `bash scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
- `bash scripts/kolme/test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh`

Regression marker:

- `Regression: #4357`

Quorum profile matrix fixture controls:

- `signer_quorum_profile_matrix_fixture_status=verified`
- `signer_quorum_profile_matrix_case_labels_csv=linked_non_failover_primary,profile_not_approved_non_failover,quorum_shortfall_non_failover,failover_previous_profile_not_approved,linked_failover_dual_approved`
- `signer_quorum_profile_matrix_fail_closed_reason_codes_csv=runtime_signer_quorum_linkage_violation,runtime_signer_attestation_quorum_shortfall,runtime_signer_failover_attestation_previous_profile_not_approved`

Validation commands:

- `cargo test -p kamn-node signer::signer_policy::tests::unit_signer_quorum_decision_path_matrix -- --exact --nocapture`
- `cargo test -p kamn-node main_tests::signer_tests::integration_kolme_live_signer_preflight_quorum_profile_matrix_paths -- --exact --nocapture`

Regression marker:

- `Regression: #3957`

### Retry Decision Matrix and Jitter Seed Contracts

`kamn-node` keeps retry behavior deterministic and bounded for live runtime submit/finality paths.

Contract helpers and invariants:

- `retry_decision_for_attempt(error, attempt, max_attempts)`:
  - returns `Retry` only for transient classes (`timeout`, `unavailable`) with `attempt < max_attempts`
  - returns `Stop` with `attempt_ceiling_reached` when transient classes hit the configured ceiling
  - returns `Stop` with `malformed_response_fail_fast` for malformed payload classes
- `deterministic_retry_jitter_seed(correlation_id)`:
  - produces a stable seed for a given correlation ID
  - different correlation IDs produce different seeds in contract tests
- `deterministic_retry_backoff_millis_with_jitter(attempt, seed)`:
  - remains deterministic for the same input pair
  - remains bounded by `retry_backoff_cap_ms`

Operational note:

- Active runtime marker emission remains on the deterministic non-jitter schedule (`deterministic_retry_backoff_millis`) until rollout issue `#4110` wires jitter into runtime retry markers.

### Retry Envelope Exhaustion and Reconnect Bound Governance (Issue #4296)

Local retry diagnostics policy enforces deterministic reconnect envelope boundaries in addition to
readiness/jitter markers.

Deterministic taxonomy markers:

- `reason_taxonomy_version=kamn.runtime.local-retry-diagnostics-reason-taxonomy.v2`
- `reason_codes_csv=local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,local_retry_envelope_exhaustion_fail_closed_missing,local_retry_reconnect_attempt_bound_drift,local_retry_reconnect_backoff_bound_drift,ci_local_network_budget_boundary_exceeded`
- `retry_envelope_exhaustion_fail_closed_status=verified`
- `reconnect_attempt_bound_status=verified`
- `reconnect_backoff_bound_status=verified`
- `retry_envelope_max_attempts=3`
- `retry_envelope_max_backoff_seconds=8`

Deterministic fail-closed reasons:

- `local_retry_envelope_exhaustion_fail_closed_missing`
- `local_retry_reconnect_attempt_bound_drift`
- `local_retry_reconnect_backoff_bound_drift`

Validation commands:

- `bash scripts/runtime/test_validate_local_retry_diagnostics_live.sh`
- `bash scripts/runtime/test_check_local_retry_diagnostics_live_policy.sh`
- `bash scripts/runtime/test_validate_local_retry_diagnostics_live_contract_lane.sh`

Regression markers:

- `Regression: #4300`
- `Regression: #4301`

### Live-Node Drift Marker Mismatch Policy Contracts (Issue #4281)

Failover/sync preflight policy checks enforce deterministic fail-closed behavior for live-node drift
marker divergence and missing-marker drift.

Deterministic marker and taxonomy contracts:

- `failover_promotion_gate_status=verified`
- `live_node_drift_parity_status=verified`
- `ci_local_promotion_budget_boundary_status=verified`
- `failover_readiness_reason_taxonomy_version=kamn.runtime.failover-readiness-reason-taxonomy.v1`
- `failover_readiness_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded`
- `failover_sync_drift_policy_status=verified`

Policy checker command:

- `bash scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh check-policy --report-file <report.json> --expected-final-decision GO --ci-fast-gate PASS --output-json <policy.json>`

Deterministic fail-closed reasons:

- `live_node_drift_marker_parity_mismatch`
- `failover_readiness_progress_stalled`
- `ci_local_promotion_budget_boundary_exceeded`
- `failover_sync_drift_policy_required_field_missing:<field>`
- `failover_sync_drift_policy_reason_taxonomy_version_mismatch`
- `failover_sync_drift_policy_reason_codes_csv_mismatch`

Validation commands:

- `bash scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`
- `bash scripts/runtime/test_run_failover_sync_drill_suite.sh`

Regression markers:

- `Regression: #4285`
- `Regression: #4286`

### Block Reconciliation Partition-Healing Mismatch Mapping Contracts (Issues #4251, #4255, #4256)

Block reconciliation partition/rejoin policy checks enforce deterministic fail-closed mismatch mapping
for marker completeness, transport/recovery contracts, and reconciliation taxonomy drift.

Deterministic marker and taxonomy contracts:

- `reconciliation_reason_taxonomy_status=verified`
- `reconciliation_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1`
- `reconciliation_reason_codes_csv=reconciliation_partition_transition_failed,reconciliation_rejoin_transition_failed,reconciliation_publish_drop_recovery_failed,reconciliation_peer_churn_recovery_failed,reconciliation_split_head_unresolved,reconciliation_replay_instability,reconciliation_fixture_contract_failed,reconciliation_unclassified_scenario_failed,reconciliation_runtime_budget_exceeded,reconciliation_ci_fast_gate_failed`
- `partition_healing_mismatch_reason_mapping_status=verified`
- `partition_healing_mismatch_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-healing-mismatch-reason-taxonomy.v1`
- `partition_healing_mismatch_reason_codes_csv=block_reconciliation_partition_rejoin_policy_required_field_missing,block_reconciliation_partition_rejoin_policy_marker_mismatch,block_reconciliation_partition_rejoin_policy_transport_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_mismatch,block_reconciliation_partition_rejoin_policy_recovery_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid,block_reconciliation_partition_rejoin_policy_lane_mode_contract_mismatch,block_reconciliation_partition_rejoin_policy_ci_fast_gate_failed,block_reconciliation_partition_rejoin_policy_expected_decision_mismatch,block_reconciliation_partition_rejoin_policy_violation`
- `partition_healing_mismatch_reason_code=none|<reason>`

Policy checker command:

- `bash scripts/runtime/check_block_reconciliation_partition_rejoin_live_policy.sh --report-file <report.json> --expected-final-decision GO --ci-fast-gate PASS --output-json <policy.json>`

Deterministic fail-closed reasons:

- `block_reconciliation_partition_rejoin_policy_required_field_missing:<field>`
- `block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid`
- `block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_csv_mismatch`
- `block_reconciliation_partition_rejoin_policy_reconciliation_consistency_reason_taxonomy_version_mismatch`
- `block_reconciliation_partition_rejoin_policy_consistency_classification_status_mismatch`

Validation commands:

- `bash scripts/runtime/test_check_block_reconciliation_partition_rejoin_live_policy.sh`
- `bash scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh`

Regression markers:

- `Regression: #4255`
- `Regression: #4256`

Regression marker:

- `Regression: #4109`

## Validation Evidence

Implemented and validated by `kamn-node` tests:

- config file parse + core field projection
- precedence `config < env < CLI`
- invalid env override fail-closed regression
- integration execution path (`parse_args` -> `execute`) with layered precedence

Live validation lane:

- `bash scripts/runtime/test_validate_config_layering_live.sh`
- `bash scripts/runtime/validate_config_layering_live.sh --output-json /tmp/config-layering-live-report.json`

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `layering_contract_status=verified`
- `precedence_contract_status=verified`
- `fail_closed_status=verified`
- `fail_closed_reason_code=invalid_sync_mode_override`

Deterministic fail-closed drill:

- inject invalid override `KAMN_NODE_SYNC_MODE=turbo` while config layering is active
- expected failure marker: `invalid sync mode: turbo`
