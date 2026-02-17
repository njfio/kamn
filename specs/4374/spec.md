# Spec — #4374 Task: Live-Provider + Native-Signer Composite Promotion Gate with CI Smoke Boundaries

Status: Reviewed
Priority: P1
Parent: #4370
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement

Promotion must fail closed when live-provider and native-signer evidence is incomplete or inconsistent, while preserving low-cost deterministic CI smoke behavior.

## Scope

In scope:
- Composite provider+signer evidence gate contracts.
- Deterministic reason taxonomy and mismatch markers.
- CI smoke/local-heavy boundary markers and enforcement.
- Docs parity for composite gate marker definitions.

Out of scope:
- Always-on deep live node orchestration in fast gate.
- Runtime architecture changes outside gate contracts.

## Acceptance Criteria

AC-1: Composite provider/signer gate emits deterministic reason taxonomy and marker outputs.

AC-2: Composite gate fails closed on missing or inconsistent cross-domain evidence.

AC-3: CI smoke/local-heavy boundary behavior is deterministic and auditable.

AC-4: Unit/Functional/Integration/Regression coverage is present and passing.

## Conformance Cases

- C-01 (AC-1, Functional): validation emits deterministic composite taxonomy version + reason-code CSV.
- C-02 (AC-1, Integration): composite marker fields are present in report JSON and stdout.
- C-03 (AC-2, Regression): incomplete provider/signer evidence linkage fails closed with deterministic reason.
- C-04 (AC-2, Regression): partial evidence acceptance is rejected.
- C-05 (AC-3, Functional): CI smoke/local-heavy boundary markers are emitted and docs-aligned.
- C-06 (AC-4, Integration): target script/docs tests and repo gates pass.

## Success Metrics

- No nondeterministic reason outputs for composite gate decisions.
- Promotion cannot pass with incomplete provider/signer evidence.
