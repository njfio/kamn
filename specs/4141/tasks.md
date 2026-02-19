# Issue #4141 Tasks

- Issue: #4141
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Ordered Tasks
- T1 (RED/Regression): validate checker tamper paths fail closed with deterministic reason markers.
- T2 (GREEN): validate checker pass path emits expected lineage and boundary markers.
- T3 (Integration): validate selector routing keeps local-heavy lane deterministic and default-excluded.
- T4 (Verify): capture bounded CI smoke runtime contract evidence.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | checker field validations in policy test harness |
| Functional | runtime contract lane smoke execution |
| Integration | `select_targets` routing matrix for lane exclusion |
| Regression | tampered marker-lineage fail-closed tests |
