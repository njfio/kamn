# 6650 Enforce Shell Script Strict Mode On Touched Scripts

## Objective

Add a ratcheted CI policy that fails closed when a touched shell script under `scripts/` is executable shell code without `set -euo pipefail`, while preserving an explicit exception path for sourced helper libraries where strict mode is not appropriate.

## Inputs/Outputs

- Inputs:
  - Existing fast-gate shell policy surface in `.github/workflows/ci-fast-gate.yml`
  - Existing CI tool regression lane in `scripts/ci/test_ci_tools.sh`
  - Existing workflow/docs contracts in `scripts/ci/test_workflow_scope_policy.sh`, `scripts/ci/test_ci_tools_command_surface_contract.sh`, `scripts/ci/test_ci_strategy_contract.sh`, and `crates/kamn-core/tests/ci_strategy_docs.rs`
  - Current shell helper libraries in `scripts/lib/`
- Outputs:
  - Touched-shell strict-mode checker with a stable wrapper entrypoint
  - Explicit strict-mode exception inventory for non-executable sourced libraries
  - Fast-gate enforcement for touched shell scripts
  - Contract coverage and docs for the new ratchet

## Boundaries/Non-goals

- Do not bulk-edit untouched legacy shell scripts in this issue
- Do not enforce strict mode on sourced helper libraries without an explicit review of side effects
- Do not redesign workflow selection or shell-surface ratio policy in this issue
- Do not broaden the checker beyond shell scripts under `scripts/`

## Failure Modes

- A touched executable shell script omits `set -euo pipefail` and still passes CI
- Sourced helper libraries without strict mode fail the checker without an explicit exception path
- The checker reports non-specific failures instead of exact offending paths
- Fast gate documentation drifts from the actual command surface
- Invalid exception inventory metadata silently disables enforcement

## Acceptance Criteria

- [x] CI rejects touched shell scripts missing strict mode where strict mode is valid
- [x] The policy supports a documented exception path for scripts where strict mode is not appropriate
- [x] Policy output identifies exact offending scripts
- [x] The shell-script policy is documented in repo docs
- [x] At least one test/contract covers the strict-mode ratchet

## Files To Touch

- `specs/6650-enforce-shell-script-strict-mode-on-touched-scripts.md`
- `scripts/ci/check_touched_shell_strict_mode.py`
- `scripts/ci/check_touched_shell_strict_mode.sh`
- `scripts/ci/test_check_touched_shell_strict_mode.sh`
- `fixtures/ci/touched_shell_strict_mode_exceptions.txt`
- `scripts/lib/exec_registry.json`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `scripts/ci/test_ci_strategy_contract.sh`
- `scripts/ci/test_workflow_scope_policy.sh`
- `.github/workflows/ci-fast-gate.yml`
- `.ci/shell_test_surface_ratio_thresholds.env`
- `.ci/shell_test_surface_ratio_waiver_6650.env`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Error Semantics

- The checker must fail with deterministic reason codes and exact offending script paths
- Missing or malformed exception inventory must fail closed rather than silently skipping enforcement
- Scripts that are explicitly exempted must be surfaced as exemptions, not treated as compliant scripts
- Entrypoints may emit summary markers once; interior helpers should return structured failure data without logging side effects

## Test Plan

- Run `bash scripts/ci/test_check_touched_shell_strict_mode.sh`
- Run `bash scripts/ci/test_workflow_scope_policy.sh`
- Run `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- Run `bash scripts/ci/test_ci_strategy_contract.sh`
- Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_touched_shell_strict_mode_markers -- --exact --nocapture`
- Run `bash scripts/ci/check_touched_shell_strict_mode.sh --output-json /tmp/touched-shell-strict-mode-report.json`

## Notes / Deviations

- Current repo sampling shows only `scripts/lib/common.sh` and `scripts/lib/test_harness.sh` lack `set -euo pipefail`; both are sourced helper libraries and will be represented as explicit exceptions rather than retrofitted in this issue.

## Integration Evidence

- `bash scripts/ci/test_check_touched_shell_strict_mode.sh`
  - passed
- `bash scripts/ci/test_workflow_scope_policy.sh`
  - passed
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - passed
- `bash scripts/ci/test_ci_strategy_contract.sh`
  - passed
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_touched_shell_strict_mode_markers -- --exact --nocapture`
  - passed
- `bash scripts/ci/check_touched_shell_strict_mode.sh --output-json /tmp/touched-shell-strict-mode-report.json`
  - passed with `status=pass` and `reason_codes=none`
- `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
  - passed after switching the shell-test ratio waiver pointer to `.ci/shell_test_surface_ratio_waiver_6650.env`
- `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`
  - passed with the new bounded `#6650` shell-test ratio waiver
- `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
  - passed in isolation

## Deviations

- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` now passes through the new touched-shell strict-mode lane, the shell-ratio waiver path, and the surrounding shell-surface contracts, but it still reaches the pre-existing late runtime-suite interaction already recorded during `#6649` work:
  - `request-validation probe expected 400 status; got 401`
  - `expected service api axum ingress validation marker status=pass`
- The isolated `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` contract still passes on this branch, so that late-suite interaction was not pursued inside `#6650`.
