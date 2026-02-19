# Issue #3785 Spec

- Title: Subtask: add CI exclusion and command-surface drift guards for observability local-heavy lane
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Observability local-heavy run-mode lanes must never leak into fast CI defaults. Exclusion rules and command-surface docs/tests need deterministic contract coverage so drift fails closed.

## Acceptance Criteria
- AC-1: CI exclusion contracts fail closed when unified API-observability local-heavy run-mode commands leak into fast workflow or ci-tools fast mode.
- AC-2: Command-surface contract entries and docs markers for unified API-observability local-heavy exclusion remain synchronized.
- AC-3: Unit, Functional, Integration, and Regression evidence is present and passing (or explicitly justified).

## Scope
In scope:
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh` (verification target, no behavior expansion)
- `specs/3785/{spec.md,plan.md,tasks.md}`

Out of scope:
- Adding new local-heavy runtime lanes
- Expanding fast-gate to execute local-heavy run-mode flows
- New shell tooling surfaces

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh` | Fails if run-mode lane appears in `.github/workflows/ci-fast-gate.yml` or ci-tools fast block |
| C-02 | AC-2 | Integration | `docs/ci/strategy.md` + docs-contract assertions | Strategy doc includes unified local-heavy exclusion command-surface markers |
| C-03 | AC-2 | Regression | `cargo test -p kamn-core --test ci_strategy_docs` | Drift in unified local-heavy command markers fails closed |
| C-04 | AC-3 | Regression | lint + shell guardrails | fmt/clippy/tests/guardrails all green |

## Test Mapping
- `bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_make_and_demo_scope_contract_rules -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3785.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3785.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3785.json`

## Success Metrics
- Unified API-observability local-heavy exclusion command is documented in strategy contracts and enforced by tests.
- No shell LOC growth is introduced for this issue (`shell_loc_delta_actual=0` target).
