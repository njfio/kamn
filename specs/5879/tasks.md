# Tasks: Issue #5879 - Runtime-Wide Production Panic-Path Audit + Env Fallback Remediation

- Issue: #5879
- Spec: `specs/5879/spec.md`
- Plan: `specs/5879/plan.md`
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (Red, Conformance): Capture failing runtime-wide audit evidence showing unsafe fallback violations in runtime files.
- T2 (Green, Functional): Expand checker default roots to runtime crate set.
- T3 (Green, Regression): Replace targeted runtime `env::var(...).unwrap_or(_else)` callsites with explicit match fallback mapping.
- T4 (Verify): Run checker wrapper/tests and targeted Rust tests; confirm AC coverage.
