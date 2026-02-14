# Node Runtime CLI Contracts (Issues #306 / #307 / #309 / #310 / #335 / #336 / #348 / #349 / #2175)

This document captures node-runtime productionization slices for machine-readable output, local role profile projection, diagnostics snapshots, deterministic runtime planning execution, and deterministic recovery-check evaluation.

## Scope Delivered
- Added output-mode support to `crates/kamn-node/src/main.rs`:
  - `--output text` (default)
  - `--output json`
- Added deterministic report rendering helpers:
  - `build_bootstrap_report(...)`
  - `render_bootstrap_report(...)`
- Added explicit invalid-mode handling through `ConfigError::InvalidOutputMode`.
- Added local profile command surface:
  - `--profile local-processor`
  - `--profile local-listener`
  - `--profile local-approver`
- Added config layering command surface:
  - `--config-file <path>`
  - `KAMN_NODE_CONFIG_FILE=<path>`
- Added explicit invalid-profile handling through `ConfigError::InvalidNodeProfile`.
- Added diagnostics mode command surface:
  - `--diagnostics basic` (default)
  - `--diagnostics snapshot`
- Added explicit invalid diagnostics-mode handling through `ConfigError::InvalidDiagnosticsMode`.
- Added runtime execution mode command surface:
  - `--runtime-mode bootstrap` (default)
  - `--runtime-mode planning`
  - `--runtime-mode recovery-check`
  - `--runtime-mode daemon`
  - `--runtime-mode kolme-live`
- Added runtime planning inputs:
  - `--expected-state-hash <state-hash>`
  - `--proposal <id|sender-did|nonce|state-hash>` (repeatable)
- Added runtime recovery-check inputs:
  - `--expected-state-version <state-version>`
  - `--expected-state-hash <state-hash>`
  - `--rejoin-attempt <node-id|state-version|state-hash|resume-token>` (repeatable)
- Added daemon bounded-loop controls:
  - `--daemon-max-ticks <positive-integer>`
  - `--daemon-tick-interval-ms <positive-integer>`
  - `--daemon-shutdown-signal-tick <positive-integer>` (repeatable, optional)
  - `--daemon-shutdown-drain-ticks <positive-integer>` (required when shutdown signal tick is configured)
  - `--daemon-shutdown-timeout-ticks <positive-integer>` (required when shutdown signal tick is configured)
  - `--daemon-peer-id <peer-id>` (optional)
  - `--daemon-lifecycle-event <start-connect|handshake-succeeded|heartbeat-missed|heartbeat-restored|disconnect|rejoin>` (repeatable)
- Added runtime observability endpoint controls:
  - `--observability-endpoint-bind <host:port>`
  - `--observability-endpoint-metrics-path </path>` (default: `/metrics`)
  - `--observability-endpoint-health-path </path>` (default: `/healthz`)
  - `--observability-endpoint-max-requests <positive-integer>` (default: `1`)
  - `--observability-endpoint-idle-timeout-ms <positive-integer>` (default: `5000`)
- Added Kolme live runtime controls:
  - `--kolme-live-base-url <http(s)-endpoint>`
  - `--kolme-live-provider-hint <provider-hint>`
  - `--kolme-live-signing-profile <signing-profile>`
  - `--kolme-live-strict-signer-contracts`
  - `--kolme-live-signer-profile <ops-primary|ops-secondary>`
  - `--kolme-live-signer-key-source <env-local|managed-external>`
- Added explicit runtime mode and proposal validation handling through:
  - `ConfigError::InvalidRuntimeMode`
  - `ConfigError::InvalidExpectedStateVersion`
  - `ConfigError::InvalidDaemonControlArgument`
  - `ConfigError::InvalidDaemonLifecycleEvent`
  - `ConfigError::InvalidKolmeLiveProviderHint`
  - `ConfigError::InvalidKolmeLiveSigningProfile`
  - `ConfigError::InvalidProposalArgument`
  - `ConfigError::InvalidRejoinAttemptArgument`
  - `ConfigError::RuntimePlanner`
  - `ConfigError::RuntimeRecovery`
  - `ConfigError::RuntimeDaemonLifecycle`
  - `ConfigError::RuntimeKolmeLive`

