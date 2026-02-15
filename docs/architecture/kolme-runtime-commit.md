# Kolme Runtime Commit Architecture

This document captures the `kamn-node` runtime-commit execution path for `--runtime-mode kolme-live`, including continuous-cycle behavior.

## Core Flow

Entry point:

- `crates/kamn-node/src/main.rs`
- `RuntimeModeKind::KolmeLive`

Execution module:

- `crates/kamn-node/src/runtime_kolme_live.rs`

Per-cycle sequence:

1. Build deterministic runtime-commit request payload.
2. Resolve signer material and emit signed wire payload.
3. Submit payload through `KolmeRuntimeCommitLiveProvider`.
4. Resolve receipt finality (`submitted`/`duplicate` plus `pending|final|failed`).
5. Poll finality when receipt is pending.
6. Emit deterministic execution status and observability telemetry.

## Continuous Mode

Continuous mode is enabled in `runtime-mode kolme-live` when both controls are present:

- `--daemon-max-ticks <positive-integer>`
- `--daemon-tick-interval-ms <positive-integer>`

Behavior:

- one runtime-commit/finality cycle per configured tick
- deterministic sleep between cycles using the provided interval
- fail-closed validation when one control is provided without the other
- final execution status includes continuity markers:
  - `continuous_mode=enabled`
  - `continuous_cycle=<n>`
  - `continuous_cycle_count=<N>`
  - `continuous_cycle_interval_ms=<ms>`
  - `continuous_completed_cycles=<N>`

## Failure Handling

Fail-closed behavior is preserved for continuous and single-cycle modes:

- malformed provider responses fail immediately
- provider hint drift fails immediately
- unsupported signer/profile declarations fail immediately
- transient submit/finality transport errors retry with bounded deterministic backoff

### Signer Provenance Failure Taxonomy

Signer contracts are fail-closed and reason-code stable:

- `production_signer_key_source_env_local_forbidden`:
  production-targeted strict signer contracts reject `env-local` key-source declarations.
- `fallback_signer_secret_present_violation`:
  fallback private-key env marker `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK` must remain unset.
- `managed_signer_raw_private_key_forbidden`:
  managed-external profiles reject raw private-key env markers for the selected signer profile.
- `managed_signer_backend_required_missing`:
  managed-external execution requires `KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND`.
- `managed_signer_key_reference_missing` / `managed_signer_key_reference_invalid`:
  managed-external signer key-reference contracts are missing or malformed.
- `managed_signer_public_key_marker_missing` / `managed_signer_public_key_marker_invalid`:
  managed-external signer public-key provenance markers are missing or malformed.
- `managed_signer_backend_response_provenance_missing` / `managed_signer_backend_response_provenance_malformed` / `managed_signer_backend_response_provenance_mismatch`:
  managed-external backend output provenance is missing, malformed, or does not match expected signer key material.

## Validation Evidence

Primary tests:

- `main_tests::core_behavior_tests::functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles`
- `main_tests::cli_contract_tests::rejects_kolme_live_continuous_mode_without_tick_interval`
- `main_tests::cli_contract_tests::rejects_kolme_live_continuous_mode_without_max_ticks`

Command:

- `cargo test -p kamn-node -- rejects_kolme_live_continuous_mode_without_tick_interval rejects_kolme_live_continuous_mode_without_max_ticks functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles`
- `bash scripts/kolme/run_continuous_runtime_commit_contract_lane.sh`
- `bash scripts/kolme/validate_continuous_runtime_commit_live.sh`
