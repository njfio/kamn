# Spec — #4251 Task: Partition Fault Marker Checks and Deterministic Healing Reconciliation Contracts

Status: Implemented
Priority: P1
Parent: #4249
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Partition/rejoin reconciliation policy checks exist, but deterministic fail-closed mismatch classification for healing contracts is incomplete. Operators need stable mismatch categories and parity across checker outputs, contract lane gates, and docs.

## Scope

In scope:
- Deterministic partition-healing policy mismatch reason mapping markers.
- Regression tests for missing marker and nondeterministic mismatch rejection behavior.
- Contract-lane marker enforcement and docs parity updates.

Out of scope:
- Consensus algorithm changes.
- New transport architecture.

## Acceptance Criteria

AC-1: Partition/healing checker validates marker completeness and reconciliation contracts deterministically.

AC-2: Mismatch outcomes fail closed with stable mismatch reason category mapping.

AC-3: Regression tests cover missing markers and nondeterministic mismatch payloads.

AC-4: Lane/docs parity checks enforce deterministic marker contract.

## Conformance Cases

- C-01 (AC-1, Functional): valid dry-run report returns `GO` and deterministic mapping marker set.
- C-02 (AC-2, Regression): missing partition/healing marker yields `NO-GO` with deterministic mismatch reason category.
- C-03 (AC-2, Regression): invalid reconciliation reason-code ordering/dedup yields deterministic mismatch reason category.
- C-04 (AC-3, Regression): repeated mismatch checks emit stable failed-check and mapped-reason outputs.
- C-05 (AC-4, Integration): contract lane and docs gates enforce mismatch mapping markers.

## Success Signals

- Policy output includes deterministic mismatch mapping taxonomy/version/codes/reason markers.
- Contract lane rejects tampered payloads with deterministic fail-closed reasons.
- Strategy/checklist/ops docs and tests remain synchronized.
