# Plan — Issue #4863

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `scripts/ci/fast_gate_budget_delta.py`
- `scripts/ci/fast_gate_budget_delta_contract_lane_impl.sh`
- `scripts/ci/test_check_fast_gate_budget_delta_threshold.sh`
- `scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh`
- `scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
- `scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
- `scripts/ci/check_pr_ci_declaration.sh`
- `scripts/ci/test_check_pr_ci_declaration.sh`
- `scripts/ci/test_pr_template_shell_surface_markers_contract.sh`
- `scripts/ci/test_shell_surface_issue_intake_contract.sh`
- `AGENTS.md`
- `.github/ISSUE_TEMPLATE/epic.md`
- `.github/ISSUE_TEMPLATE/story.md`
- `.github/ISSUE_TEMPLATE/task.md`
- `.github/ISSUE_TEMPLATE/subtask.md`
- `.github/pull_request_template.md`
- `.github/CONTRIBUTING.md`
- `docs/ci/strategy.md`
- `docs/ci/ci-cost-and-lane-framework.md`
- `.ci/fast-gate-budget-delta-ratchet.env`

## Risks / Mitigations

- Risk: migration drift or hidden coupling across scripts/wrappers/checkers.
  Mitigation: phased rollout with compatibility checks and deterministic regression lanes.
- Risk: CI runtime growth.
  Mitigation: retain bounded fast-gate budgets and enforce explicit threshold checks.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Keep reason taxonomy/version markers deterministic and fail closed on drift.

## ADR

- Required if implementation introduces architecture/dependency/protocol strategy changes.
