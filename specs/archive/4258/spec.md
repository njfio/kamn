# Spec — #4258 Subtask: Finality Taxonomy Enforcement and Runbook Marker Parity Checks

Status: Implemented
Priority: P1
Parent: #4252
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Finality convergence checker outputs currently expose taxonomy markers but do not enforce
checker-to-runbook parity as a fail-closed policy contract.

## Acceptance Criteria

AC-1: Checker and policy outputs include deterministic finality taxonomy/runbook parity markers.

AC-2: Policy checker validates runbook marker parity and fails closed on drift.

AC-3: Contract-lane, strategy, runbook, and release checklist surfaces enforce the same marker contract.

## Conformance Cases

- C-01 (AC-1): GO path emits new parity markers and `finality_taxonomy_runbook_reason_code=none`.
- C-02 (AC-2): taxonomy drift and runbook divergence emit deterministic fail-closed reasons.
- C-03 (AC-3): contract lane and docs parity tests verify required marker surface.
