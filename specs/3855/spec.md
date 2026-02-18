# Spec - Issue #3855

- Title: Task: automate live-validation release evidence bundle and milestone closure summary
- Parent: #3851
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Manual evidence assembly and closure summary generation are error-prone and slow go/no-go decisions.

## Objective

Automate release evidence bundle generation plus deterministic closure-summary/docs synchronization checks for R27 governance review.

## Scope

In scope:
- Go/no-go evidence bundle generation and fail-closed policy validation.
- Milestone closure summary generation for budget artifacts.
- Docs-contract synchronization checks for closure-governance markers.

Out of scope:
- Core runtime behavior changes.

## Acceptance Criteria

- AC-1: Live-validation release evidence bundle automation remains deterministic.
- AC-2: Artifact policy checks fail closed for tamper/drift scenarios.
- AC-3: Milestone closure summary generation is deterministic and auditable.
- AC-4: Docs-contract synchronization checks remain green for closure-governance markers.

## Conformance Cases

- C-01 (AC-1/AC-2): `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` passes.
- C-02 (AC-3): `bash scripts/ci/test_summarize_budget_artifacts.sh` passes.
- C-03 (AC-4): `bash scripts/ci/test_ci_strategy_contract.sh` passes.

## Success Metrics

- Release evidence and closure-summary flows are automated and deterministic.
- Governance marker drift remains fail-closed through contract checks.
