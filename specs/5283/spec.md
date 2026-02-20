# Issue #5283 Spec

- Title: Task: validate realtime backpressure guardrails and finalize presence gateway ops docs
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-5 realtime integration is functionally live, but story `#5252` still needs explicit bounded-load validation for M9 backpressure and anti-spam guardrails plus operator-facing configuration/troubleshooting documentation for websocket presence mode.

## Scope
In:
- Add deterministic bounded-load integration coverage for websocket delivery behavior when M9 backpressure and anti-spam decisions escalate.
- Add regression coverage for ordered delivery and duplicate suppression under pressured flows.
- Update `docs/ops/configuration.md` with realtime presence-mode headers, owner-scope requirements, and fail-closed reason-marker troubleshooting.

Out:
- Multi-node websocket fan-out replication.
- External push delivery integrations.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Bounded-load websocket tests demonstrate deterministic backpressure escalation handling with stable outcomes.
- AC-2: Anti-spam rejection behavior is validated in realtime flow with stable reason markers.
- AC-3: Ordering and duplicate-suppression regressions are covered and passing for the scoped realtime path.
- AC-4: `docs/ops/configuration.md` documents presence-mode headers, required owner-scope inputs, and fail-closed troubleshooting markers.
- AC-5: `cargo fmt --check`, strict `clippy`, and targeted test commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | bounded-load websocket flow with elevated queue pressure | deterministic backpressure decision path and stable emitted behavior |
| C-02 | AC-1 | Performance | bounded perf-smoke run for realtime gateway slice | no contract regression in pressure lane behavior |
| C-03 | AC-2 | Regression | anti-spam threshold breach in realtime path | fail-closed rejection with stable anti-spam reason marker |
| C-04 | AC-3 | Regression | duplicate delivery candidate in pressured flow | duplicate suppression preserved |
| C-05 | AC-3 | Regression | ordered sequence under pressure | delivery order guarantee preserved for scoped sequence |
| C-06 | AC-4 | Functional | operator reads configuration guide | required headers and fail-closed reason codes are documented |
| C-07 | AC-5 | Verification | fmt/clippy + targeted tests | all required checks pass |

## Success Metrics
- Story `#5252` acceptance criteria for guardrail enforcement and operator documentation are verifiably closed.
- No shell-surface growth while expanding realtime validation depth.
