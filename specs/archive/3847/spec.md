# Spec - Issue #3847

- Title: Subtask: define combined live-proof artifact schema for convergence and finality checkpoints
- Parent: #3845
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Combined live-proof evidence needed deterministic schema and checkpoint marker contracts for convergence/finality promotion decisions.

## Objective

Enforce combined live-node validation bundle schema and policy-marker contracts for convergence/finality checkpoints.

## Scope

In scope:
- Contract-lane wiring and manifest coverage for live-node validation bundle flow.
- Required artifact lineage markers (rollback/recovery) and policy GO path checks.
- Docs parity checks for required runner/policy/marker snippets.

Out of scope:
- New runtime behavior beyond contract coverage.

## Acceptance Criteria

- AC-1: Live-node validation bundle contract lane wiring and coverage markers remain valid.
- AC-2: Summary/policy schemas and deterministic marker fields are validated in contract tests.
- AC-3: Docs parity for required bundle markers and regression notes remains enforced.

## Conformance Cases

- C-01 (AC-1/AC-2/AC-3): `bash scripts/kolme/test_run_local_live_node_validation_bundle_contract_lane.sh` passes.

## Success Metrics

- Combined live-proof artifact schema/contracts remain deterministic and fail closed on marker drift.
