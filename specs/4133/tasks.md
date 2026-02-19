# Issue #4133 Tasks

- Issue: #4133
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Ordered Tasks
- [x] T1 (Child #4139): add RED->GREEN corpus drift and parser taxonomy contracts.
- [x] T2 (Child #4140): add deterministic seed provenance and bounded budget marker contracts.
- [x] T3 (Refactor): keep contract markers deterministic and avoid shell wrapper growth.
- [x] T4 (Regression): run targeted contract suites and lint/format gates.
- [x] T5 (Closeout): publish parent task spec/plan/tasks and close with explicit markers.

## Test Tier Mapping
| Tier | Coverage |
|---|---|
| Unit | marker presence assertions in cargo-fuzz contract tests |
| Functional | metadata/docs marker contracts for parser fuzz governance |
| Conformance | CI strategy marker parity checks |
| Regression | drift/tamper fail-closed assertions for taxonomy and provenance |
