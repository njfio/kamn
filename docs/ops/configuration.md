# Node Configuration Layering

This document defines the deterministic configuration contracts for `kamn-node`.

## Scope

Phase 6.2 implementation adds:

- `--config-file <path>` support using a deterministic `key=value` format.
- Environment-variable overrides for selected node settings.
- Strict validation with fail-closed behavior on malformed config or invalid override values.

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

Kolme-live keys:

- `kolme_live_base_url`, `kolme_live_provider_hint`, `kolme_live_signing_profile`
- `kolme_live_strict_signer_contracts`, `kolme_live_signer_profile`, `kolme_live_signer_key_source`

## Async API Backpressure Failure Modes (Issue #4315)

`kamn-node` async API ingress limits remain fail closed under bounded-concurrency pressure.

Deterministic taxonomy markers:

- `service_api_backpressure_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1`
- `service_api_lifecycle_rejection_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1`
- `service_api_lifecycle_rejection_reason_codes_csv=service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid`
- `async_lifecycle_backpressure_projection_status=verified`
- `reason_codes_value=none|service_api_axum_policy_*`

Backpressure reason markers:

- `service_api_ingress_concurrency_limit_exceeded`
- `service_api_ingress_rate_limit_exceeded`
- `service_api_ingress_sender_rate_limit_exceeded`

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
- `signature_decision_reason_codes_csv=runtime_signer_profile_missing,runtime_signer_profile_invalid,runtime_signer_previous_profile_missing,runtime_signer_previous_profile_invalid,runtime_signer_failover_profile_unchanged,runtime_signer_profile_changed_without_failover,runtime_signer_rotation_epoch_stale,runtime_signer_attestation_schema_invalid,runtime_signer_attestation_required_approvals_invalid,runtime_signer_attestation_approved_signers_invalid,runtime_signer_attestation_approved_signers_not_unique,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_quorum_linkage_contract_version_invalid,runtime_signer_quorum_linkage_contract_version_mismatch,runtime_signer_quorum_required_approvals_invalid,runtime_signer_quorum_required_approvals_mismatch,runtime_signer_quorum_approved_signers_count_invalid,runtime_signer_quorum_approved_signers_count_mismatch,runtime_signer_quorum_profile_linked_invalid,runtime_signer_quorum_profile_linked_mismatch,runtime_signer_quorum_satisfied_invalid,runtime_signer_quorum_satisfied_mismatch,runtime_signer_quorum_linked_invalid,runtime_signer_quorum_linkage_drift,runtime_signer_quorum_linkage_violation,runtime_signer_failover_attestation_required_approvals_insufficient,runtime_signer_failover_attestation_previous_profile_not_approved`
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
