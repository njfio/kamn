# Objective
Enforce a fail-closed GitHub Actions runtime policy so repository workflows cannot run for hours unnoticed. Every workflow job must declare an explicit runtime budget, no job may exceed a repository-wide ceiling of 60 minutes, and pull-request workflows that supersede earlier runs must cancel in-progress predecessors automatically.

## Inputs/Outputs
- Inputs:
  - `.github/workflows/*.yml`
  - `docs/ci/strategy.md`
  - existing CI policy harnesses under `scripts/ci/` and `crates/kamn-core/tests/`
- Outputs:
  - a policy checker or contract lane that fails when a workflow omits `timeout-minutes`
  - enforcement that no workflow job exceeds 60 minutes
  - enforcement that PR-triggered workflows define top-level `concurrency` with `cancel-in-progress: true`
  - workflow updates for existing violating files
  - docs describing the runtime ceiling and cancellation policy

## Boundaries/Non-goals
- Do not redesign the logical contents of CI workflows.
- Do not remove validation coverage solely to reduce runtime.
- Do not change product runtime behavior outside GitHub Actions policy enforcement.
- Do not introduce new third-party dependencies.

## Failure modes
- A workflow job omits `timeout-minutes`.
- A workflow job sets `timeout-minutes` greater than 60.
- A workflow with a `pull_request` trigger omits top-level `concurrency`.
- A workflow with a `pull_request` trigger omits `cancel-in-progress: true` in the top-level concurrency block.
- Policy documentation drifts from enforced workflow behavior.

## Acceptance criteria
- [ ] Every job in `.github/workflows/*.yml` declares `timeout-minutes`.
- [ ] No workflow job declares `timeout-minutes` greater than 60.
- [ ] Every workflow with a `pull_request` trigger defines top-level `concurrency` with `cancel-in-progress: true`.
- [ ] Existing violating workflows are updated to comply, including `ci-fast-gate.yml`, `ci-deep-validate.yml`, `ci-supply-chain-advisory.yml`, `e2e-live.yml`, and `branch-cleanup.yml`.
- [ ] A fail-closed CI policy test/checker exercises the workflow set and fails on missing timeouts, excessive ceilings, or missing PR concurrency cancellation.
- [ ] `docs/ci/strategy.md` documents the workflow runtime ceiling and superseded-run cancellation rule.

## Files to touch
- `specs/6845-enforce-actions-runtime-ceilings-and-cancel-in-progress-policy.md`
- `.github/workflows/ci-fast-gate.yml`
- `.github/workflows/ci-deep-validate.yml`
- `.github/workflows/ci-supply-chain-advisory.yml`
- `.github/workflows/e2e-live.yml`
- `.github/workflows/branch-cleanup.yml`
- `scripts/ci/test_workflow_runtime_ceiling_policy.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- optionally one Rust docs/contract test if required to keep policy discoverable

## Error semantics
- Policy violations must hard-fail with file path and missing/invalid marker context.
- The checker must not silently skip workflows or jobs.
- Existing workflows that remain over budget after the change are not allowed; the policy is fail-closed.

## Test plan
1. Add a failing policy test that scans `.github/workflows/*.yml` and reports:
   - missing `timeout-minutes`
   - any timeout greater than 60
   - missing top-level PR concurrency cancellation
2. Run the new policy test and confirm it fails on the current workflow set.
3. Update the violating workflows with explicit compliant budgets and concurrency markers.
4. Re-run the policy test until green.
5. Run existing workflow-policy regression tests impacted by the edits.
6. Wire the new policy test into the real CI tools entrypoint so future workflow additions are checked automatically.
