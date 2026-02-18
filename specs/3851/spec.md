# Spec - Issue #3851

- Title: Story: unify governance budgets and automate release evidence closure for R27
- Parent: #3815
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Budget signals and release evidence flows spanned multiple surfaces, increasing closure risk and audit overhead.

## Objective

Close the story with deterministic governance budget reporting and release-evidence automation coverage across script/harness/ignored-test and go/no-go artifact-pack surfaces.

## Scope

In scope:
- Unified governance budget reporting/policy and ignored-test integration (`#3852` chain).
- Automated release evidence bundle and closure summary/docs synchronization (`#3855` chain).
- Story-level consolidated verification and lifecycle closure artifacts.

Out of scope:
- New protocol/runtime feature development unrelated to governance closure.

## Acceptance Criteria

- AC-1: Unified governance budget reporting and threshold-policy checks remain deterministic.
- AC-2: Release evidence bundle generation/policy checks remain deterministic and fail closed.
- AC-3: Closure summary/docs-contract synchronization checks remain green.
- AC-4: Story-level conformance coverage remains auditable and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` and `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh` pass.
- C-02 (AC-2): `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` passes.
- C-03 (AC-3): `bash scripts/ci/test_summarize_budget_artifacts.sh` and `bash scripts/ci/test_ci_strategy_contract.sh` pass.
- C-04 (AC-4): consolidated checks above pass in the story closure run.

## Success Metrics

- Governance budgets and release-evidence packaging remain deterministic and fail closed.
- Story closure is fully traceable with AC-to-test mapping.
