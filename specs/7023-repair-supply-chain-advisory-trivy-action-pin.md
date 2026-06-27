# 7023-repair-supply-chain-advisory-trivy-action-pin

## Objective
Repair the `Supply-Chain Advisory` workflow action reference so GitHub can resolve the Trivy action and its nested setup action before checkout, then run the existing advisory scans.

## Inputs/Outputs
- Inputs:
  - `.github/workflows/ci-supply-chain-advisory.yml`
  - `scripts/ci/test_workflow_scope_policy.sh`
  - `scripts/ci/test_supply_chain_advisory_artifact_contract.sh`
  - `crates/kamn-core/tests/supply_chain_advisory_workflow_contract.rs`
- Outputs:
  - Every Trivy step uses `aquasecurity/trivy-action@v0.31.0`.
  - Local workflow contracts fail if the action pin regresses to the unresolvable `0.28.0` form or the intermediate `v0.28.0` form that still references missing `setup-trivy`.
  - Existing advisory report artifact markers remain unchanged.

## Boundaries/Non-goals
- Do not change advisory scan policy, scanner lists, report names, upload paths, or `continue-on-error` behavior.
- Do not upgrade Trivy beyond the minimal checked tag that resolves both the top-level action and nested setup action.
- Do not modify required-check policy, workflow triggers, or CI bypass behavior.
- Do not add dependencies.

## Failure Modes
- GitHub cannot resolve the top-level Trivy action ref and the workflow fails before checkout.
- GitHub resolves Trivy but fails during setup because the Trivy action references a missing nested setup action tag.
- A future edit reintroduces an unresolvable Trivy action pin.
- The fix accidentally removes advisory scan, SBOM, license, or artifact contract markers.

## Acceptance Criteria
- [x] The supply-chain advisory workflow references `aquasecurity/trivy-action@v0.31.0` for every Trivy step.
- [x] Local contracts fail against the current `aquasecurity/trivy-action@0.28.0` workflow.
- [x] Local contracts pass after the workflow pin repair.
- [x] `v0.31.0` tag and action metadata prove the nested setup action is pinned by SHA.
- [x] Existing supply-chain artifact contract coverage still passes.
- [x] No advisory scan, SBOM, secret, vulnerability, or license evidence is disabled or weakened.

## Files To Touch
- `.github/workflows/ci-supply-chain-advisory.yml`
- `scripts/ci/test_workflow_scope_policy.sh`
- `scripts/ci/test_supply_chain_advisory_artifact_contract.sh`
- `crates/kamn-core/tests/supply_chain_advisory_workflow_contract.rs`

## Error Semantics
- Workflow action-resolution failures must remain hard failures.
- Advisory findings remain advisory per the existing workflow behavior; this issue only repairs the action reference needed to run the advisory lane.
- Contract test failures must print the missing expected action pin marker.

## Test Plan
- Red: update workflow contract tests to require `aquasecurity/trivy-action@v0.31.0`, then run them and confirm they fail on the current workflow.
- Green: repair every Trivy action reference in `.github/workflows/ci-supply-chain-advisory.yml`, then rerun the targeted contracts.
- Integration: run `bash scripts/ci/test_supply_chain_advisory_artifact_contract.sh`, `bash scripts/ci/test_workflow_scope_policy.sh`, and `cargo test -p kamn-core --test supply_chain_advisory_workflow_contract`.
- Final quality: rerun PR declaration validation locally after updating the PR body if needed.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +20`
- `rust_loc_delta_estimate: +60`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7023`

## Phase Evidence
- Red:
  - `bash scripts/ci/test_supply_chain_advisory_artifact_contract.sh` failed on missing `aquasecurity/trivy-action@v0.31.0`.
  - `bash scripts/ci/test_workflow_scope_policy.sh` failed on missing `aquasecurity/trivy-action@v0.31.0`.
  - `target/debug/deps/supply_chain_advisory_workflow_contract-69fdb5be3435aa11 --nocapture` failed on missing `aquasecurity/trivy-action@v0.31.0`.
- Green:
  - `bash scripts/ci/test_supply_chain_advisory_artifact_contract.sh && bash scripts/ci/test_workflow_scope_policy.sh` passed.
  - `cargo test -p kamn-core --test supply_chain_advisory_workflow_contract` passed.
  - `gh api repos/aquasecurity/trivy-action/git/ref/tags/v0.31.0 --jq .ref` returned `refs/tags/v0.31.0`.
  - `v0.31.0` action metadata references `aquasecurity/setup-trivy@ff1b8b060f23b650436d419b5e13f67f5d4c3087`.
- Refactor:
  - No separate code reshape was warranted for this pin-only workflow repair; adding indirection would make the workflow less explicit.
- Integration:
  - The workflow contract scripts exercise the real `.github/workflows/ci-supply-chain-advisory.yml` file.
  - `scripts/ci/collect_shell_rust_loc_telemetry.sh --output-json <tmp>` reported `status=ok`, `final_decision=GO`.

## Shell-Surface Closure Metrics
- `shell_loc_delta_actual: +219`
- `rust_loc_delta_actual: +166008`
- `shell_to_rust_ratio_delta_actual: -0.181472`
- `shell_surface_ratio_target_status: improved`
- `shell_surface_mitigation_issue: None`
