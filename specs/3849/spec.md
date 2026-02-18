# Spec - Issue #3849

- Title: Subtask: add real secp256k1 signer profile matrix checks for managed and failover paths
- Parent: #3848
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Production-readiness validation required deterministic real secp256k1 signer-profile matrix checks, including known-bad failover-path proofs.

## Objective

Provide deterministic signature parity matrix coverage for managed/failover signer paths and required negative vectors.

## Scope

In scope:
- Signature parity matrix runner contract coverage and schema checks.
- Required negative-vector reason-code assertions for bad signature/recovery/pubkey cases.

Out of scope:
- New wallet integrations.

## Acceptance Criteria

- AC-1: Signature parity matrix runner emits deterministic schema/status markers.
- AC-2: Managed/failover negative vectors are present and fail closed with required reason codes.
- AC-3: Matrix runner supports bounded case execution and invalid-fixture fail-closed behavior.

## Conformance Cases

- C-01 (AC-1/AC-2/AC-3): `bash scripts/kolme/test_run_signature_parity_matrix.sh` passes.

## Success Metrics

- Signer-profile matrix proofs remain deterministic and auditable for real secp256k1 parity paths.
