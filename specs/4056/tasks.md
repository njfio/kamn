# Issue #4056 Tasks

- Issue: #4056
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Ordered Tasks
- [x] T1 (RED): add failing fixture/parser/scope-rejection/docs-parity tests for scope-policy contracts (`C-01`..`C-08`).
- [x] T2 (GREEN): implement fail-closed scope-policy checker and middleware wiring for protected routes (`C-01`..`C-04`).
- [x] T3 (GREEN): add deterministic scope-policy fixture matrix and parser/helper validation contracts (`C-05`, `C-06`).
- [x] T4 (GREEN): add scope-policy docs markers/remediation mappings in strategy + ops docs (`C-07`, `C-08`).
- [x] T5 (Regression): enforce source/docs taxonomy parity and remediation coverage in `ci_strategy_docs.rs` (`C-07`, `C-08`).
- [x] T6 (Verify): run targeted verification commands and scoped fmt/clippy/test gate (`C-09`).

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | scope-policy fixture parser metadata + row contract checks |
| Functional | fixture allow/deny rows vs route-required scope mapping |
| Integration | missing/invalid/mismatched/matching scope behavior on protected routes |
| Regression | source/docs taxonomy parity + remediation marker coverage |
| Performance | N/A (low-cost checker path already bounded by middleware flow) |
