# Spec — #4376 Subtask: deterministic in-memory provider rejection reason mapping

Status: Reviewed
Priority: P1
Parent: #4371
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement
Operators require stable, auditable reason codes when production-mode provider command surfaces drift
back to in-memory references.

## Scope
In scope:
- Deterministic policy reason mapping for in-memory provider references.
- Docs parity for release/go-no-go and configuration references.

Out of scope:
- Transport/provider codepath redesign.

## Acceptance Criteria
AC-1: Policy checker emits deterministic in-memory provider rejection reason.

AC-2: In-memory marker drift always yields NO-GO.

AC-3: Docs and docs-tests reference the deterministic reason marker.

## Conformance Cases
- C-01 (AC-1, Functional): checker output includes `runtime_commit_in_memory_provider_reference_detected` when command marker contains in-memory provider.
- C-02 (AC-2, Regression): contract-lane tamper flow fails closed with NO-GO.
- C-03 (AC-3, Integration): docs/tests pass with updated marker references.
