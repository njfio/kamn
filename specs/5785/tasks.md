# Tasks: Issue #5785 — Finalize R53 Milestone Closure Markers and Docs-Contract Guard

- Issue: #5785
- Spec: `specs/5785/spec.md`
- Plan: `specs/5785/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (RED/Functional): add R53 milestone closure assertion(s) in `docs_contract_release_group.rs` and run targeted test expecting failure.
2. T2 (GREEN/Implementation): update `specs/milestones/r53-e2e-scenario-execution-activation/index.md` to closed-state markers.
3. T3 (Regression/Integration): run targeted and group-level docs-contract tests.
4. T4 (Quality): run `cargo fmt --all --check` and targeted clippy lane for e2e harness tests.
5. T5 (Lifecycle): update `r52` milestone index with completed slice entry and preserve spec-dir cap.

## AC/Tier Mapping
- AC-1: T1, T2 (Functional)
- AC-2: T1, T3 (Regression)
- AC-3: T2, T3 (Integration)
