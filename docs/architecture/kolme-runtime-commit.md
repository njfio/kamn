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

### Shared Transport Transient Classifier and Bounded Retry Schedule

- `retry_classifier_contract_version=v1`
- `retry_backoff_sequence_ms=10,20,40,40,40`
- `retry_backoff_cap_ms=40`
- terminal decision markers:
  - `attempt_ceiling_reached`
  - `malformed_response_fail_fast`

| Provider error class | Classifier reason | Decision before max attempt | Decision at max attempt |
|---|---|---|---|
| `Timeout` | `timeout` | retry with deterministic bounded backoff | `attempt_ceiling_reached` |
| `Unavailable` | `unavailable` | retry with deterministic bounded backoff | `attempt_ceiling_reached` |
| `MalformedResponse` | non-transient | fail closed immediately | `malformed_response_fail_fast` |

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

### Signer Rotation and Quorum Preflight Contracts

Before nonce resolution and submit/finality calls, the live runtime now performs signer
rotation/quorum preflight checks and fails closed on contract violations:

- `runtime_signer_key_source_profile_pair_disallowed`:
  disallowed signer profile/key-source pair was requested.
- `runtime_signer_rotation_epoch_stale`:
  failover was active and signer rotation epoch did not increase.
- `runtime_signer_attestation_quorum_shortfall`:
  quorum-approved signer count was below the required approval threshold.
- `runtime_signer_quorum_linkage_violation`:
  active signer profile was not linked to the quorum-approved signer set.
- `runtime_signer_failover_attestation_required_approvals_insufficient`:
  failover signer rotation declared fewer than two required approvals.
- `runtime_signer_failover_attestation_previous_profile_not_approved`:
  failover signer rotation did not include the previous signer profile in approvals.

Execution status now emits deterministic signer readiness markers:

- `signer_previous_profile`
- `signer_failover_active`
- `signer_rotation_epoch`
- `signer_previous_rotation_epoch`
- `signer_quorum_linkage_contract_version=v1`
- `signer_quorum_required_approvals`
- `signer_quorum_approved_signers_count`
- `signer_quorum_profile_linked`
- `signer_quorum_satisfied`
- `signer_quorum_linked`

### Signer Key Decode Zeroization Guarantees

- `signer_decode_zeroization_contract_version=v1`
- Decode-path zeroization markers:
  - `signer_decode_zeroization_success_path=private_key_hex.zeroize+private_key_bytes.zeroize`
  - `signer_decode_zeroization_failure_path=decoded.zeroize+private_key_hex.zeroize`
- Redaction marker:
  - `signer_decode_error_redaction_policy=raw_private_key_value_never_emitted`
- Contract command:
  - `cargo test -p kamn-node --test signer_secret_hygiene_contract -- --nocapture`

### Signer Adapter API Boundary

- `signer_adapter_boundary_contract_status=active`
- `signer_adapter_boundary_contract_version=v1`
- `signer_adapter_module_path=crates/kamn-node/src/signer/signer_adapter.rs`
- `signer_adapter_reexport_owner=crates/kamn-node/src/signer.rs`
- `signer_adapter_owned_symbols_csv=KolmeForkSecp256k1SignerAdapter,decode_kolme_hex_bytes,encode_kolme_hex_lower,build_kolme_live_managed_signing_key,resolve_kolme_live_signer_private_key_env_name`
- Contract command:
  - `cargo test -p kamn-node --test signer_adapter_boundary_contract -- --nocapture`

## Validation Evidence

Primary tests:

- `main_tests::core_behavior_tests::functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles`
- `main_tests::cli_contract_tests::rejects_kolme_live_continuous_mode_without_tick_interval`
- `main_tests::cli_contract_tests::rejects_kolme_live_continuous_mode_without_max_ticks`

Command:

- `cargo test -p kamn-node -- rejects_kolme_live_continuous_mode_without_tick_interval rejects_kolme_live_continuous_mode_without_max_ticks functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles`
- `bash scripts/kolme/run_continuous_runtime_commit_contract_lane.sh`
- `bash scripts/kolme/validate_continuous_runtime_commit_live.sh`
