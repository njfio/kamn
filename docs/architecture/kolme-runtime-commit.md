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

## Validation Evidence

Primary tests:

- `main_tests::core_behavior_tests::functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles`
- `main_tests::cli_contract_tests::rejects_kolme_live_continuous_mode_without_tick_interval`
- `main_tests::cli_contract_tests::rejects_kolme_live_continuous_mode_without_max_ticks`

Command:

- `cargo test -p kamn-node -- rejects_kolme_live_continuous_mode_without_tick_interval rejects_kolme_live_continuous_mode_without_max_ticks functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles`
- `bash scripts/kolme/run_continuous_runtime_commit_contract_lane.sh`
- `bash scripts/kolme/validate_continuous_runtime_commit_live.sh`
