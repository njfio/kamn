# Issue #4094 Spec

- Title: Subtask: implement local-heavy overload stress runner and deterministic degradation-recovery artifact schema
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Overload resilience claims require deterministic local-heavy stress artifacts with explicit degradation/recovery markers. Existing runner coverage must be bound to issue-scoped, fail-closed docs and test contracts.

## Acceptance Criteria
- AC-1: Local-heavy daemon OS-signal stress matrix artifacts remain deterministic and schema-valid.
- AC-2: Baseline and injected-overload profiles map to stable degradation/recovery markers.
- AC-3: Unit, Functional, Integration, and Regression coverage exists and passes (or has explicit N/A justification).

## Scope
In scope:
- Add `#4094` docs markers in `docs/ops/configuration.md` for stress matrix profile mapping.
- Verify existing shell runner/test artifacts for deterministic schema and fail-closed behavior.
- Add issue lifecycle artifacts `specs/4094/{spec,plan,tasks}.md`.

Out of scope:
- New shell runner implementation.
- New workflow/CI topology changes.
- Internet-scale stress/load generation.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional | `bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh` | stress matrix schema + deterministic stable/fail markers validated fail-closed |
| C-02 | AC-2 | Functional | `rg -n \"daemon_os_signal_stress_profile_baseline_reason_code=stable_success\" docs/ops/configuration.md` | baseline profile marker exists and is deterministic |
| C-03 | AC-2 | Integration | `rg -n \"daemon_os_signal_stress_profile_injected_overload_reason_code=matrix_failure_threshold_exceeded\" docs/ops/configuration.md` | injected-overload profile marker exists and is deterministic |
| C-04 | AC-3 | Regression | `rg -n \"daemon_os_signal_stress_matrix_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1\" docs/ops/configuration.md` | schema marker drift fails deterministically |

## Test Mapping
- `bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh`
- `rg -n \"daemon_os_signal_stress_profile_baseline_reason_code=stable_success\" docs/ops/configuration.md`
- `rg -n \"daemon_os_signal_stress_profile_injected_overload_reason_code=matrix_failure_threshold_exceeded\" docs/ops/configuration.md`
- `rg -n \"daemon_os_signal_stress_matrix_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1\" docs/ops/configuration.md`

## Success Metrics
- No shell LOC increase required for this closure path (`shell_loc_delta_actual=0` target).
- AC-to-test mapping is explicit and reproducible via deterministic command outputs.
