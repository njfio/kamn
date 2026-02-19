# Issue #3784 Spec

- Title: Subtask: implement local-heavy observability lane artifact and policy contracts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Local observability lane summary/policy artifacts must remain deterministic with stable schema and taxonomy markers, and docs-contract coverage must fail closed on drift.

## Acceptance Criteria
- AC-1: Local observability lane declares deterministic summary/policy/contract-lane artifact schema markers.
- AC-2: Contract tests fail closed on marker/taxonomy drift for local observability lane artifacts.
- AC-3: Functional, Integration, and Regression evidence is present and passing (Unit/Performance N/A justified for this docs-contract increment).

## Scope
In scope:
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- Runtime local-observability lane/policy/contract test scripts as verification targets (no new shell scripts)
- `specs/3784/{spec.md,plan.md,tasks.md}`

Out of scope:
- New runtime observability behaviors
- CI fast-path expansion for local-heavy run mode
- Dependency or protocol changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | `docs/ci/strategy.md` section for local observability lane | Explicit schema markers for summary/policy/contract artifacts are present |
| C-02 | AC-2 | Functional | `scripts/runtime/test_validate_local_observability_scrape_live.sh` | Summary/policy artifact schema/taxonomy checks pass fail-closed |
| C-03 | AC-2 | Functional | `scripts/runtime/test_check_local_observability_scrape_live_policy.sh` and `scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh` | Policy and contract-lane marker drift checks pass |
| C-04 | AC-3 | Regression | docs-contract/lint/shell guardrails | All gates green |

## Test Mapping
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_runtime_local_observability_scrape_contract_lane_ci_mode_markers -- --exact`
- `scripts/runtime/test_validate_local_observability_scrape_live.sh`
- `scripts/runtime/test_check_local_observability_scrape_live_policy.sh`
- `scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3784.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3784.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3784.json`

## Success Metrics
- Local observability artifact schema markers are explicit in strategy docs and pinned by tests.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
