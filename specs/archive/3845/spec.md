# Spec - Issue #3845

- Title: Task: implement triadic local topology orchestrator and convergence/finality live lane
- Parent: #3844
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Combined libp2p+Kolme readiness required triadic topology orchestration and deterministic convergence/finality artifact contracts.

## Objective

Close triadic topology orchestration and combined live-proof artifact schema coverage through deterministic contract-lane validation.

## Scope

In scope:
- Triadic three-node orchestration contract lane validation (`#3846`).
- Combined live-node validation bundle schema/policy/docs contract validation (`#3847`).
- Task-level closure artifacts and consolidated verification.

Out of scope:
- Protocol redesign and unrelated runtime feature work.

## Acceptance Criteria

- AC-1: Triadic local topology orchestration contract lane remains deterministic.
- AC-2: Combined live-proof convergence/finality bundle schema/policy contracts remain deterministic.
- AC-3: Docs parity and regression markers for bundle contracts remain enforced.
- AC-4: Task-level conformance coverage remains passing and auditable.

## Conformance Cases

- C-01 (AC-1): `bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh` passes.
- C-02 (AC-2/AC-3): `bash scripts/kolme/test_run_local_live_node_validation_bundle_contract_lane.sh` passes.
- C-03 (AC-4): consolidated checks above pass in task closure run.

## Success Metrics

- Triadic topology and live-proof artifact contracts remain deterministic and fail closed on drift.
