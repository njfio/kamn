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

Kolme-live keys:

- `kolme_live_base_url`, `kolme_live_provider_hint`, `kolme_live_signing_profile`
- `kolme_live_strict_signer_contracts`, `kolme_live_signer_profile`, `kolme_live_signer_key_source`

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
