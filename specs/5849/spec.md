# Spec: Issue #5849 - Enforce Live E2E CI Lane With Local Kolme + KAMN Runtime

- Issue: #5849
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The repository contains a live E2E workflow (`.github/workflows/e2e-live.yml`), but there is no fail-closed contract test ensuring that live execution markers remain intact as the workflow evolves. This leaves risk of silent regression back to partial/stubbed coverage. SDK-direct live execution is currently scoped to a subset of scenarios; this issue hardens coverage expectations and enforces them through executable CI contracts.

## Scope
In scope:
- Add a deterministic workflow checker for `e2e-live.yml` that validates live-stack orchestration markers (local Kolme boot, KAMN service health waits, live env toggles, external execution mode).
- Enforce SDK-direct live scenario matrix breadth to include S-01..S-15.
- Add regression tests for the checker (pass + fail fixtures).
- Wire checker tests into CI tool regression lane.
- Document contract markers in CI strategy docs.

Out of scope:
- Rewriting e2e-harness scenario logic.
- Altering upstream Kolme repo source code.
- Making `e2e-live.yml` a required PR gate.

## Acceptance Criteria
- AC-1: `e2e-live.yml` contains explicit SDK-direct live enablement and local runtime orchestration markers required for non-stubbed execution.
- AC-2: SDK-direct job in `e2e-live.yml` executes full scenario set S-01..S-15.
- AC-3: A checker script fails closed when required live markers or full scenario set drift.
- AC-4: Checker regression tests cover valid workflow and deterministic failure reason taxonomy.
- AC-5: CI tool regression lane executes the checker tests, and strategy docs contain marker references.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | current `e2e-live.yml` | required live orchestration/env markers present |
| C-02 | AC-2 | Functional | SDK-direct scenario CLI fragment | exactly S-01..S-15 present |
| C-03 | AC-3 | Regression | workflow missing `KAMN_E2E_SDK_DIRECT_LIVE` or `--enable-external-execution` | checker fails with deterministic reason code |
| C-04 | AC-3 | Regression | workflow with truncated scenario list | checker fails with deterministic reason code |
| C-05 | AC-4 | Unit | checker reason taxonomy output | stable reason taxonomy + CSV markers |
| C-06 | AC-5 | Functional | `scripts/ci/test_ci_tools.sh` + `docs/ci/strategy.md` | checker test wired + docs marker parity present |

## Test Mapping
- `bash scripts/ci/test_check_e2e_live_workflow_contract.sh`
- `bash scripts/ci/test_ci_tools.sh` (fast mode)

## Success Metrics / Observable Signals
- Workflow drift that removes live markers is caught by CI-tool regression tests.
- SDK-direct live scenario execution remains full-matrix and non-stubbed by contract.
- CI strategy docs encode the new contract lane marker and reason taxonomy.
