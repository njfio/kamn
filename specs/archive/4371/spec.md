# Spec — #4371 Task: production-mode live provider enforcement and in-memory commit-path rejection

Status: Implemented
Priority: P1
Parent: #4369
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement
Production-mode runtime integration must fail closed when in-memory provider paths are accepted so
promotion cannot pass with non-live provider behavior.

## Scope
In scope:
- Deterministic rejection reason mapping for in-memory provider references in production-mode checks.
- Contract-lane regression coverage for in-memory provider drift.
- Docs/runbook marker parity for operator-facing reason expectations.

Out of scope:
- Runtime transport/provider architecture redesign.
- New deep-lane orchestration.

## Acceptance Criteria
AC-1: Production-mode policy rejects in-memory provider references with deterministic reason codes.

AC-2: Contract-lane tests fail when in-memory references are accepted.

AC-3: Documentation surfaces include deterministic provider rejection markers.

AC-4: Unit/Functional/Integration/Regression gates pass for this slice.

## Conformance Cases
- C-01 (AC-1, Functional): policy checker emits `runtime_commit_in_memory_provider_reference_detected` on in-memory command marker drift.
- C-02 (AC-1, Functional): policy checker emits deterministic taxonomy/marker outputs in JSON report.
- C-03 (AC-2, Regression): contract-lane tamper case fails closed when in-memory provider marker is injected.
- C-04 (AC-3, Integration): docs + docs-contract tests assert in-memory rejection marker presence.
- C-05 (AC-4, Integration): targeted script/docs tests and repo gates pass.

## Success Metrics
- No GO decision when in-memory provider reference exists in production-mode contract path.
- Deterministic reason code stability across reruns.
