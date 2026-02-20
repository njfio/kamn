# Issue #4000 Spec

- Title: Subtask: create benchmark fixture matrix for runtime-signing-transport hot paths
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement
The current performance smoke report generator emits lane-level static values only. It does not use a deterministic fixture matrix that covers runtime/signing/transport hot-path workloads, and it has no explicit fixture-schema validation contract.

## Acceptance Criteria
- AC-1: A deterministic fixture matrix exists for runtime/signing/transport hot paths and includes smoke/deep workload values.
- AC-2: The report generator supports workload selection and validates fixture schema fail-closed.
- AC-3: Fixture-schema and workload-selection behavior is covered by unit/functional/integration/regression shell tests.
- AC-4: Docs include a fixture-to-SLO mapping table for the new workload matrix.

## Scope
In scope:
- Add fixture matrix file under `fixtures/ci/`.
- Extend `scripts/ci/generate_performance_smoke_report.sh` to read workload-specific metrics from the fixture matrix with schema validation.
- Extend `scripts/ci/test_generate_performance_smoke_report.sh` with workload and drift checks.
- Update `docs/foundation/observability-slo-dashboards.md` with fixture mapping.

Out of scope:
- Host-specific calibration or auto-tuning.
- New benchmark executables.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | fixture matrix file | schema + runtime/signing/transport workloads present |
| C-02 | AC-2 | Functional | `--lane smoke --workload signing` | report emits deterministic signing-smoke metrics |
| C-03 | AC-2 | Regression | invalid/missing fixture schema marker | generator fails closed |
| C-04 | AC-3 | Integration | test harness invocation | matrix parsing + workload selection tests pass |
| C-05 | AC-4 | Docs | observability dashboards doc | fixture-to-SLO mapping table present |

## Test Mapping
- `bash scripts/ci/test_generate_performance_smoke_report.sh`
- `bash scripts/ci/generate_performance_smoke_report.sh --lane smoke --workload runtime --output-json <tmp>`

## Success Metrics
- Generator emits deterministic metrics for all three workloads in smoke/deep lanes.
- Drift in schema/workload entries fails tests deterministically.
