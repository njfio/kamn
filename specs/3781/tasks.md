# Issue #3781 Tasks

- Issue: #3781
- Status: Done

## Ordered Tasks
- [x] T1 (Verify child closure): confirm subtasks `#3789` and `#3788` merged and closed.
- [x] T2 (Specify): add `specs/3781/spec.md` with AC and conformance mapping.
- [x] T3 (Plan): add `specs/3781/plan.md` with risks/mitigations and verification approach.
- [x] T4 (Regression): run mapped observability tests + fmt/clippy + shell guardrails.
- [x] T5 (Verify): merge closure PR and close parent issue with DoD markers.

## Tier Mapping
- Unit: docs-contract assertions for observability parity/taxonomy markers.
- Functional: route parity behavior assertions.
- Integration: baseline + secure-mode live route probes.
- Regression: secure-mode fail-closed matrix + drift marker stability checks.
- Performance: N/A (closure/spec traceability increment only).
