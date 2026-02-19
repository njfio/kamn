# Issue #3783 Spec

- Title: Subtask: enforce tracing event taxonomy drift and docs-contract parity
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-14-observability-standardization-and-route-contract-hardening/index.md

## Problem Statement
Tracing/observability taxonomy vocabulary can drift unless `docs/observability/contracts.md` and runtime source markers are pinned with fail-closed tests.

## Acceptance Criteria
- AC-1: Docs define tracing event taxonomy version and required vocabulary for execution/mode/route/reason/checkpoint markers.
- AC-2: Docs-contract tests fail closed on missing/renamed taxonomy markers.
- AC-3: Runtime source parity checks ensure required markers are present in core observability/tracing surfaces.
- AC-4: Targeted tests and lint gates pass.

## Scope
In scope:
- `docs/observability/contracts.md`
- `crates/kamn-node/tests/observability_contracts_docs.rs` (new)
- `specs/3783/{spec.md,plan.md,tasks.md}`

Out of scope:
- New runtime event families
- Logging framework migration
- Shell/workflow changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Observability contracts doc | Taxonomy version and required field/event markers are present |
| C-02 | AC-2 | Regression | Docs-contract tests | Missing markers fail closed |
| C-03 | AC-3 | Integration | Doc markers vs runtime source files | Required marker vocabulary exists in both docs and code |
| C-04 | AC-4 | Regression | fmt/clippy/tests/shell guardrails | All green |

## Test Mapping
- `cargo test -p kamn-node --test observability_contracts_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3783.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3783.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3783.json`

## Success Metrics
- `docs/observability/contracts.md` taxonomy markers are explicit and machine-checked.
- Drift in required vocabulary is detected by deterministic tests.
