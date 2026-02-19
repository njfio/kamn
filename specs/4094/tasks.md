# Issue #4094 Tasks

- Issue: #4094
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Ordered Tasks
- T1 (RED): confirm missing `#4094` ops markers fails deterministic lookup checks.
- T2 (GREEN): add `#4094` overload profile marker section in `docs/ops/configuration.md` and make checks pass.
- T3 (Verify): run targeted shell test + marker lookup checks covering schema/marker determinism.
- T4 (Close): set spec status to `Implemented`; update issue process log and closure evidence.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | runner contract test script validates schema keys and deterministic pass/fail marker mapping |
| Functional | ops docs marker presence + guard-command references |
| Integration | shell runner contract + ops marker composition |
| Regression | marker drift in ops docs fails deterministic marker lookup checks |
| Performance | N/A (no runner/workflow runtime-path change); existing bounded budget checks retained |
