# Spec - Issue #3850

- Title: Subtask: add signature parity policy checker and reason-taxonomy drift contracts
- Parent: #3848
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Signature parity policy evaluation needed deterministic reason-taxonomy enforcement to fail closed on schema/vector drift.

## Objective

Enforce deterministic signature parity policy checking with explicit drift/failure reason markers.

## Scope

In scope:
- Signature parity policy checker schema/final-decision validation.
- Fail-closed checks for invalid reports and missing required reason codes.

Out of scope:
- Runtime feature additions.

## Acceptance Criteria

- AC-1: Policy checker emits deterministic policy schema/final decision for valid reports.
- AC-2: Invalid report structure fails closed with deterministic reason markers.
- AC-3: Missing NO-GO case reason codes fail closed with deterministic reason markers.

## Conformance Cases

- C-01 (AC-1/AC-2/AC-3): `bash scripts/kolme/test_check_signature_parity_policy.sh` passes.

## Success Metrics

- Signature parity policy drift is detected deterministically with stable reason taxonomy markers.
