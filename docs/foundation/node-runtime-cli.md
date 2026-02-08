# Node Runtime CLI Contracts (Issues #306 / #307 / #309 / #310)

This document captures the first two node-runtime productionization slices for machine-readable output and local role profile projection.

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

## Output Mode Rules
- Default behavior remains text output when `--output` is omitted.
- JSON output is deterministic and includes:
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

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-node
cargo fmt --check
cargo clippy -p kamn-node -- -D warnings
```

Then run broader regression:

```bash
cargo test -p kamn-core
```