## Output Mode Rules
- Default behavior remains text output when `--output` is omitted.
- JSON output is deterministic and includes:
  - `runtime_mode`
  - `diagnostics_mode`
  - `profile`
  - `role`
  - `chain_id`
  - `chain_version`
  - `storage_dir`
  - `gossip_enabled`
  - `sync_mode`
  - `sync_startup`
  - `sync_recovery`
  - `state_version`
  - `pending_migrations`
  - `component_count`
  - `planning_expected_state_hash`
  - `planning_candidate_count`
  - `planning_scheduled_candidate_ids`
  - `recovery_expected_state_version`
  - `recovery_expected_state_hash`
  - `recovery_attempt_count`
  - `recovery_decisions`
  - `daemon_max_ticks`
  - `daemon_tick_interval_ms`
  - `daemon_executed_ticks`
  - `daemon_completion_reason`
  - `daemon_observability_latency_p50_ms`
  - `daemon_observability_latency_p99_ms`
  - `daemon_observability_throughput_tps`
  - `daemon_observability_error_rate_bps`
  - `daemon_observability_availability_bps`
  - `daemon_observability_health`
  - `daemon_observability_alert_count`
  - `daemon_peer_id`
  - `daemon_peer_lifecycle_final_state`
  - `daemon_peer_lifecycle_applied_events`
  - `kolme_live_provider_client_contract`
  - `kolme_live_base_url`
  - `kolme_live_provider_hint`
  - `kolme_live_signing_profile`
  - `kolme_live_signer_profile_selector_env`
  - `kolme_live_signer_profile`
  - `kolme_live_signer_key_source`
  - `kolme_live_signer_private_key_env`
  - `kolme_live_execution_status`
  - `kolme_live_observability_latency_p50_ms`
  - `kolme_live_observability_latency_p99_ms`
  - `kolme_live_observability_throughput_tps`
  - `kolme_live_observability_error_rate_bps`
  - `kolme_live_observability_availability_bps`
  - `kolme_live_observability_health`
  - `kolme_live_observability_alert_count`
  - `components`
- Invalid modes are rejected with explicit typed error.

## Local Profile Rules
- Supported profiles:
  - `local-processor`
  - `local-listener`
  - `local-approver`
- Profile defaults are deterministic:
  - `chain_id`: `kamn-localnet`
  - `chain_version`: `v0.1.0`
  - `storage_dir`: role-scoped (`./data/processor`, `./data/listener`, `./data/approver`)
  - `sync_mode`: `fast`
  - `enable_gossip`: `true`
  - `role`: mapped from selected profile
- Explicit CLI flags override profile defaults (`--chain-id`, `--storage-dir`, `--sync-mode`, `--disable-gossip`, `--role`).
- Invalid profiles are rejected with explicit typed error.

## Configuration Layering Rules
- Config layering is enabled when either `--config-file` or `KAMN_NODE_CONFIG_FILE` is set.
- Config format is deterministic line-oriented `key=value` with optional `#` comments.
- Boolean keys use strict `true|false`; invalid values fail closed.
- Precedence order (low -> high):
  - built-in defaults
  - profile defaults
  - config file values
  - `KAMN_NODE_*` environment overrides
  - explicit CLI flags
- Environment overrides are projected only when config layering is active.
- Invalid config lines, unknown keys, duplicate `--config-file` declarations, and invalid env marker values are rejected with typed `ConfigError::InvalidNodeConfig` or existing typed parse errors (for example `InvalidSyncMode`).
- Full operator reference: `docs/ops/configuration.md`.

## Diagnostics Snapshot Rules
- Supported diagnostics modes:
  - `basic` (default)
  - `snapshot`
- Snapshot output includes deterministic component summary:
  - `component_count`
  - `components`
- Invalid diagnostics modes are rejected with explicit typed error.

## Runtime Planning Rules
- Supported runtime modes:
  - `bootstrap` (default)
  - `planning`
- Planning mode requires:
  - `--expected-state-hash`
  - at least one `--proposal`
- Proposal argument format is strict:
  - `<id|sender-did|nonce|state-hash>`
