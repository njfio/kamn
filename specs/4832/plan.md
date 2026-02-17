# Plan — Issue #4832

## Approach

1. Add a workflow contract test that fails when ratio/policy/telemetry shell-surface checks are missing from `ci-fast-gate`.
2. Wire three deterministic fast-gate steps (scoped to `run_script_surface_budget_checks`):
   - generate combined shell-surface trend report
   - check combined shell-surface policy thresholds
   - collect shell-vs-Rust telemetry from generated report
3. Upload deterministic workflow artifacts for:
   - script-surface budget
   - combined shell-surface trend report + policy
   - shell-vs-Rust telemetry
4. Keep runtime bounded by preserving `timeout-minutes: 20` and reusing generated report via collector `--report-file`.
5. Add workflow-wiring test to CI tools regression and validate via fast-mode suite.

## Affected Modules

- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/collect_shell_rust_loc_telemetry.sh`
- `docs/ci/strategy.md`

## Risks / Mitigations

- Risk: fast-gate runtime increase from additional governance checks.
  Mitigation: retain `timeout-minutes: 20`, scope checks behind selector, and reuse generated trend report.
- Risk: workflow drift between docs and actual commands.
  Mitigation: add deterministic workflow wiring contract test and run through CI tools regression lane.
- Risk: telemetry/report contract mismatch.
  Mitigation: enforce report schema and fail-closed taxonomy mapping in collector and existing policy tests.

## Interfaces / Contracts

- Fast-gate selector gate:
  - `if: steps.scope.outputs.run_script_surface_budget_checks == 'true'`
- Workflow artifact contracts:
  - `ci-script-surface-budget-<run_id>-<run_attempt>`
  - `ci-combined-shell-surface-trend-<run_id>-<run_attempt>`
  - `ci-shell-rust-loc-telemetry-<run_id>-<run_attempt>`
- Workflow command contract:
  - `check_script_duplication_budget.sh`
  - `generate_combined_shell_surface_trend_report.sh`
  - `check_combined_shell_surface_trend_policy.sh`
  - `collect_shell_rust_loc_telemetry.sh --report-file <generated-report>`

## ADR

No ADR required. No dependency/protocol boundary changes were introduced.
