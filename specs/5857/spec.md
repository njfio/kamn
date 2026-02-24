# Spec: Issue #5857 - Enforce Push-Executed Live E2E CI Lane

- Issue: #5857
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The repository has a real live E2E workflow (`.github/workflows/e2e-live.yml`) with Kolme + `kamn-node` orchestration, but it is schedule/manual-only. That leaves a confidence gap because normal integration flow on `main` does not guarantee a live execution. R57 reports this as a high-priority gap.

## Scope
In scope:
- Add deterministic push trigger coverage for `main` in `e2e-live.yml`.
- Ensure at least the SDK-direct live lane executes for push-triggered runs.
- Add fail-closed workflow contract checks for push trigger markers and main-branch targeting.
- Update CI strategy markers to reflect the expanded contract taxonomy.

Out of scope:
- Branch protection/ruleset administration outside repository files.
- Changes to external Kolme repository code.
- Expanding scenario matrix beyond existing S-01..S-15 coverage.

## Acceptance Criteria
- AC-1: `.github/workflows/e2e-live.yml` declares deterministic `push` trigger scope for `main`, while preserving schedule/manual triggers.
- AC-2: Push-triggered execution path includes a live lane that runs full SDK-direct S-01..S-15 probes against locally bootstrapped Kolme + KAMN processes.
- AC-3: Workflow contract tests fail closed when `push` trigger markers or `main` branch scoping markers are removed.
- AC-4: CI strategy documentation contains updated E2E live workflow marker taxonomy matching code-level contract checks.
- AC-5: Targeted unit/functional/regression checks for touched workflow-contract surfaces pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Repository `e2e-live.yml` triggers block | Contains `push:`, `branches:`, `- main`, `schedule`, and `workflow_dispatch` markers |
| C-02 | AC-2 | Functional/Integration | SDK-direct job block under push-enabled workflow | Keeps `KAMN_E2E_SDK_DIRECT_LIVE: "1"`, external execution marker, and full S-01..S-15 matrix |
| C-03 | AC-3 | Regression | Mutated workflow removing `push` trigger | Contract decision includes `push_trigger_missing` and fails NO-GO |
| C-04 | AC-3 | Regression | Mutated workflow removing `- main` branch scope | Contract decision includes `push_main_branch_scope_missing` and fails NO-GO |
| C-05 | AC-4 | Functional/Regression | `docs/ci/strategy.md` E2E live section | Updated reason-codes marker includes push-trigger reason codes |
| C-06 | AC-5 | Verification | Targeted cargo tests for workflow contract files | All pass |

## Test Mapping
- `cargo test -p kamn-core --test e2e_live_workflow_lane`
- `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract`

## Success Metrics / Observable Signals
- Live E2E workflow is no longer schedule/manual-only.
- Contract tests enforce push-trigger guardrails and fail closed on drift.
- Strategy marker taxonomy and code checks remain synchronized.
