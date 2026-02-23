# Tasks: Issue #5806 - Branch Budget Cleanup Tranche

- Issue: #5806
- Spec: `specs/5806/spec.md`
- Plan: `specs/5806/plan.md`
- Status: Done

## Ordered Tasks
- [x] T1 (Conformance): capture pre-cleanup remote branch inventory/count and merged-lineage candidates.
- [x] T2 (GREEN): reconcile merged branch cleanup for two stale merged refs and exclude `main`/HEAD.
- [x] T3 (Regression): verify post-cleanup count target (`<=50`) and run docs-contract regression lane.
- [x] T4 (Closeout): update milestone index and finalize issue/PR lifecycle markers.

## Tier Mapping
- Conformance: pre/post count and deletion evidence.
- Functional: merged-lineage-only deletion execution.
- Regression: docs-contract sanity lane and lifecycle closeout evidence.
- Performance: N/A (no performance-sensitive code path).
