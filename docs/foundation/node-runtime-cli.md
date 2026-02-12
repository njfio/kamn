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
  - `--daemon-peer-id <peer-id>` (optional)
  - `--daemon-lifecycle-event <start-connect|handshake-succeeded|heartbeat-missed|heartbeat-restored|disconnect|rejoin>` (repeatable)
- Added Kolme live runtime controls:
  - `--kolme-live-base-url <http(s)-endpoint>`
  - `--kolme-live-provider-hint <provider-hint>`
  - `--kolme-live-signing-profile <signing-profile>`
  - `--kolme-live-strict-signer-contracts`
  - `--kolme-live-signer-profile <ops-primary|ops-secondary>`
  - `--kolme-live-signer-key-source <env-local>`
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
  - `daemon_peer_id`
  - `daemon_peer_lifecycle_final_state`
  - `daemon_peer_lifecycle_applied_events`
  - `kolme_live_provider_client_contract`
  - `kolme_live_base_url`
  - `kolme_live_provider_hint`
  - `kolme_live_signing_profile`
  - `kolme_live_execution_status`
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
- Kolme-live mode:
  - `kamn-node --role processor --runtime-mode kolme-live`
  - `kamn-node --role processor --runtime-mode kolme-live --kolme-live-base-url http://127.0.0.1:3000 --kolme-live-provider-hint kolme-fork-local --kolme-live-signing-profile kolme-fork-secp256k1-v1`
  - `kamn-node --role processor --runtime-mode kolme-live --kolme-live-base-url http://127.0.0.1:3000 --kolme-live-provider-hint kolme-fork-local --kolme-live-signing-profile kolme-fork-secp256k1-v1 --kolme-live-strict-signer-contracts --kolme-live-signer-profile ops-primary --kolme-live-signer-key-source env-local`

## Daemon Runtime Rules
- Supported runtime modes:
  - `bootstrap` (default)
  - `daemon`
- Daemon mode requires:
  - `--daemon-max-ticks`
  - `--daemon-tick-interval-ms`
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
- Daemon execution is deterministic and bounded by tick budget:
  - `daemon_executed_ticks` equals configured `daemon_max_ticks`
  - `daemon_completion_reason` emits `tick-budget-exhausted`

## Kolme Live Runtime Rules
- Supported runtime modes:
  - `bootstrap` (default)
  - `kolme-live`
- Kolme-live mode requires:
  - `--kolme-live-base-url`
  - `--kolme-live-provider-hint`
  - `--kolme-live-signing-profile`
- Strict signer contracts:
  - when `--kolme-live-strict-signer-contracts` is present, `--kolme-live-signer-profile` and `--kolme-live-signer-key-source` are both required
  - supported signer profiles: `ops-primary`, `ops-secondary`
  - supported key source marker: `env-local`
  - fail-closed error semantics:
    - empty profile/source declarations are rejected
    - unsupported profile/source declarations are rejected
    - strict profile declaration must not conflict with `KAMN_KOLME_LIVE_SIGNER_PROFILE` when that env marker is set
- Provider wiring is fail-closed:
  - runtime config must reject in-memory provider-hint markers such as `InMemoryKolmeRuntimeCommitClient`
  - signing profile must match `kolme-fork-secp256k1-v1`
- Runtime path constructs `KolmeRuntimeCommitLiveProvider` with deterministic transport timeout, submits one deterministic runtime-commit request, and emits bounded finality follow-up checks:
  - pending submit receipts poll finality via `/runtime-commit/status` with max-attempt budget `2`
  - malformed finality responses fail closed
  - finality transport timeout/unavailable keeps execution in pending status without falling back to in-memory adapters
- Runtime reports:
  - `kolme_live_provider_client_contract=KolmeRuntimeCommitLiveProvider`
  - `kolme_live_execution_status=submitted;commit_id=<deterministic-commit-id>;finality=<pending|final|failed>;resolution=<submit-receipt|finality-polled|finality-timeout|finality-unavailable>`

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
cargo test -p kamn-node integration_runtime_kolme_live_renders_provider_contract_markers
cargo test -p kamn-node regression_runtime_kolme_live_rejects_provider_marker_drift
```

## Processor HA Runtime References

- Processor HA snapshot restore and construct-lock contract details:
  - `docs/foundation/runtime-processor-ha.md`
- Processor daemon lease tick gate reference:
  - `execute_processor_daemon_tick` in `crates/kamn-core/src/runtime.rs`
