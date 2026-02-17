# Spec — #4287 Subtask: Add Red Tests for Drift Taxonomy Drift and Runbook Marker Divergence

Status: Implemented
Priority: P1
Parent: #4282
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

The failover preflight test surface needs explicit red tests proving taxonomy and runbook marker parity drift are rejected deterministically.

## Scope

In scope:
- Red tests for taxonomy-marker drift rejection.
- Red tests for runbook marker divergence rejection.
- Red tests for deterministic repeated mismatch output stability.

Out of scope:
- Runbook workflow redesign.

## Acceptance Criteria

AC-1: Tests fail on drift taxonomy divergence.

AC-2: Tests fail on runbook marker divergence.

AC-3: Regression tests preserve deterministic taxonomy/runbook parity outputs.

## Conformance Cases

- C-01 (AC-1, Regression): drifted taxonomy mapping marker fails with deterministic taxonomy-drift reason.
- C-02 (AC-2, Regression): missing runbook marker declaration fails with deterministic runbook-parity reason.
- C-03 (AC-3, Regression): repeated checks over identical taxonomy/runbook drift fixture preserve deterministic reason ordering.

## Success Signals

- New tests fail before taxonomy/runbook parity enforcement implementation.
- Tests pass after checker implementation and remain as regression guards.
