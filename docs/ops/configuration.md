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
