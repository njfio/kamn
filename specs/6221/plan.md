# Issue 6221 Plan

## Approach
1. Add a failing assertion in `phase4i_ci_workflow_contract.rs` that enforces explicit TLS mode markers for all three live lanes.
2. Update `.github/workflows/e2e-live.yml` run scripts to export `KAMN_SERVICE_API_TLS_MODE=disable` before launching `kamn-node`.
3. Run targeted workflow contract tests for verification.

## Affected Modules
- `.github/workflows/e2e-live.yml`
- `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs`

## Risks and Mitigations
- Risk: future workflow refactors remove env marker and reintroduce startup failure.
  - Mitigation: contract test checks explicit marker count across all live lanes.
- Risk: disable marker only in one lane.
  - Mitigation: deterministic count assertion `== 3`.

## Interfaces
- Workflow runtime environment contract for `kamn-node` live lane startup.
