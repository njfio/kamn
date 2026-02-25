# Tasks: Issue #5935 - Task: Eliminate high-impact code duplication classes (JSON parser, helper utilities, digest/path helpers)

- Issue: #5935
- Spec: `specs/5935/spec.md`
- Plan: `specs/5935/plan.md`
- Status: Completed
- Last Updated: 2026-02-25

## Ordered Tasks
- [x] T1 (RED / Conformance): derive failing tests from all C-xx conformance cases before implementation.
- [x] T2 (GREEN / Implementation): implement in-scope behavior changes with minimal diff.
- [x] T3 (Refactor): improve structure/readability while preserving green tests.
- [x] T4 (Regression): run targeted module tests plus issue-specific regression suites.
- [x] T5 (Verify): run cargo fmt --check, strict clippy for touched crates, and scoped tests to close ACs.
- [x] T6 (Process): update docs/spec status and attach AC evidence in PR + issue closure.
