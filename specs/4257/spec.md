# Spec — #4257 Subtask: Red Tests for Finality Taxonomy Drift and Runbook Marker Divergence

Status: Reviewed
Priority: P1
Parent: #4252
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Current convergence policy tests validate baseline and selected tamper paths but do not explicitly
cover finality taxonomy drift projection and runbook marker divergence fail-closed behavior.

## Acceptance Criteria

AC-1: Red tests fail when finality taxonomy markers drift.

AC-2: Red tests fail when required runbook markers diverge or go missing.

AC-3: Drift/divergence failures remain deterministic across repeated runs.

## Conformance Cases

- C-01 (AC-1): taxonomy marker tamper yields deterministic `finality_taxonomy_mapping_drift_detected`.
- C-02 (AC-2): runbook marker removal yields deterministic `runbook_marker_parity_mismatch`.
- C-03 (AC-3): repeated divergence checks preserve deterministic reason projection output.
