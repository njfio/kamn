# Issue #5184 Spec

- Title: Task: consolidate doc-contract tests into data-driven harness to reduce file sprawl
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Problem Statement
R42 review findings require explicit closure through spec-driven implementation and verification artifacts.

## Scope
In:
- Deliver issue-scoped behavior and verification updates.
- Preserve shell-surface constraints and ratio guardrails.

Out:
- Unrelated feature work outside R42 closure scope.

## Acceptance Criteria
- AC-1: Issue-specific behavior updates are implemented or explicitly sequenced when larger than this cycle.
- AC-2: Conformance coverage is mapped to concrete tests and/or policy checks.
- AC-3: Shell-surface impact is neutral or accompanied by mitigation tracking.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Issue-specific implementation path | Target behavior is satisfied with deterministic outcome |
| C-02 | AC-2 | Conformance | Issue-specific contract checks | Contract and marker parity hold |
| C-03 | AC-3 | Regression | Shell/Rust surface measurements | Guardrail remains compliant or has linked mitigation |

## Test Mapping
- C-01 -> targeted unit/integration/functional tests for the issue scope
- C-02 -> conformance/docs-contract tests and/or policy checks
- C-03 -> regression checks for shell-surface ratio governance

## Success Metrics
- All mapped ACs verified with explicit command evidence in PR.
