# Tasks: Issue #5815 - Close Residual S-03 sdk-direct Mutation Escapes

- Issue: #5815
- Spec: `specs/5815/spec.md`
- Plan: `specs/5815/plan.md`
- Status: Done

## Ordered Tasks
- [x] T1 (RED): add deterministic sdk-direct S-03 mismatch tests for query/list guard checks.
- [x] T2 (GREEN): extract and wire S-03 response-shape validation helper in sdk_direct.
- [x] T3 (Regression): run harness regression + docs contract lane.
- [x] T4 (Mutation): rerun `cargo mutants --in-diff` and confirm escaped sdk-direct mutants are caught.
- [x] T5 (Lifecycle): finalize spec/task statuses and issue process log updates.

## Tier Mapping
- Unit: helper mismatch guard tests.
- Functional: S-03 fail-closed path assertions.
- Integration: full harness suite.
- Regression: existing live scenario lanes remain green.
- Mutation: in-diff mutation gate for changed sdk-direct code.
