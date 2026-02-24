# Spec: Issue #5850 - Mitigate Shell-Surface Regression from #5849 via Rust Contract Lane Consolidation

- Issue: #5850
- Status: Reviewed
- Type: task
- Priority: P2
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Issue #5849 improved live E2E workflow contract coverage, but it introduced net shell/python/workflow LOC growth via a dedicated Python checker and shell harness. We need to preserve fail-closed, deterministic contract enforcement while reducing shell-surface pressure.

## Scope
In scope:
- Replace the shell/Python e2e-live workflow contract validation lane with an equivalent Rust contract test lane in `kamn-core`.
- Preserve deterministic reason taxonomy and fail-closed negative fixtures (missing live toggle, truncated scenarios, missing external execution, missing strategy markers).
- Keep CI tool fast-mode and command-surface contract lanes green with updated invocation wiring.
- Update strategy docs markers for the consolidated Rust lane.

Out of scope:
- Reducing scenario breadth below `S-01..S-15`.
- Modifying upstream `fpco/kolme` repository.
- Changing core workflow runtime orchestration semantics outside existing invariants.

## Acceptance Criteria
- AC-1: e2e-live workflow contract checks run from Rust (`kamn-core` test lane), not the removed shell/Python checker pair.
- AC-2: Deterministic reason taxonomy remains enforced and validated through pass/fail fixtures.
- AC-3: Fast CI-tools lane and command-surface contracts remain green after lane consolidation.
- AC-4: Shell-surface impact for this issue is neutral or improved (no net shell/python/workflow growth).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | tracked `.github/workflows/e2e-live.yml` + `docs/ci/strategy.md` | Rust lane validates live markers and passes |
| C-02 | AC-2 | Regression | workflow fixture missing `KAMN_E2E_SDK_DIRECT_LIVE` | lane fails with `sdk_direct_live_toggle_missing` |
| C-03 | AC-2 | Regression | workflow fixture truncated to `S-01..S-06` | lane fails with `sdk_direct_scenarios_not_full_matrix` |
| C-04 | AC-2 | Regression | workflow fixture missing `--enable-external-execution` | lane fails with `sdk_direct_external_execution_flag_missing` |
| C-05 | AC-2 | Regression | strategy fixture missing required marker section | lane fails with `ci_strategy_markers_missing` |
| C-06 | AC-3 | Integration | `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` | includes Rust lane and passes |
| C-07 | AC-4 | Regression | shell/python script inventory before/after #5850 diff | no net shell/python/workflow increase from #5850 |

## Test Mapping
- `cargo test -p kamn-core --test e2e_live_workflow_lane`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`

## Success Metrics / Observable Signals
- Removed shell/Python checker pair with equivalent or stricter Rust fail-closed assertions.
- CI-tools fast mode and workspace contract gates remain green.
- Shell-surface mitigation target for #5850 is non-regressive.
