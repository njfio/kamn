# Tasks: Issue #5840 - cfg(test) Parsing Hardening for `expect()` Inventory

- Issue: #5840
- Spec: `specs/5840/spec.md`
- Plan: `specs/5840/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED): add checker regression fixture for brace-heavy `#[cfg(test)]` module leakage.
- [x] T2 (GREEN): harden Python cfg(test)-item skipping scanner.
- [x] T3 (GREEN): harden Rust docs-contract cfg(test)-item skipping helper.
- [x] T4 (GREEN): align R55 formula marker text with implemented semantics.
- [x] T5 (Regression): run mapped checker + docs-contract lanes and fix regressions.

## Tier Mapping
- Unit: scanner fixture around cfg(test) item parsing.
- Conformance: R55 marker/inventory invariants in review docs-contract lane.
- Integration: checker harness + docs-contract lane pass together.
- Regression: preserve existing reason-taxonomy and violation detection fixtures.
- Performance: N/A (test/checker logic only).
