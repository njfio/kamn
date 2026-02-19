# Spec — #4377 Subtask: RED tests for submission/finality evidence lineage gaps

Status: Implemented
Priority: P1
Parent: #4372
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement
Tests must fail when submit/finality evidence paths drift from canonical lineage while still appearing present.

## Scope
In scope:
- RED tests for stale/cross-linked artifact-path lineage.
- Deterministic lineage mismatch reason assertions.

Out of scope:
- Checker implementation logic.

## Acceptance Criteria
AC-1: checker test suite fails on request/submit/finality lineage cross-link drift.

AC-2: contract-lane test suite fails on stale finality lineage drift.

AC-3: failure output expects deterministic lineage mismatch reason markers.

## Conformance Cases
- C-01 (AC-1): request payload artifact path linked to finality artifact path causes fail-closed mismatch.
- C-02 (AC-1): submit artifact path linked to finality artifact path causes fail-closed mismatch.
- C-03 (AC-2): finality artifact path linked to live output path causes fail-closed mismatch.
- C-04 (AC-3): expected reasons are asserted in test output.
