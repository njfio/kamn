# Spec — #4252 Task: Fork-Choice Finality Taxonomy and Runbook Parity Under Partition Recovery

Status: Reviewed
Priority: P1
Parent: #4249
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Process-isolated libp2p convergence checks emit deterministic fork-choice reason markers, but do
not yet enforce a deterministic finality taxonomy/runbook parity contract. Drift between checker
taxonomy markers and runbook declarations can silently desynchronize operator promotion decisions.

## Scope

In scope:
- Deterministic finality taxonomy/runbook parity markers in convergence lane and policy outputs.
- Fail-closed runbook parity checks and deterministic reason mapping for taxonomy/runbook drift.
- Regression tests for taxonomy drift and runbook marker divergence.
- Release checklist/strategy/runbook documentation parity updates.

Out of scope:
- New transport algorithms or consensus behavior changes.
- Multi-region topology redesign.

## Acceptance Criteria

AC-1: Fork-choice finality reconciliation taxonomy mapping remains deterministic in lane and policy outputs.

AC-2: Runbook parity checks fail closed when required finality taxonomy markers drift.

AC-3: Regression tests cover both taxonomy drift and runbook marker divergence deterministically.

AC-4: Documentation gates (strategy/runbook/release checklist) and doc-contract tests stay synchronized.

## Conformance Cases

- C-01 (AC-1, Functional): baseline GO path emits deterministic finality taxonomy/runbook parity markers.
- C-02 (AC-2, Regression): tampered taxonomy marker payload fails closed with deterministic taxonomy-drift reason.
- C-03 (AC-2, Regression): missing runbook marker parity declaration fails closed with deterministic runbook-parity reason.
- C-04 (AC-3, Regression): repeated drift checks preserve deterministic reason projection ordering.
- C-05 (AC-4, Integration): contract-lane/docs parity gates enforce marker presence and fail-closed reasons.

## Success Signals

- Policy outputs include deterministic finality taxonomy/runbook parity markers and resolved reason-code projection.
- Runbook and release checklist markers stay parity-validated by script tests and doc contract tests.
- Drift paths produce stable fail-closed reasons (`finality_taxonomy_mapping_drift_detected`, `runbook_marker_parity_mismatch`).
