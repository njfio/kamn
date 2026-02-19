# Issue #3955 Tasks

- Issue: #3955
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing tests for managed key-source adapter provenance marker emission and mismatch fail-closed behavior.
- [x] T2 (Green): implement managed key-source adapter abstraction with deterministic provenance marker output.
- [x] T3 (Green): consume provenance marker in signer selection/profile parity checks.
- [x] T4 (Green): update `docs/ops/configuration.md` with managed adapter provenance mapping marker and add docs-contract assertion.
- [x] T5 (Regression): run targeted signer/docs tests and `cargo fmt --check`.
- [ ] T6 (Verify): update issue process log, TDD checklist, and labels for `#3955`.

## Tier Mapping
- Unit: adapter provenance marker emission/shape checks.
- Functional: managed signer flow marker parity pass case.
- Integration: runtime managed signer payload path through adapter.
- Regression: mismatch fail-closed reason-code stability.
- Performance: N/A (no algorithmic expansion; existing bounded signer path reused).
