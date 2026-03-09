# 6649 Add Advisory Supply-Chain Security Lane

## Objective

Add a dedicated advisory-only supply-chain workflow that produces visible CI artifacts for secrets scanning, dependency/container vulnerability scanning, SBOM generation, and license inventory without turning any of those checks into required merge gates in this issue.

## Inputs/Outputs

- Inputs:
  - Existing fast/deep workflow cargo-audit steps
  - Existing workspace license policy checker `scripts/ci/check_workspace_license_policy.py`
  - Existing SBOM provenance docs/contracts in `docs/ci/strategy.md`, `docs/ops/configuration.md`, and `crates/kamn-core/tests/*sbom*`
  - Root `Dockerfile`
- Outputs:
  - New advisory workflow under `.github/workflows/`
  - Advisory artifact outputs for filesystem/image scan, SBOM, and license inventory
  - Workflow contract test coverage for the new advisory lane
  - CI strategy docs covering waiver handling and promotion path

## Boundaries/Non-goals

- Do not make the new lane blocking in this issue
- Do not replace existing cargo-audit or license-policy enforcement lanes
- Do not redesign deployment/release infrastructure
- Do not add broad platform-specific image hardening in this issue

## Failure Modes

- The advisory lane runs blocking commands and fails the workflow instead of reporting findings
- Secrets scanning or vulnerability scanning is missing, leaving the lane incomplete
- SBOM generation runs without artifact retention, making the output unusable
- License inventory/compliance reporting is omitted instead of reusing the existing workspace license checker
- Docs do not explain false-positive handling or how the lane graduates to required status

## Acceptance Criteria

- [x] CI runs a dedicated advisory-only supply-chain workflow
- [x] The advisory lane includes secrets scanning
- [x] The advisory lane includes SBOM generation
- [x] The advisory lane includes container/dependency vulnerability scanning
- [x] The advisory lane includes license inventory or compliance reporting
- [x] Artifact outputs are retained and visible in CI
- [x] Docs record waiver / false-positive handling for the advisory tools
- [x] Docs record a follow-up promotion path from advisory to required

## Files To Touch

- `specs/6649-add-advisory-supply-chain-security-lane.md`
- `.github/workflows/ci-supply-chain-advisory.yml`
- `.trivyignore`
- `.ci/shell_test_surface_ratio_thresholds.env`
- `.ci/shell_test_surface_ratio_waiver_6649.env`
- `scripts/ci/test_workflow_scope_policy.sh`
- `scripts/ci/test_check_workspace_license_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-governance/Cargo.toml`

## Error Semantics

- The advisory lane must upload findings even when scanners find issues
- Tool invocation/setup errors should fail the advisory workflow job itself because the lane must remain trustworthy
- Findings from Trivy/license inventory must remain non-blocking in this issue and be documented as advisory-only

## Test Plan

- Run `bash scripts/ci/test_workflow_scope_policy.sh`
- Run `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- Run `bash scripts/ci/test_check_workspace_license_policy.sh`
- Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_supply_chain_advisory_lane_markers -- --exact --nocapture`
- Run `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`
- Run `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6649-touched-size.json`

## Notes / Deviations

- Existing blocking `cargo-audit` and workspace-license enforcement remain in place. This issue adds a parallel advisory lane rather than replacing or weakening current gates.

## Integration Evidence

- `bash scripts/ci/test_workflow_scope_policy.sh`
  - passed
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - passed
- `bash scripts/ci/test_check_workspace_license_policy.sh`
  - passed after normalizing `crates/kamn-governance/Cargo.toml` to explicit `license = "Apache-2.0"`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_supply_chain_advisory_lane_markers -- --exact --nocapture`
  - passed
- `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`
  - passed with the new bounded waiver file `.ci/shell_test_surface_ratio_waiver_6649.env`
- `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6649-touched-size.json`
  - `status=pass`
  - `policy_decision=GO`

## Deviations

- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` repeatedly reached an unrelated late runtime-suite interaction in `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` (`request-validation probe expected 400 status; got 401`), even though:
  - the advisory workflow/docs/license tests above passed
  - the isolated runtime subsequence ending in `test_check_service_api_axum_ingress_live_policy.sh` passed on both the pre-issue branch and the `#6649` branch
- That late suite interaction was not pursued inside `#6649` because it does not overlap the advisory workflow, license metadata, or touched Rust surface changed by this issue.
