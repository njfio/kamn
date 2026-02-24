# Tasks: Issue #5883 - Shell LOC Reduction Wave 2 (Selector Test Surface)

- Issue: #5883
- Spec: `specs/5883/spec.md`
- Plan: `specs/5883/plan.md`
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (Red): Capture pre-change shell LOC baseline.
- T2 (Green): Apply whitespace-only LOC reduction in `scripts/ci/test_select_targets.sh`.
- T3 (Regression): Run `scripts/ci/test_select_targets.sh`.
- T4 (Verify): Run shell LOC metric + `review_r53_docs_contract`.
