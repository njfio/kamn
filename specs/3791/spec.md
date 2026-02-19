# Issue #3791 Spec

- Title: Subtask: integrate HTTP transport retry loop with deterministic attempt markers
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
The shared Kolme HTTP transport retry helpers must remain verifiably integrated in runtime execution paths, with deterministic retry-attempt and terminal-decision markers that fail closed on drift.

## Acceptance Criteria
- AC-1: Shared HTTP transport execution applies bounded retry/backoff for transient timeout/unavailable failures.
- AC-2: Terminal retry failures emit deterministic terminal decision markers and reason codes.
- AC-3: Runtime planning docs include deterministic transport retry validation commands and marker contracts.
- AC-4: Unit, Functional, Integration, Regression, and bounded Performance evidence is present and passing (or explicitly justified N/A).

## Scope
In scope:
- `crates/kamn-node/src/runtime_kolme_live.rs` integrated retry loop behavior (verification target)
- `crates/kamn-node/src/main_tests/runtime_tests.rs` retry-loop functional/integration/regression/performance tests (verification target)
- `docs/planning/kolme-devnet-ops.md` retry validation command/marker contract surface
- `crates/kamn-node/tests/kolme_devnet_ops_docs.rs` docs-contract drift assertions
- `specs/3791/{spec.md,plan.md,tasks.md}`

Out of scope:
- New retry policies for unrelated transports
- Protocol/wire-format changes
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional | runtime retry helper + integrated retry runtime test paths | Timeout/unavailable failures retry with bounded attempts/backoff |
| C-02 | AC-2 | Functional/Regression | submit/finality retry-exhaustion runtime tests | Terminal markers emit deterministic `attempt_ceiling_reached` / `malformed_response_fail_fast` decisions |
| C-03 | AC-2 | Integration | structured retry marker integration test over mock transport faults | Retry markers include deterministic attempt/reason/decision/jitter/backoff fields |
| C-04 | AC-3 | Regression | docs-contract assertions over `docs/planning/kolme-devnet-ops.md` | Retry validation command and marker contracts fail closed on docs drift |
| C-05 | AC-4 | Performance | bounded retry performance budget test | Retry recovery remains within explicit bounded time budget |

## Test Mapping
- `cargo test -p kamn-node main_tests::runtime_tests::functional_kolme_live_retry_emits_structured_retry_markers -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_kolme_live_submit_retry_exhaustion_emits_terminal_decision_marker -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::functional_kolme_live_finality_retry_exhaustion_emits_terminal_decision_marker -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::performance_runtime_kolme_live_retry_recovery_stays_within_budget -- --exact`
- `cargo test -p kamn-node --test kolme_devnet_ops_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3791.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3791.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3791.json`

## Success Metrics
- Retry loop integration remains deterministic under transient failure and retry exhaustion.
- Docs/planning retry validation commands/markers are contract-pinned and fail closed on drift.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
