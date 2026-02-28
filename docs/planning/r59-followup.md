# R59 Follow-up

## Issue #6247: Critical-path Coverage Threshold Hardening

Threshold source of truth: `.ci/critical-path-coverage-thresholds.json`

Measured coverage reference (current probe): `ci-critical-path-coverage-policy.json`

| Target | Previous line/function min | Updated line/function min | Measured line/function | Rationale |
|---|---:|---:|---:|---|
| `crates/kamn-core/src/direct_message_crypto.rs` | 60 / 60 | 80 / 80 | 96.30 / 90.91 | Crypto path has strong test depth; threshold now reflects security-critical expectation. |
| `crates/kamn-core/src/group_channel_crypto.rs` | 50 / 50 | 55 / 55 | 58.63 / 56.60 | Ratchet above prior baseline while preserving variance headroom. |
| `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs` | 27 / 22 | 37 / 32 | 40.43 / 37.70 | Transport error mapping is operationally critical; minima moved toward measured envelope. |
| `crates/kamn-node/src/runtime_orchestration.rs` | 15 / 18 | 20 / 28 | 25.50 / 31.58 | Increase orchestration guardrail with function threshold aligned to current exercised control paths. |
| `crates/kamn-node/src/service_api_endpoint.rs` | 45 / 24 | 46 / 25 | 47.80 / 25.64 | Incremental ratchet to avoid no-op threshold while retaining determinism. |
| `crates/kamn-node/src/signer.rs` | 4.5 / 8.5 | 15 / 19 | 28.39 / 38.46 | Large signer uplift eliminates under-defended minima and makes threshold materially meaningful. |

Notes:
- Thresholds were raised for all currently gated targets.
- Probe set was expanded for `http_transport`, `runtime_orchestration`, and `signer` paths to keep raised thresholds backed by deterministic exercised behavior.
- Checker behavior remains fail-closed via `scripts/ci/check_critical_path_coverage.py`.
- Follow-up hardening should continue by increasing test depth and ratcheting minima in controlled increments.

## Issue #6249: kamn-core Wave2 Shim Retirement

### Shim Inventory And Decisions

| Shim module (`kamn-core`) | Extracted owner crate | Decision (R59) | Workspace migration status | Removal target |
|---|---|---|---|---|
| `src/anti_spam.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | `kamn-node` service-api anti-spam path now imports extracted crate directly | R61 |
| `src/fairness_policy.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/quota_policy.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/message_delivery_guards.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/retention_engine.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/watchdog.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/live_probe_matrix.rs` | `kamn-live-probe-matrix` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/cross_chain_receipt.rs` | `kamn-bridges` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |

### Notes

- Shim modules remain only for transitional compatibility and are annotated as deprecated.
- `kamn-core` root exports for the migrated surfaces no longer depend on shim modules; they re-export directly from extracted crates.
- New in-repo imports should target extracted crates directly.

## Issue #6250 — Shell/Workflow/Template Surface Ratio Governance

### Objective
Close the remaining shell-surface follow-up by keeping shell/rust ratio below 1.0, tightening deterministic CI ratio guardrails, and migrating one PR-critical CI tools regression lane from shell wrapper execution to Rust test coverage.

### Delivered
- Migrated the shell-rust guardrail regression lane from shell wrapper execution to Rust:
  - Added `cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract` to fast-mode CI tool regression lane (`scripts/ci/test_ci_tools.sh`).
  - Added `crates/kamn-core/tests/ci_shell_rust_ratio_guardrail_contract.rs` with pass/warn/fail/validation checker contracts.
  - Removed retired shell wrapper regression script: `scripts/ci/test_check_shell_rust_ratio_guardrail.sh`.
- Tightened ratio thresholds in `.ci/shell-rust-ratio-guardrail.env`:
  - `WARN_SHELL_RUST_RATIO_MAX`: `0.95 -> 0.75`
  - `FAIL_SHELL_RUST_RATIO_MAX`: `1.00 -> 0.95`
- Updated selector command-surface contract and CI strategy docs:
  - `scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `docs/ci/strategy.md`

### Measured Before/After
- Baseline (pre-change; command: `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json /tmp/shell-rust-ratio-before-6250.json`):
  - `shell_line_total=121119`
  - `rust_line_total=237683`
  - `shell_to_rust_ratio=0.509582`
  - `warn_shell_rust_ratio_max=0.95`
  - `fail_shell_rust_ratio_max=1.00`
- Post-change (command: `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json /tmp/shell-rust-ratio-after-6250.json`):
  - `shell_line_total=120954`
  - `rust_line_total=237944`
  - `shell_to_rust_ratio=0.50833`
  - `warn_shell_rust_ratio_max=0.75`
  - `fail_shell_rust_ratio_max=0.95`
  - `shell_loc_delta_actual=-165`
  - `rust_loc_delta_actual=261`
  - `shell_to_rust_ratio_delta_actual=-0.001252`
  - `shell_surface_ratio_target_status=improved`
  - `shell_surface_mitigation_issue=None`

### Verification Commands
- `cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract -- --nocapture`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json /tmp/shell-rust-ratio-after-6250.json`
- `bash scripts/ci/check_shell_surface_threshold_ratchet.sh --hard-ceiling-file .ci/shell-loc-hard-ceiling.env --ratio-threshold-file .ci/shell-rust-ratio-guardrail.env --ratchet-exception-file .ci/shell-surface-threshold-ratchet-exception.json --output-json /tmp/shell-surface-threshold-ratchet-6250.json`

### Outcome
- Ratio remains safely below 1.0 with deterministic fast-gate enforcement retained.
- One PR-critical CI tools lane has been moved from shell regression wrapper execution to Rust contract coverage.