- Planning mode uses deterministic candidate ordering inherited from `DeterministicProposalPlanner`:
  - nonce ascending
  - sender DID ascending
  - candidate ID ascending
- Duplicate candidate IDs and stale state hashes are rejected with explicit typed runtime planner error.
- Runtime planning outputs include:
  - `planning_expected_state_hash`
  - `planning_candidate_count`
  - `planning_scheduled_candidate_ids`

## Recovery Check Rules
- Supported runtime modes:
  - `bootstrap` (default)
  - `recovery-check`
- Recovery-check mode requires:
  - `--expected-state-version`
  - `--expected-state-hash`
  - at least one `--rejoin-attempt`
- Rejoin-attempt argument format is strict:
  - `<node-id|state-version|state-hash|resume-token>`
- Recovery-check mode evaluates rejoin attempts in deterministic input order using `RecoveryRejoinGuard`.
- Recovery-check decision mapping:
  - accepted rejoin -> `rejoin-accepted`
  - lagging state -> `catch-up-required:<from_version>-><to_version>`
- Replay resume tokens and version/hash mismatch scenarios are rejected with explicit typed runtime recovery error.
- Runtime recovery-check outputs include:
  - `recovery_expected_state_version`
  - `recovery_expected_state_hash`
  - `recovery_attempt_count`
  - `recovery_decisions`

## Runtime Mode Command Examples
- Planning mode:
  - `kamn-node --role processor --runtime-mode planning`
  - `kamn-node --role processor --runtime-mode planning --expected-state-hash state-1 --proposal tx-1|did:kamn:agent:aaa|1|state-1`
- Recovery-check mode:
  - `kamn-node --role processor --runtime-mode recovery-check`
  - `kamn-node --role processor --runtime-mode recovery-check --expected-state-version 42 --expected-state-hash state-42 --rejoin-attempt node-a|42|state-42|resume-1`
- Daemon mode:
  - `kamn-node --role processor --runtime-mode daemon`
  - `kamn-node --role processor --runtime-mode daemon --daemon-max-ticks 3 --daemon-tick-interval-ms 25`
  - `kamn-node --role processor --runtime-mode daemon --daemon-max-ticks 10 --daemon-tick-interval-ms 25 --daemon-shutdown-signal-tick 3 --daemon-shutdown-drain-ticks 2 --daemon-shutdown-timeout-ticks 4`
- Kolme-live mode:
  - `kamn-node --role processor --runtime-mode kolme-live`
  - `kamn-node --role processor --runtime-mode kolme-live --kolme-live-base-url http://127.0.0.1:3000 --kolme-live-provider-hint kolme-fork-local --kolme-live-signing-profile kolme-fork-secp256k1-v1`
  - `kamn-node --role processor --runtime-mode kolme-live --kolme-live-base-url http://127.0.0.1:3000 --kolme-live-provider-hint kolme-fork-local --kolme-live-signing-profile kolme-fork-secp256k1-v1 --kolme-live-strict-signer-contracts --kolme-live-signer-profile ops-primary --kolme-live-signer-key-source env-local`
  - `KAMN_KOLME_LIVE_SIGNER_KEY_REF=secure:aws-kms:role-operator/key-live-ops-primary KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX=03af446f76cf36092a4e45864210a1dbf03e872756eec21de61910859f8a607dd2 KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND='sh /opt/kamn/signer-backend.sh' kamn-node --role processor --runtime-mode kolme-live --kolme-live-base-url http://127.0.0.1:3000 --kolme-live-provider-hint kolme-fork-local --kolme-live-signing-profile kolme-fork-secp256k1-v1 --kolme-live-strict-signer-contracts --kolme-live-signer-profile ops-primary --kolme-live-signer-key-source managed-external`

## Daemon Runtime Rules
- Supported runtime modes:
  - `bootstrap` (default)
  - `daemon`
- Daemon mode requires:
  - `--daemon-max-ticks`
  - `--daemon-tick-interval-ms`
- Shutdown contract controls:
  - `--daemon-shutdown-signal-tick` may be supplied multiple times to inject deterministic shutdown signals.
  - when any shutdown signal tick is supplied, both `--daemon-shutdown-drain-ticks` and `--daemon-shutdown-timeout-ticks` are mandatory.
  - drain/timeout controls without a shutdown signal tick are rejected.
