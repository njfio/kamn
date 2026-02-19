# Spec — #4272 Subtask: Red Tests for Protocol Taxonomy Drift and Runbook Marker Divergence

Status: Implemented
Priority: P1
Parent: #4267
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Without explicit regression coverage, taxonomy/runbook drift can silently pass and break deterministic remediation contracts.

## Acceptance Criteria

AC-1: Tests fail on protocol taxonomy drift.

AC-2: Tests fail on runbook marker divergence.

AC-3: Regression checks preserve deterministic fail-closed reason outputs.

## Conformance Cases

- C-01 (AC-1): runbook taxonomy marker drift produces deterministic `protocol_taxonomy_mapping_drift_detected`.
- C-02 (AC-2): runbook section/marker divergence produces deterministic `runbook_marker_parity_mismatch`.
- C-03 (AC-3): regression checks validate deterministic reason output contract.
