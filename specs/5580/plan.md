# Issue #5580 Plan - PRD Phase-4i CI Live-Lane Integration and Hardening Contracts

## Approach
1. Add RED tests for `e2e-live` workflow markers and phase-4i docs/milestone markers.
2. Add `.github/workflows/e2e-live.yml` with PRD lane structure and deterministic markers.
3. Add phase-4i docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `.github/workflows/e2e-live.yml` (new)
- `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase4i-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: workflow syntax/policy failures in CI checks.
  - Mitigation: keep YAML minimal and deterministic; verify marker tests locally.
- Risk: lane drift from PRD topology.
  - Mitigation: lock required markers with contract tests.

## Interfaces / Contracts
- Workflow file contract:
  - `.github/workflows/e2e-live.yml`
  - Trigger markers: `schedule`, `workflow_dispatch`
  - Lane markers: `e2e-sdk-direct`, `e2e-mcp-agent`, `e2e-cli-smoke`
  - Mode markers: `--mode sdk-direct`, `--mode mcp-tau`, `--mode cli-scripted`

## ADR
- Not required for CI contract scaffold.
