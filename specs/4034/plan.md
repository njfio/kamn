# Issue #4034 Plan - License Policy Parity Checker Across Root Policy and Crate Manifests

## Approach
1. Extend `check_workspace_license_policy.py` with root license policy file parity validation and deterministic reason-code taxonomy updates.
2. Extend `test_check_workspace_license_policy.sh` with root-policy mismatch coverage and deterministic output assertions.
3. Add Rust contract suite for unit/functional/integration/regression/performance coverage by executing checker commands against fixture mutations.
4. Wire checker shell lane into `scripts/ci/test_ci_tools.sh` (fast/full) and update command-surface contract expectations.
5. Update `docs/ci/strategy.md` markers/commands and `ci_strategy_docs.rs` assertions to keep docs parity fail-closed.

## Affected Modules
- `scripts/ci/check_workspace_license_policy.py`
- `scripts/ci/test_check_workspace_license_policy.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/workspace_license_policy_contract.rs` (new)
- `specs/4034/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: taxonomy drift breaking downstream contracts.
  - Mitigation: deterministic reason code ordering and docs-contract assertions.
- Risk: CI command-surface drift between fast/full lanes.
  - Mitigation: explicit wiring in both blocks plus command-surface contract test updates.
- Risk: checker runtime overhead.
  - Mitigation: bounded performance test in Rust contract suite.

## Interfaces / Contracts
- Checker schema: `kamn.ci.dependency-license-metadata-governance-report.v1`
- Reason taxonomy: `kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1`
- Root-policy reason codes (new):
  - `license_policy_file_not_found`
  - `license_policy_marker_mismatch`

## Validation Strategy
- RED: introduce Rust/docs contract tests expecting root-policy parity enforcement and CI command-surface inclusion before implementation.
- GREEN: implement checker/shell/docs/ci wiring and rerun targeted suites.
- VERIFY: run fmt, clippy, targeted Rust/shell tests, and command-surface contract checks.
