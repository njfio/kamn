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

- [ ] CI runs a dedicated advisory-only supply-chain workflow
- [ ] The advisory lane includes secrets scanning
- [ ] The advisory lane includes SBOM generation
- [ ] The advisory lane includes container/dependency vulnerability scanning
- [ ] The advisory lane includes license inventory or compliance reporting
- [ ] Artifact outputs are retained and visible in CI
- [ ] Docs record waiver / false-positive handling for the advisory tools
- [ ] Docs record a follow-up promotion path from advisory to required

## Files To Touch

- `specs/6649-add-advisory-supply-chain-security-lane.md`
- `.github/workflows/ci-supply-chain-advisory.yml`
- `scripts/ci/test_supply_chain_advisory_workflow.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Error Semantics

- The advisory lane must upload findings even when scanners find issues
- Tool invocation/setup errors should fail the advisory workflow job itself because the lane must remain trustworthy
- Findings from Trivy/license inventory must remain non-blocking in this issue and be documented as advisory-only

## Test Plan

- Run `bash scripts/ci/test_supply_chain_advisory_workflow.sh`
- Run `bash scripts/ci/test_ci_tools.sh`
- Run `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_supply_chain_advisory_lane_markers -- --exact --nocapture`

## Notes / Deviations

- Existing blocking `cargo-audit` and workspace-license enforcement remain in place. This issue adds a parallel advisory lane rather than replacing or weakening current gates.