- Daemon loop controls must be positive integers.
- Optional daemon lifecycle inputs:
  - `--daemon-peer-id`
  - repeatable `--daemon-lifecycle-event`
- Daemon lifecycle events are evaluated in input order using `PeerLifecycle` transitions.
- Invalid lifecycle event names are rejected with explicit typed error.
- Invalid lifecycle transitions are rejected with explicit typed runtime daemon lifecycle error.
- Processor daemon tick execution requires an active construct-lock lease owner and matching fencing token.
- Daemon lease checks align with `execute_processor_daemon_tick` validation in `kamn-core`.
- Missing or invalid daemon lease execution is rejected with typed construct-lock errors.
- Daemon execution is deterministic and bounded:
  - no valid shutdown signal => `daemon_executed_ticks` equals configured `daemon_max_ticks` and `daemon_completion_reason` emits `tick-budget-exhausted`
  - graceful completion => `daemon_completion_reason` emits `graceful-shutdown:...`
  - timeout/fail-closed completion => `daemon_completion_reason` emits `graceful-shutdown-timeout:...`
  - repeated/late shutdown signals are counted as `ignored_signals` in completion metadata.
- Structured daemon completion marker fields:
  - `shutdown_drain_status=<not-signaled|completed|timeout>`
  - `shutdown_signal_tick=<u64|none>`
  - `shutdown_drain_ticks=<u64>`
  - `shutdown_timeout_ticks=<u64>`
  - `shutdown_ignored_signals=<u64>`
- Daemon observability telemetry is emitted in deterministic report fields:
  - `daemon_observability_latency_p50_ms`
  - `daemon_observability_latency_p99_ms`
  - `daemon_observability_throughput_tps`
  - `daemon_observability_error_rate_bps`
  - `daemon_observability_availability_bps`
  - `daemon_observability_health`
  - `daemon_observability_alert_count`

## Runtime Observability Endpoint Rules
- Endpoint export is optional and enabled only when `--observability-endpoint-bind` is set.
- Endpoint path controls:
  - metrics: `--observability-endpoint-metrics-path` (defaults to `/metrics`)
  - health: `--observability-endpoint-health-path` (defaults to `/healthz`)
- Endpoint budget controls:
  - `--observability-endpoint-max-requests` bounds accepted requests before shutdown.
  - `--observability-endpoint-idle-timeout-ms` bounds wait time for incoming requests.
- Endpoint validation:
  - endpoint path controls without bind address are rejected.
  - metrics/health paths must start with `/`.
  - request and timeout controls must be positive integers.
- Supported payloads:
  - `/metrics` returns deterministic Prometheus text metrics for latency/throughput/error/availability, health, and alert count.
  - `/healthz` returns deterministic JSON health snapshot for runtime mode and source telemetry.
- Example:
  - `kamn-node --role processor --runtime-mode daemon --daemon-max-ticks 3 --daemon-tick-interval-ms 25 --observability-endpoint-bind 127.0.0.1:9108 --observability-endpoint-max-requests 1`

## Kolme Live Runtime Rules
- Supported runtime modes:
  - `bootstrap` (default)
  - `kolme-live`
- Kolme-live mode requires:
  - `--kolme-live-base-url`
  - `--kolme-live-provider-hint`
  - `--kolme-live-signing-profile`
  - `--kolme-live-signer-key-source`
- Optional continuous mode controls for kolm-live:
  - `--daemon-max-ticks <positive-integer>`
  - `--daemon-tick-interval-ms <positive-integer>`
  - both controls are fail-closed pair requirements in kolm-live mode
