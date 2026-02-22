# Tasks: Issue #5771 — Enforce Pre-Merge Workspace Cargo-Test Gate

- Issue: #5771
- Spec: `specs/5771/spec.md`
- Plan: `specs/5771/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks

1. T1 (Functional/Regression): add/adjust failing contract assertions in `ci_fast_gate_workspace_premerge_contract` for bounded-retry workflow markers and CI strategy docs parity.
2. T2 (Implementation): add explicit pre-merge workspace gate marker block in `docs/ci/strategy.md` with canonical command string.
3. T3 (Green verification): run `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract`.
4. T4 (Quality gates): run `cargo fmt --all --check` and targeted clippy lane for `kamn-core` tests.
5. T5 (Lifecycle closure): update milestone index and issue process log; set spec status to `Implemented` after validation.

## AC/Tier Mapping
- AC-1/AC-2: T1, T3 (Functional)
- AC-3: T2, T3 (Functional)
- AC-4: T1, T3 (Regression)
