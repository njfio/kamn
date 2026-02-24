# Tasks: Issue #5936 - Task: Wire Data Layer M0-M11 into service runtime paths with staged activation

- Issue: #5936
- Spec: `specs/5936/spec.md`
- Plan: `specs/5936/plan.md`
- Status: Draft
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED / Conformance): derive failing tests from all C-xx conformance cases before implementation.
- T2 (GREEN / Implementation): implement in-scope behavior changes with minimal diff.
- T3 (Refactor): improve structure/readability while preserving green tests.
- T4 (Regression): run targeted module tests plus issue-specific regression suites.
- T5 (Verify): run cargo fmt --check, strict clippy for touched crates, and scoped tests to close ACs.
- T6 (Process): update docs/spec status and attach AC evidence in PR + issue closure.
