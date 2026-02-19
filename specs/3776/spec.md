# Issue #3776 Spec

- Title: Task: add cost-bounded local-heavy observability lane governance contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Local-heavy observability validation must provide deterministic evidence while preserving fast-gate CI cost boundaries and explicit exclusion semantics.

## Acceptance Criteria
- AC-1: Local-heavy observability lane emits deterministic summary and policy artifacts.
- AC-2: CI contracts fail closed when local-heavy observability run-mode leaks into fast-gate/ci-tools fast mode.
- AC-3: Command-surface/docs contracts remain synchronized with implemented lane scripts.
- AC-4: Functional, Integration, and Regression evidence is present and passing.

## Scope
In scope:
- Child subtask outcomes: `#3784`, `#3785`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- Local observability runtime and CI exclusion policy contract tests
- `specs/3776/{spec.md,plan.md,tasks.md}`

Out of scope:
- Running local-heavy lane in default PR fast-gate
- New runtime observability capabilities outside existing lane contracts

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | local observability lane summary/policy/contract tests | Deterministic schema/taxonomy markers pass |
| C-02 | AC-2 | Functional | unified local-heavy CI exclusion policy test | Leakage into fast workflow/ci-tools fast mode fails closed |
| C-03 | AC-3 | Integration | docs-contract assertions in `ci_strategy_docs` | Strategy markers remain synchronized with command surface |
| C-04 | AC-4 | Regression | lint + shell guardrails | All gates green |

## Test Mapping
- `bash scripts/runtime/test_validate_local_observability_scrape_live.sh`
- `bash scripts/runtime/test_check_local_observability_scrape_live_policy.sh`
- `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`
- `bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_runtime_local_observability_scrape_contract_lane_ci_mode_markers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_make_and_demo_scope_contract_rules -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3776.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3776.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3776.json`

## Success Metrics
- Local-heavy observability lane remains local-only with deterministic artifact and policy markers.
- CI fast-gate leakage protection stays fail-closed.
- Shell LOC does not increase for this consolidation closure (`shell_loc_delta_actual=0`).
