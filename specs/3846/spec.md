# Spec - Issue #3846

- Title: Subtask: add three-node orchestration harness with deterministic churn and recovery drills
- Parent: #3845
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Readiness validation required deterministic triadic topology coverage for churn/recovery behavior beyond single-path checks.

## Objective

Provide deterministic three-node orchestration contract coverage for triadic devnet smoke/convergence behavior.

## Scope

In scope:
- Triadic devnet smoke contract-lane wrapper/manifest/implementation checks.
- Deterministic schema and PASS decision assertions for triadic validation output.

Out of scope:
- Protocol redesign.

## Acceptance Criteria

- AC-1: Triadic contract lane wiring (wrapper/manifest/implementation markers) is valid.
- AC-2: Contract lane emits deterministic success marker and valid report schema.
- AC-3: Triadic report final decision is deterministic (`PASS`) for baseline path.

## Conformance Cases

- C-01 (AC-1/AC-2/AC-3): `bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh` passes.

## Success Metrics

- Triadic orchestration coverage remains executable and deterministic for local-heavy readiness proofs.
