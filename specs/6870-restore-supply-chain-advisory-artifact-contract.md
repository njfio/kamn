# 6870-restore-supply-chain-advisory-artifact-contract

## Objective
Restore the `Supply-Chain Advisory` workflow so it always emits the expected advisory report artifacts on `main`, even when Trivy or SBOM generation does not write the requested output files.

## Inputs/Outputs
- Inputs:
  - `.github/workflows/ci-supply-chain-advisory.yml`
  - `scripts/ci/test_workflow_scope_policy.sh`
  - `scripts/ci/test_ci_tools.sh`
  - new workflow contract test script(s) under `scripts/ci/`
- Outputs:
  - advisory workflow always leaves behind:
    - `ci-supply-chain-advisory-trivy-fs.json`
    - `ci-supply-chain-advisory-trivy-image.json`
    - `ci-supply-chain-advisory-sbom.cdx.json`
    - `ci-supply-chain-advisory-license.json`
  - missing scanner output is represented by an explicit placeholder artifact with observable status metadata

## Boundaries/Non-goals
- Do not change governance/feature commit-ratio policy
- Do not convert the advisory lane into a required blocking lane
- Do not replace Trivy or redesign the advisory workflow beyond artifact-contract repair

## Failure modes
- Trivy action completes without creating the requested filesystem report
- Trivy action completes without creating the requested image report
- SBOM generation completes without creating the requested CycloneDX file
- workflow upload step fails because expected artifact paths are absent
- workflow silently hides scanner output loss instead of surfacing it in generated artifacts

## Acceptance criteria
- [ ] The advisory workflow guarantees all four advisory report paths exist before upload
- [ ] Missing Trivy/SBOM outputs are converted into explicit placeholder JSON artifacts with machine-readable status
- [ ] The placeholder path is observable in workflow summary or report payload, not silent
- [ ] CI contract tests fail if the workflow stops guaranteeing those four files
- [ ] Fast-mode CI tooling runs the new workflow artifact contract test

## Files to touch
- `.github/workflows/ci-supply-chain-advisory.yml`
- `scripts/ci/test_workflow_scope_policy.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_supply_chain_advisory_artifact_contract.sh`

## Error semantics
- The advisory lane remains advisory for scanner findings, but artifact-production failures are handled by emitting explicit placeholder JSON documents.
- Placeholder documents must include enough structured context to distinguish `generated_by_scan` from `placeholder_due_to_missing_output`.
- The workflow must fail only if it cannot guarantee the report contract after placeholder handling.

## Test plan
- Add a red shell contract test asserting placeholder-generation markers and artifact-guarantee markers in `ci-supply-chain-advisory.yml`
- Extend workflow policy coverage so fast-mode CI runs the new contract test
- Run the new contract test directly
- Run `scripts/ci/test_workflow_scope_policy.sh`
- Run `scripts/ci/test_ci_tools.sh` in fast mode only if needed for integration evidence

## Phase 6 Evidence
- `bash scripts/ci/test_supply_chain_advisory_artifact_contract.sh`
- `bash scripts/ci/test_workflow_scope_policy.sh`
- `python3 -m py_compile scripts/ci/ensure_advisory_report.py`
- `docker build -t kamn-supply-chain-advisory:local .`
- `docker image inspect kamn-supply-chain-advisory:local`

## Deviations
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` still exits on the unrelated governance/feature commit-ratio gate on `main`; this issue used the targeted workflow contract and policy entrypoints instead.

## Shell-Surface Closure Metrics
- `shell_loc_delta_actual: 126`
- `rust_loc_delta_actual: 0`
- `shell_to_rust_ratio_delta_actual: 0.0`
- `shell_surface_ratio_target_status: regressed_with_waiver`