- Strict signer contracts:
  - when `--kolme-live-strict-signer-contracts` is present, `--kolme-live-signer-profile` and `--kolme-live-signer-key-source` are both required
  - supported signer profiles: `ops-primary`, `ops-secondary`
  - supported key source markers: `env-local`, `managed-external`
  - managed-external key-reference env markers:
    - `ops-primary`: `KAMN_KOLME_LIVE_SIGNER_KEY_REF`
    - `ops-secondary`: `KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY`
  - managed-external signer public-key env markers:
    - `ops-primary`: `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX`
    - `ops-secondary`: `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY`
    - missing marker fails closed with `managed_signer_public_key_marker_missing`
    - invalid/empty/non-secp256k1 marker fails closed with `managed_signer_public_key_marker_invalid`
  - managed-external mode rejects raw private-key env markers for the selected profile with deterministic reason code `managed_signer_raw_private_key_forbidden`
  - managed-external signer mode requires `KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND`; if absent, runtime fails closed with `managed_signer_backend_required_missing`
  - managed-external compatibility marker parsing:
    - `KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED=true|false`
    - invalid/empty values fail closed with `managed_signer_backend_required_invalid`
    - marker presence does not relax mandatory backend command execution for managed-external signing
  - managed-external backend command output contract:
    - `signature_hex=<128-hex>`
    - `recovery_id=<0..3>`
    - `signer_public_key_hex=<33-byte-compressed-secp256k1-hex>`
    - missing signer provenance marker fails closed with `managed_signer_backend_response_provenance_missing`
    - malformed signer provenance marker fails closed with `managed_signer_backend_response_provenance_malformed`
    - signer provenance mismatch fails closed with `managed_signer_backend_response_provenance_mismatch`
  - fail-closed error semantics:
    - empty profile/source declarations are rejected
    - unsupported profile/source declarations are rejected
    - strict profile declaration must not conflict with `KAMN_KOLME_LIVE_SIGNER_PROFILE` when that env marker is set
- Provider wiring is fail-closed:
  - runtime config must reject in-memory provider-hint markers such as `InMemoryKolmeRuntimeCommitClient`
  - signing profile must match `kolme-fork-secp256k1-v1`
- Runtime path constructs `KolmeRuntimeCommitLiveProvider` with deterministic transport timeout and runs bounded runtime-commit/finality cycles:
  - default mode executes one deterministic runtime-commit request
  - continuous mode executes one runtime-commit/finality sequence per `--daemon-max-ticks` cycle
  - pending submit receipts poll finality via `/runtime-commit/status` with max-attempt budget `2`
  - submit retries are bounded to max-attempt budget `3`
  - finality retries are bounded to max-attempt budget `3`
  - retry backoff is deterministic exponential (`10ms`, `20ms`, capped at `40ms`)
  - structured retry markers include `attempt`, `max_attempts`, `reason`, `backoff_ms`, and `correlation_id`
  - malformed finality responses fail closed
  - finality transport timeout/unavailable keeps execution in pending status without falling back to in-memory adapters
  - managed-external signer mode enforces secure-provider handshake routing before payload signing and maps provider failures to deterministic reason codes such as:
    - `managed_signer_provider_unavailable`
    - `managed_signer_provider_handshake_rejected`
    - `managed_signer_backend_error`
- Runtime reports:
  - `kolme_live_provider_client_contract=KolmeRuntimeCommitLiveProvider`
  - signer-selection evidence markers:
    - `kolme_live_signer_profile_selector_env=KAMN_KOLME_LIVE_SIGNER_PROFILE`
    - `kolme_live_signer_profile=ops-primary|ops-secondary`
    - `kolme_live_signer_key_source=env-local|managed-external`
    - `kolme_live_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX|KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY`
  - `kolme_live_execution_status=submitted;commit_id=<deterministic-commit-id>;finality=<pending|final|failed>;resolution=<submit-receipt|finality-polled|finality-timeout|finality-unavailable>;submit_attempts=<u32>;submit_retry_reason=<none|timeout|unavailable>;submit_retry_max_attempts=3;finality_retry_attempts=<u32>;finality_retry_reason=<none|timeout|unavailable>;finality_retry_max_attempts=3;retry_backoff_base_ms=10;retry_backoff_cap_ms=40`
  - `kolme_live_observability_latency_p50_ms=<u64>`
  - `kolme_live_observability_latency_p99_ms=<u64>`
  - `kolme_live_observability_throughput_tps=<u64>`
  - `kolme_live_observability_error_rate_bps=<u64>`
  - `kolme_live_observability_availability_bps=<u64>`
  - `kolme_live_observability_health=<healthy|degraded|critical>`
  - `kolme_live_observability_alert_count=<usize>`

