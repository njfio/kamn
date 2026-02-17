# Milestone Spec Index — R27.26 Multi-node Partition-Healing and Finality-Convergence Governance

Milestone: R27.26 Multi-node partition-healing and finality-convergence governance  
GitHub Milestone: https://github.com/njfio/kamn/milestone/60

## Scope

- Deterministic partition fault and healing marker contracts.
- Finality convergence evidence governance and promotion decision mapping.
- Fail-closed taxonomy and runbook parity checks.
- Low-cost CI smoke boundaries with explicit local-heavy partition drills.

## Issue Hierarchy

- Epic: #4248
- Story: #4249
- Task: #4251
- Subtasks: #4255, #4256

## Required Artifacts

- `specs/4251/spec.md`, `specs/4251/plan.md`, `specs/4251/tasks.md`
- `specs/4255/spec.md`, `specs/4255/plan.md`, `specs/4255/tasks.md`
- `specs/4256/spec.md`, `specs/4256/plan.md`, `specs/4256/tasks.md`

## Validation Baseline

- Runtime lane + policy scripts:
  - `scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh`
  - `scripts/runtime/check_block_reconciliation_partition_rejoin_live_policy.sh`
  - `scripts/runtime/validate_block_reconciliation_partition_rejoin_live_contract_lane.sh`
- Runtime contract tests:
  - `scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live.sh`
  - `scripts/runtime/test_check_block_reconciliation_partition_rejoin_live_policy.sh`
  - `scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh`

## Status

In Progress
