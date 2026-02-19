# Issue #3775 Tasks

- Issue: #3775
- Status: Done

## Ordered Tasks
- [x] T1 (Verify child closure): confirm `#3782` and `#3783` merged/closed.
- [x] T2 (Specify): add `specs/3775/spec.md` AC/conformance/test mapping.
- [x] T3 (Plan): add `specs/3775/plan.md` approach and risk mapping.
- [x] T4 (Regression): run mapped tracing tests + fmt/clippy + shell guardrails.
- [x] T5 (Verify): merge closure PR and close parent issue with DoD markers.

## Tier Mapping
- Unit: log-config and taxonomy vocabulary assertions.
- Functional: runtime event payload behavior checks.
- Integration: runtime-mode startup tracing marker checks.
- Regression: invalid-config fail-closed + taxonomy drift marker stability.
- Performance: N/A (parent closure/spec traceability increment only).
