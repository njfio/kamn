# Issue #3778 Spec

- Title: Task: add deterministic retry/backoff for shared Kolme HTTP transport client
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Shared Kolme HTTP transport paths must apply deterministic bounded retry/backoff for transient failures and fail closed for non-transient failures, with deterministic marker and taxonomy contracts across runtime behavior and documentation.

## Acceptance Criteria
- AC-1: Timeout/unavailable transient faults trigger bounded deterministic retry/backoff.
- AC-2: Non-transient failures remain fail-closed without retries.
- AC-3: Retry markers and reason taxonomy are deterministic and contract-tested.
- AC-4: Unit, Functional, Integration, Regression, and bounded Performance evidence is present and passing.

## Scope
In scope:
- Parent task closure over delivered child subtasks `#3790` and `#3791`
- Retry helper contracts and docs matrix in:
  - `crates/kamn-node/src/runtime_kolme_live.rs`
  - `docs/architecture/kolme-runtime-commit.md`
  - `crates/kamn-node/tests/kolme_runtime_commit_docs.rs`
- Integrated retry-loop marker contracts and validation commands in:
  - `crates/kamn-node/src/main_tests/runtime_tests.rs`
  - `docs/planning/kolme-devnet-ops.md`
  - `crates/kamn-node/tests/kolme_devnet_ops_docs.rs`
- `specs/3778/{spec.md,plan.md,tasks.md}`

Out of scope:
- Notification reconnect pacing (task `#3779`)
- Local-heavy lane additions (task `#3780`)
- Protocol semantic changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | retry classifier/backoff helper tests | Timeout/unavailable retry classes and bounded backoff remain deterministic |
| C-02 | AC-2 | Regression | malformed-response helper regression and terminal-decision retry exhaustion tests | Non-transient failures fail closed without retry |
| C-03 | AC-3 | Functional/Integration | structured retry marker runtime tests over transient transport faults | Retry/terminal markers include deterministic attempt/reason/decision fields |
| C-04 | AC-3 | Regression | docs-contract tests over runtime architecture and devnet planning docs | Retry policy matrix and validation marker/command drift fails closed |
| C-05 | AC-4 | Performance | bounded retry recovery budget test | Retry logic remains bounded and CI-fast compatible |

## Test Mapping
- `cargo test -p kamn-node runtime_kolme_live::tests::unit_retry_classifier_marks_transient_provider_errors -- --exact`
- `cargo test -p kamn-node runtime_kolme_live::tests::unit_retry_backoff_policy_is_deterministic_and_bounded -- --exact`
- `cargo test -p kamn-node runtime_kolme_live::tests::regression_retry_classifier_keeps_malformed_fail_fast -- --exact`
- `cargo test -p kamn-node --test kolme_runtime_commit_docs`
- `cargo test -p kamn-node main_tests::runtime_tests::functional_kolme_live_retry_emits_structured_retry_markers -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_kolme_live_submit_retry_exhaustion_emits_terminal_decision_marker -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::functional_kolme_live_finality_retry_exhaustion_emits_terminal_decision_marker -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::performance_runtime_kolme_live_retry_recovery_stays_within_budget -- --exact`
- `cargo test -p kamn-node --test kolme_devnet_ops_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3778.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3778.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3778.json`

## Success Metrics
- Shared transport retry/backoff behavior is deterministic and bounded for transient faults.
- Fail-fast behavior remains deterministic for non-transient failures.
- Retry marker/taxonomy docs remain fail-closed under contract tests.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
