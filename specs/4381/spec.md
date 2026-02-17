# Spec — #4381 Subtask: RED tests for live-provider/native-signer evidence convergence gaps

Status: Implemented
Priority: P1
Parent: #4374
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement

Composite promotion gates need deterministic failing coverage for missing or partial provider/signer evidence linkage.

## Scope

In scope:
- Add failing assertions for missing/incomplete provider+signer marker combinations.
- Add deterministic regression assertions for partial-evidence rejection.

Out of scope:
- Green-path implementation changes.

## Acceptance Criteria

AC-1: Tests fail when provider/signer linkage evidence is incomplete.

AC-2: Tests fail when composite gate accepts partial evidence.

AC-3: Regression failure messages remain deterministic.

## Conformance Cases

- C-01 (AC-1, Functional): missing provider marker fails composite check test.
- C-02 (AC-1, Functional): missing signer marker fails composite check test.
- C-03 (AC-2, Regression): partial evidence acceptance path is rejected deterministically.
