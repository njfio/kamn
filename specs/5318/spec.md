# Issue #5318 Spec

- Title: Mitigation: reduce shell LOC introduced by performance baseline provenance contracts
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement
Issue #4001 added provenance + drift-seed enforcement in CI scripts, but that enforcement currently lives in large shell scripts. This increases shell surface area and maintenance burden. We need a thin-shell pattern while preserving deterministic fail-closed behavior and existing contract tests.

## Acceptance Criteria
- AC-1: Move performance report generation/check enforcement logic out of shell into a shared Python implementation while keeping shell entrypoints stable.
- AC-2: Reduce shell LOC across `scripts/ci/generate_performance_smoke_report.sh` and `scripts/ci/check_performance_thresholds.sh` relative to current baseline.
- AC-3: Preserve deterministic pass/fail markers and error messages required by existing tests.
- AC-4: Targeted CI contract tests for performance report generation/checking remain green.

## Scope
In scope:
- Add shared Python implementation for generation + threshold checking.
- Convert both shell scripts to thin wrappers.
- Keep command surface unchanged for workflows/docs/tests.

Out of scope:
- Workflow command changes.
- New policy thresholds or fixture schema revisions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | call existing shell entrypoints | wrappers delegate to Python and preserve behavior |
| C-02 | AC-2 | Functional | `wc -l` on target shell scripts | combined shell LOC decreases |
| C-03 | AC-3 | Regression | invalid workload/schema/missing marker inputs | same deterministic failure markers/messages |
| C-04 | AC-4 | Conformance | existing script tests | all targeted tests pass |

## Test Mapping
- `bash scripts/ci/test_generate_performance_smoke_report.sh`
- `bash scripts/ci/test_check_performance_thresholds.sh`
- `cargo test -p kamn-core --test shell_test_surface_migration_wave1 spec_c03_performance_threshold_checker_contract`

## Success Metrics
- Shell LOC for the two target scripts decreases by at least 120 lines.
- No regressions in generation/checker contract tests.
