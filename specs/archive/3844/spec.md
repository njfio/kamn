# Spec - Issue #3844

- Title: Story: validate live libp2p+Kolme convergence and finality under local-heavy triadic topology
- Parent: #3815
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Combined libp2p+Kolme readiness required deterministic triadic topology and signer realism evidence for release promotion.

## Objective

Close the story with deterministic triadic convergence/finality and signature realism contract coverage across completed child tasks.

## Scope

In scope:
- Triadic topology orchestration and live-proof artifact schema contracts (`#3845` chain).
- Real secp256k1 signer profile parity and policy contracts (`#3848` chain).
- Story-level consolidated verification and lifecycle closure.

Out of scope:
- Public network rollout and unrelated protocol work.

## Acceptance Criteria

- AC-1: Triadic local-heavy convergence/finality contract lanes remain deterministic.
- AC-2: Signature realism/parity contract lanes remain deterministic and fail closed.
- AC-3: Required docs/marker parity checks remain enforced in contract suites.
- AC-4: Story-level conformance coverage is auditable and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh` and `bash scripts/kolme/test_run_local_live_node_validation_bundle_contract_lane.sh` pass.
- C-02 (AC-2): `bash scripts/kolme/test_run_signature_parity_contract_lane.sh` passes.
- C-03 (AC-3/AC-4): consolidated checks above pass in story closure run.

## Success Metrics

- Local-heavy triadic + signer-parity readiness evidence remains deterministic, fail-closed, and auditable.
