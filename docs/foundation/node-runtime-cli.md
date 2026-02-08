# Node Runtime CLI Contracts (Issues #306 / #307)

This document captures the first node-runtime productionization slice for machine-readable CLI output.

## Scope Delivered
- Added output-mode support to `crates/kamn-node/src/main.rs`:
  - `--output text` (default)
  - `--output json`
- Added deterministic report rendering helpers:
  - `build_bootstrap_report(...)`
  - `render_bootstrap_report(...)`
- Added explicit invalid-mode handling through `ConfigError::InvalidOutputMode`.

## Output Mode Rules
- Default behavior remains text output when `--output` is omitted.
- JSON output is deterministic and includes:
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
  - `components`
- Invalid modes are rejected with explicit typed error.

## Test Coverage Mapping
- Unit:
  - default mode behavior and mode parsing checks
- Functional:
  - deterministic JSON rendering contract
- Integration:
  - CLI parse -> bootstrap -> render projection path
- Regression:
  - invalid output mode rejection (`Regression: #307`)

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
