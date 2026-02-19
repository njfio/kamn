# Issue #4094 Plan

- Issue: #4094
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Approach
1. Reuse existing daemon OS-signal stress matrix runner and contract test harness as the implementation baseline.
2. Add explicit overload profile marker documentation in `docs/ops/configuration.md` linking:
   - baseline profile (`stable_success`) and
   - injected-overload profile (`matrix_failure_threshold_exceeded`).
3. Use deterministic docs marker lookups (`rg`) and existing shell contract tests as verification evidence.
4. Execute targeted validation commands and mark spec status `Implemented`.

## Affected Files
- `docs/ops/configuration.md`
- `specs/4094/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: docs drift from stress runner reason taxonomy.
  - Mitigation: deterministic marker lookups are part of conformance mapping.
- Risk: local-heavy semantics accidentally pulled into CI smoke path.
  - Mitigation: retain existing dry-run/opt-in wording and bounded runtime markers from current strategy contracts.
- Risk: shell-surface expansion.
  - Mitigation: no runner/script/workflow edits for this issue closure.

## Interface Contract
- Required ops marker keys:
  - `daemon_os_signal_stress_matrix_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1`
  - `daemon_os_signal_stress_profile_baseline_reason_code=stable_success`
  - `daemon_os_signal_stress_profile_injected_overload_reason_code=matrix_failure_threshold_exceeded`
  - `daemon_os_signal_stress_profile_recovery_reason_code=stable_success_with_quarantine_followup`
  - `daemon_os_signal_stress_profile_runtime_budget_reason_code=runtime_budget_exceeded`
- Required validation commands:
  - `bash scripts/ci/run_daemon_os_signal_stress_matrix.sh`
  - `bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh`