## Service API Ingress Logging Contracts
- Service API ingress emits structured request markers for deterministic correlation and auditability:
  - `service.api.request.received`
  - `service.api.request.outcome`
- Correlation marker is deterministic per request envelope:
  - authenticated requests: `service-api:<method-lower>:<path>:<sender-did>:<nonce>`
  - unauthenticated/parse-error paths: deterministic fallback tag
- Required marker fields:
  - `correlation_id`
  - `method`
  - `path`
  - `status_code` (for outcome marker)
  - `outcome` (for outcome marker)
- Service API JSON envelopes are represented by serde-backed DTO contracts in `crates/kamn-node/src/service_api_endpoint.rs`:
  - `ServiceApiHealthBody`
  - `ServiceApiMessageCreateBody`
  - `ServiceApiMessageGetBody`
  - `ServiceApiChannelCreateBody`
  - `ServiceApiChannelMessagesBody`
  - `ServiceApiTaskCreateBody`
  - `ServiceApiTaskGetBody`
  - `ServiceApiAgentGetBody`
  - `ServiceApiErrorBody`
  - `ServiceApiWebsocketStateTransitionBody`
- Payload decode failures map to deterministic reason-code prefixes:
  - `service_api_payload_json_syntax_invalid`
  - `service_api_payload_structure_invalid`
  - `service_api_payload_io_error`

## Decomposition Guardrails
- `main.rs` orchestrates only and must not absorb parser/signer/wire/live-runtime implementation details.
- Canonical module ownership map:
  - `docs/architecture/kamn-node-module-map.md`
- Module ownership boundaries:
  - `src/cli.rs` owns CLI parsing and parser helper validation.
  - `src/runtime_kolme_live.rs` owns live submit/finality execution.
  - `src/signer.rs` owns signer policy and signing adapters.
  - `src/wire_payload.rs` owns deterministic wire payload rendering.
- Regression marker:
  - `Regression: #2606`

## Test Coverage Mapping
- Unit:
  - default mode behavior and mode parsing checks
- Functional:
  - deterministic JSON rendering contract
- Integration:
  - CLI parse -> bootstrap -> render projection path
- Regression:
  - invalid output mode rejection (`Regression: #307`)
  - invalid profile rejection (`Regression: #310`)
  - invalid diagnostics mode rejection (`Regression: #313`)
  - duplicate/stale runtime planning candidate rejection (`Regression: #335`)
  - replay/version/hash recovery-check rejection (`Regression: #336`)
  - zero/invalid daemon bounded-loop control rejection (`Regression: #348`)
  - invalid daemon lifecycle transition rejection (`Regression: #349`)
  - daemon lease guard no-lease/invalid-owner rejection (`Regression: #388`)
  - in-memory fallback and invalid signing profile rejection (`Regression: #2175`)
  - provider marker drift rejection in live submit/finality flow (`Regression: #2176`)

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-node
cargo test -p kamn-node --test node_runtime_cli_docs
cargo test -p kamn-node --test node_module_map_docs
cargo test -p kamn-core --test runtime_network_docs
cargo test -p kamn-core construct_lock
cargo fmt --check
cargo clippy -p kamn-node -- -D warnings
```

Then run broader regression:

```bash
cargo test -p kamn-core
```

### Daemon-focused fast lane

```bash
cargo test -p kamn-node integration_runtime_daemon_renders_bounded_completion_output
cargo test -p kamn-node regression_runtime_daemon_rejects_invalid_lifecycle_transition
cargo test -p kamn-node functional_runtime_daemon_applies_graceful_shutdown_signal
cargo test -p kamn-node integration_runtime_daemon_shutdown_timeout_is_fail_closed
cargo test -p kamn-node integration_runtime_kolme_live_renders_provider_contract_markers
cargo test -p kamn-node regression_runtime_kolme_live_rejects_provider_marker_drift
```

## Processor HA Runtime References

- Processor HA snapshot restore and construct-lock contract details:
  - `docs/foundation/runtime-processor-ha.md`
- Processor daemon lease tick gate reference:
  - `execute_processor_daemon_tick` in `crates/kamn-core/src/runtime.rs`
