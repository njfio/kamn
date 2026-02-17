# Spec — #4260 Subtask: Implement Finality Evidence Convergence Verifier and Deterministic Promotion Decision Reason Mapping

Status: Reviewed
Priority: P1
Parent: #4253
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

The policy checker does not currently emit explicit promotion-decision reason mapping markers, and
there is no deterministic verifier that confirms policy/lane/source evidence convergence.

## Scope

In scope:
- Add deterministic promotion reason mapping output to libp2p convergence policy.
- Add convergence verifier command that validates lane-policy-source linkage and reason mapping.
- Integrate checker in contract lane outputs.

Out of scope:
- New transport failover logic.
- Non-libp2p runtime contract changes.

## Acceptance Criteria

AC-1: Policy output includes deterministic promotion decision reason mapping markers.

AC-2: Evidence convergence verifier rejects link/tamper drift deterministically.

AC-3: Contract lane surfaces convergence status and deterministic reason taxonomy markers.

## Conformance Cases

- C-01 (AC-1): GO-path policy emits promotion reason mapping markers with reason code `none`.
- C-02 (AC-2): missing/tampered artifacts return deterministic reason taxonomy codes.
- C-03 (AC-2): mapping drift returns `libp2p_finality_promotion_decision_reason_mapping_mismatch`.
- C-04 (AC-3): contract lane output includes convergence marker projection from verifier.
