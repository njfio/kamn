# Spec - Issue #3848

- Title: Task: enforce signature realism and managed-signer parity in live validation matrix
- Parent: #3844
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Simulated signature pathways were insufficient for production-readiness evidence; live validation needed real signer profile parity and deterministic policy contracts.

## Objective

Close signature realism/managed-signer parity task with deterministic matrix, policy checker, and contract-lane coverage.

## Scope

In scope:
- Real secp256k1 signer profile parity matrix checks (`#3849`).
- Signature parity policy checker and reason-taxonomy drift contracts (`#3850`).
- Task-level conformance closure artifacts.

Out of scope:
- New wallet integrations.

## Acceptance Criteria

- AC-1: Real signer profile matrix checks remain deterministic and schema-validated.
- AC-2: Parity policy checker enforces deterministic fail-closed taxonomy behavior.
- AC-3: Contract-lane composition for matrix + policy + docs parity remains passing.
- AC-4: Task-level conformance coverage remains auditable and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/kolme/test_run_signature_parity_matrix.sh` passes.
- C-02 (AC-2): `bash scripts/kolme/test_check_signature_parity_policy.sh` passes.
- C-03 (AC-3/AC-4): `bash scripts/kolme/test_run_signature_parity_contract_lane.sh` passes.

## Success Metrics

- Signature realism evidence is deterministic and fail-closed across matrix, policy, and composed contract lanes.
