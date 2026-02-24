# Plan: Issue #5850

## Approach
1. Add a new Rust contract test file (`crates/kamn-core/tests/e2e_live_workflow_lane.rs`) that implements the same invariant checks and deterministic reason taxonomy currently enforced by shell/Python checker fixtures.
2. Remove now-redundant shell/Python checker artifacts:
   - `scripts/ci/check_e2e_live_workflow_contract.py`
   - `scripts/ci/test_check_e2e_live_workflow_contract.sh`
3. Update CI-tools fast-lane wiring to invoke the Rust contract test directly.
4. Update CI command-surface contract expectations and strategy docs markers to reflect the consolidated Rust lane.
5. Verify targeted Rust lane and full fast-mode regression suite.

## Affected Modules
- `crates/kamn-core/tests/e2e_live_workflow_lane.rs` (new)
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `docs/ci/strategy.md`
- delete: `scripts/ci/check_e2e_live_workflow_contract.py`
- delete: `scripts/ci/test_check_e2e_live_workflow_contract.sh`

## Risks & Mitigations
- Risk: test semantics drift during migration from shell/Python to Rust.
  - Mitigation: preserve identical reason codes and fixture mutations with explicit assertions.
- Risk: command-surface contract drift.
  - Mitigation: update required command list in same change and run fast-mode suite.

## Interfaces / Contracts
- Reason taxonomy remains: `kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1`
- Reason codes remain:
  - `workflow_file_missing`
  - `strategy_doc_missing`
  - `sdk_direct_job_missing`
  - `sdk_direct_live_toggle_missing`
  - `sdk_direct_external_execution_flag_missing`
  - `sdk_direct_scenarios_not_full_matrix`
  - `kolme_bootstrap_step_missing`
  - `kamn_runtime_bootstrap_missing`
  - `service_health_wait_marker_missing`
  - `ci_strategy_markers_missing`

## ADR
- Not required (no dependency/protocol addition).
