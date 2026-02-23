# Tasks: Issue #5842 - R56 Governance/Audit Gap Closure

- Issue: #5842
- Spec: `specs/5842/spec.md`
- Plan: `specs/5842/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing freeze/cfg-count/governance-coupling assertions that reproduce audit findings.
- [x] T2 (GREEN/Freeze): implement deterministic post-publication review-doc immutability enforcement.
- [x] T3 (GREEN/Expect Inventory): harden cfg(test) detection and reconcile production `expect()` inventory markers.
- [x] T4 (GREEN/Governance): add fail-closed structural-coupling policy enforcement for ratio target.
- [x] T5 (GREEN/Shell Surface): reduce shell surface measurably and update shell governance evidence markers.
- [x] T6 (Regression): run docs-contract + targeted checker suites (`review_r53`, `review_r50`, `test_check_no_production_expect`, shell ratio/ceiling checks); full `test_ci_tools` blocked by pre-existing `spec_archive_pointer_missing` baseline drift on `origin/main`.
- [x] T7 (Lifecycle): update milestone slice and set spec/plan/tasks status to Implemented/Completed.

## Tier Mapping
- Unit: cfg(test) parser semantics and marker parsing helpers.
- Functional: review docs-contract enforcement for freeze/governance/expect inventory.
- Integration: ci-tools fast/full checker execution with shell-surface policy checks.
- Conformance: AC-to-case coverage C-01..C-09.
- Regression: untracked spec-dir contamination and review-doc mutation guards.
- Performance: N/A (non-hot-path governance contract changes only).
