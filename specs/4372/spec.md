# Spec — #4372 Task: live submission/finality evidence convergence and provider failure taxonomy

Status: Implemented
Priority: P1
Parent: #4369
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement
Promotion gating must fail closed when runtime submit/finality evidence lineage drifts or provider failure reason outputs become unstable/non-deterministic.

## Scope
In scope:
- Submit/finality evidence lineage validation hardening in local runtime commit live policy checks.
- Deterministic provider failure taxonomy outputs for policy reports.
- Contract-lane + docs parity updates.

Out of scope:
- Runtime transport/provider backend redesign.
- Multi-chain finality modeling.

## Acceptance Criteria
AC-1: Submission/finality lineage drift is rejected with deterministic reason markers.

AC-2: Provider failure taxonomy is emitted deterministically by the policy checker.

AC-3: Contract-lane tests cover lineage drift and provider taxonomy output parity.

AC-4: Docs and docs-contract tests capture provider/finality reason matrix and lineage-failure cases.

AC-5: Unit/Functional/Integration/Regression verification passes.

## Conformance Cases
- C-01 (AC-1, Functional): checker rejects run-mode reports when request/submit/finality artifact paths are cross-linked incorrectly.
- C-02 (AC-1, Regression): checker emits deterministic lineage mismatch reasons for stale artifact linkage.
- C-03 (AC-2, Functional): checker output includes provider failure taxonomy version, codes CSV, and normalized codes value.
- C-04 (AC-3, Integration): finality evidence contract lane fails closed on lineage drift and validates taxonomy output parity.
- C-05 (AC-4, Integration): release/go-no-go and planning docs include new taxonomy/lineage markers and are asserted by docs tests.
- C-06 (AC-5, Integration): targeted script checks and repo gates pass.

## Success Metrics
- No GO decision on stale/mismatched submit/finality lineage.
- Provider-failure taxonomy fields stable across repeated runs.
