# Issue #5037 Spec

- Title: Subtask: M8 crypto-shred and retention-policy legal-hold conformance suite
- Status: Draft
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver the highest-risk validation/conformance sub-scope for parent task

## Acceptance Criteria
- AC-1: Scope for issue #5037 is decomposed into explicit implementation/integration/validation outcomes with deterministic test evidence.
- AC-2: The issue maps to PRD sections and conformance scenarios with clear test commands and result expectations.
- AC-3: Shell-surface impact remains neutral by default (net shell LOC delta <= 0) unless explicitly waived with mitigation issue linkage.

## Scope
In scope:
- Issue-specific delivery for Subtask: M8 crypto-shred and retention-policy legal-hold conformance suite.
- Contract-driven lifecycle artifacts (`spec.md`, `plan.md`, `tasks.md`).
- Test-tier mapping and conformance evidence capture.

Out of scope:
- Unapproved dependency/protocol changes.
- Work outside the parent milestone scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Execute issue task plan for #5037 | Planned implementation/integration steps are completed with evidence |
| C-02 | AC-2 | Conformance | Run mapped test commands for #5037 | All mapped conformance checks pass and produce deterministic markers |
| C-03 | AC-3 | Regression | Run shell-surface and ratio governance checks | No net shell-surface regression without waiver |

## Test Mapping
- `cargo test -p kamn-core` (scoped by issue-specific suites)
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh` (when shell/python/workflow surface is touched)
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh` (when shell/python/workflow surface is touched)

## Success Metrics
- Issue #5037 reaches `Status: Implemented` with ACs mapped to passing conformance evidence.
- Shell-to-Rust ratio guardrails remain within thresholds.
