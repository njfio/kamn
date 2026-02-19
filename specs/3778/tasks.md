# Issue #3778 Tasks

- Issue: #3778
- Status: In Progress

## Ordered Tasks
- [x] T1 (Specify/Plan): add parent task spec/plan/tasks artifacts consolidating `#3790` and `#3791`.
- [x] T2 (Verify): execute parent-level helper, integration, regression, docs-contract, and performance checks.
- [x] T3 (Regression): run fmt/clippy and shell guardrails.
- [ ] T4 (Verify): open/merge closure PR and close `#3778` with DoD markers.

## Tier Mapping
- Unit: retry helper classifier/backoff contracts.
- Functional: transient transport retry marker behavior.
- Integration: mock transport retry loop path with deterministic attempt markers.
- Regression: fail-fast terminal marker behavior + docs drift checks.
- Performance: bounded retry recovery budget test.
