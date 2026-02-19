# Issue #3941 Tasks

- Issue: #3941
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add signer-source regression test for `unreachable!()` and run it to fail first.
- [x] T2 (Green): replace signer decode-failure `unreachable!()` branch with explicit typed assertion flow.
- [x] T3 (Regression): run scoped signer/startup panic-path tests and confirm deterministic behavior.
- [x] T4 (Verify): run formatting/lint checks for touched crate scope and finalize PR evidence.

## Tier Mapping
- Unit: typed decode-failure assertion path in signer tests.
- Functional: signer-source macro marker absence check.
- Integration: startup panic-path regression test plus signer tests in same crate run.
- Regression: explicit regression marker test for `unreachable!()` reintroduction.
- Performance: N/A (test-path hardening only; no runtime hot-path changes).
