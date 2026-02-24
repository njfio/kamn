# Tasks: Issue #5881 - Shell LOC Reduction Wave (No Behavior Change)

- Issue: #5881
- Spec: `specs/5881/spec.md`
- Plan: `specs/5881/plan.md`
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (Red, Integration): Capture pre-change shell LOC baseline.
- T2 (Green, Functional): Apply whitespace-only LOC reduction in `scripts/ci/select_targets.sh`.
- T3 (Regression): Run `scripts/ci/test_select_targets.sh`.
- T4 (Verify): Run shell LOC metric check and `review_r53_docs_contract`.
