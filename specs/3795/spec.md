# Issue #3795 Spec

- Title: Subtask: enforce CI-fast exclusion and docs parity for transport resilience local-heavy lane
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Transport resilience local-heavy exclusion checks exist, but selector guard assertions and docs parity for the live transport fault-matrix reason/marker surface are not explicit enough to reliably fail closed on drift.

## Acceptance Criteria
- AC-1: CI exclusion test asserts selector-gated local-heavy workflow condition for Kolme heavy lanes.
- AC-2: CI exclusion test fails closed if live transport run-mode command leaks into `ci-tools` fast-mode block.
- AC-3: `docs/ci/strategy.md` transport-fault-matrix markers/taxonomy text stays synchronized with live policy checker output surface.
- AC-4: Unit/Functional/Integration/Regression evidence for exclusion + docs parity checks is present and passing.

## Scope
In scope:
- `scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/3795/{spec.md,plan.md,tasks.md}`

Out of scope:
- Enabling local-heavy run lanes in PR fast gate
- Changes to transport runtime behavior
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | workflow + exclusion policy script execution | fails closed if Kolme local-heavy selector gate drifts |
| C-02 | AC-2 | Functional | `ci-tools` fast-mode block scan in exclusion test | fails closed when live transport run-mode command leaks into fast mode |
| C-03 | AC-3 | Regression | strategy docs contract test | transport-fault-matrix reason-code/marker parity remains synchronized |
| C-04 | AC-4 | Integration | CI exclusion policy + docs contract suite | exclusion + docs parity checks pass together |
| C-05 | AC-4 | Regression | shell guardrails | shell LOC/ratio/ratchet checks remain green |

## Test Mapping
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_live_transport_fault_matrix_ci_exclusion_policy_contract_markers -- --exact`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3795.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3795.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3795.json`

## Success Metrics
- CI exclusion checks fail closed for selector/fast-mode leakage drift.
- Strategy docs parity is contract-tested for transport-fault-matrix markers and reason taxonomy content.
- Guardrails remain green after the change.
