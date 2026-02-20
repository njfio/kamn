# Issue #4057 Tasks

- Issue: #4057
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Ordered Tasks
- [x] T1 (RED): add failing route authz matrix and docs parity/remediation contract tests (`C-01`..`C-05`).
- [x] T2 (GREEN): implement request-path matrix checks in `kamn-node` tests and stabilize fail-closed assertions (`C-01`..`C-03`).
- [x] T3 (GREEN): add strategy/ops docs authz route parity markers and remediation mappings (`C-04`, `C-05`).
- [x] T4 (Regression): enforce source-taxonomy/docs parity and per-reason remediation checks in `ci_strategy_docs.rs` (`C-04`, `C-05`).
- [x] T5 (Verify): run targeted verification commands and set spec status to `Implemented` (`C-06`).

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | route matrix decision checks (`route_requires_auth`) |
| Functional | public-route unauthenticated reachability checks |
| Integration | protected-route fail-closed auth checks + docs/source parity |
| Regression | per-reason remediation marker coverage + drift assertions |
