# Spec — #4227 Subtask: Red Tests for Admission Taxonomy Drift and Runbook Marker Divergence

Status: Implemented
Priority: P1
Parent: #4222
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement
Without explicit red tests, admission decision taxonomy and runbook parity changes can regress silently and reintroduce aspirational/incomplete merges.

## Scope
In scope:
- Add failing/tamper tests for admission decision taxonomy drift in policy checks.
- Add failing/tamper tests for runbook marker divergence in contract-lane checks.

Out of scope:
- Implementing final production marker emission/checking logic.

## Acceptance Criteria
AC-1: Policy red tests fail deterministically on admission decision taxonomy drift.
AC-2: Lane red tests fail deterministically on runbook marker divergence.
AC-3: Regression fixtures preserve deterministic reason-code outputs.

## Conformance Cases
- C-01 (AC-1, Regression): tampered admission decision taxonomy version fails with deterministic mismatch reason.
- C-02 (AC-1, Regression): tampered admission decision reason-codes csv fails with deterministic mismatch reason.
- C-03 (AC-2, Regression): runbook taxonomy marker drift fixture fails with deterministic reason.
- C-04 (AC-2, Regression): runbook marker divergence fixture fails with deterministic reason.
- C-05 (AC-3, Regression): repeated tampered runs keep deterministic reason-code ordering.
