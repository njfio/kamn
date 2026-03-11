# 6866-align-supply-chain-advisory-rust-toolchain

## Objective
Restore mergeability of PRs blocked by the `supply-chain-advisory` workflow by aligning the advisory Docker build toolchain with the repository's supported Rust version and adding a fail-closed policy contract that prevents advisory toolchain drift from recurring silently.

## Inputs/Outputs
- Inputs:
  - Root [`Dockerfile`](../Dockerfile)
  - [`ci-supply-chain-advisory.yml`](../.github/workflows/ci-supply-chain-advisory.yml)
  - Existing workflow policy tests in [`scripts/ci/test_workflow_scope_policy.sh`](../scripts/ci/test_workflow_scope_policy.sh)
- Outputs:
  - Advisory Docker build succeeds under the repo's current dependency MSRV
  - A repository-level toolchain declaration for the advisory build surface
  - Policy coverage that fails if the advisory Docker builder drifts from that declared toolchain

## Boundaries/Non-goals
- Do not repair the governance/feature commit-ratio gate in this issue.
- Do not redesign the advisory lane or add new scanners.
- Do not change application runtime behavior.
- Do not migrate every CI lane away from `stable` unless required to make the advisory lane coherent.

## Failure modes
- Advisory Docker build still uses Rust 1.85 and fails on crates that now require 1.88.
- Dockerfile and repository toolchain declaration drift apart.
- Policy tests fail to detect advisory builder drift.
- Advisory reports are still missing because the image build fails before scans run.

## Acceptance criteria
- [ ] Root builder toolchain for the advisory Docker image aligns with the repository-declared Rust version.
- [ ] A fail-closed policy test asserts the advisory Docker builder toolchain contract.
- [ ] Existing workflow policy tests cover the advisory build marker and remain green.
- [ ] The advisory Docker image build succeeds locally on the issue branch.
- [ ] The issue spec records any deviation from broader CI toolchain pinning.

## Files to touch
- `Dockerfile`
- `rust-toolchain.toml`
- `scripts/ci/test_workflow_scope_policy.sh`
- `docs/ci/strategy.md`
- `specs/6866-align-supply-chain-advisory-rust-toolchain.md`

## Error semantics
- Policy failures must hard-fail with explicit mismatch messages naming the expected builder image / toolchain markers.
- Advisory build failures remain loud and must stop the workflow before artifact upload.
- No silent fallback to older Rust toolchains.

## Test plan
1. Add a red policy contract asserting the advisory Docker builder toolchain and repo toolchain declaration.
2. Confirm the new contract fails on current `main` because the Dockerfile still uses Rust 1.85 and no repo toolchain file exists.
3. Implement the minimal alignment change.
4. Run:
   - `bash scripts/ci/test_workflow_scope_policy.sh`
   - `docker build -t kamn-supply-chain-advisory:local .`
5. Record the outcome and any scope deviation in the spec.
