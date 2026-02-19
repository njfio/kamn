# Issue #5015 Spec

- Title: Story: cross-cutting conformance harness and shell-surface budget neutrality
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Enforce PRD critical scenario conformance and ensure test orchestration remains
Rust-first with net-zero shell LOC growth against active CI ratio thresholds.
Story delivery is completed through child task `#5028`.

## Acceptance Criteria
- AC-1: PRD critical scenario conformance (`62..71`) is implemented with deterministic and fail-closed evaluation behavior.
- AC-2: Conformance gate reports only `Conformant` when all required scenarios pass under Rust-only orchestration policy.
- AC-3: Shell-surface impact remains neutral for story delivery (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level closure evidence for child deliverable `#5028`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented lifecycle status.
- PRD section 18.2 critical-scenario traceability and shell-neutral policy evidence.

Out of scope:
- Additional scenario classes outside PRD critical set `62..71`.
- Dependency/protocol/wire-format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_prd_critical_scenario_conformance` catalog + mutation guard tests | Required scenario set is deterministic and invalid/mutating inputs fail closed |
| C-02 | AC-2 | Conformance | Run all-pass and failure-policy tests for conformance evaluator | Only fully passing Rust-only execution reaches `Conformant`; failures/policy violations are blocked |
| C-03 | AC-3 | Regression | Inspect child task diff and shell guardrail evidence | `shell_loc_delta_actual = 0` and ratio posture improves/holds |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_prd_critical_scenario_conformance`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for the story child implementation because shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5015` closes with child task `#5028` merged and ACs mapped to deterministic passing tests.
- PRD critical-scenario conformance remains reproducible and fail-closed.
- Shell-to-Rust guardrail posture improves/holds with zero shell delta.
