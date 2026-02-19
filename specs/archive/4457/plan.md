# Plan: Issue #4457

Status: Completed
Issue: #4457

## Approach

1. Add RED assertions to workspace license policy tests for deterministic taxonomy/class outputs and
   CI/local boundary markers, including local-heavy opt-in enforcement.
2. Implement taxonomy and boundary reporting in `scripts/ci/check_workspace_license_policy.py`
   while preserving existing stderr failure details.
3. Update `docs/ci/strategy.md` with metadata-governance CI smoke/local-heavy matrix and bind it
   with docs-contract assertions in `crates/kamn-core/tests/ci_strategy_docs.rs`.
4. Run scoped red/green verification plus fmt/clippy hygiene.

## Affected Modules

- `scripts/ci/check_workspace_license_policy.py`
- `scripts/ci/test_check_workspace_license_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4457/*`

## Risks and Mitigations

- Risk: checker output changes could break existing script consumers.
  - Mitigation: retain legacy stderr summary lines and add deterministic markers in stdout/json.
- Risk: boundary markers drift from docs contract.
  - Mitigation: add focused docs-contract test assertions tied to exact marker strings.

## Interfaces / Contracts

- Workspace license checker outputs deterministic markers:
  - `reason_taxonomy_version`
  - `reason_codes_csv`
  - `reason_codes_value`
  - `reason_class`
  - `ci_smoke_local_heavy_boundary_status`
  - `ci_smoke_lane_cost_profile`
  - `local_heavy_lane_execution_mode`
- Local-heavy boundary fail-closed reason:
  - `metadata_governance_local_heavy_opt_in_required`

## ADR

Not required: no new dependencies or architecture changes.
