# Spec — #4388 Task: Real Peer Transport Integrity Checks and Deterministic Retry-Timeout Governance

Status: Implemented
Priority: P1
Parent: #4385
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Peer transport promotion evidence must fail closed with deterministic reason mapping for sender-integrity drift and retry-timeout classification, and this evidence must be reproducible across multi-process lane hooks.

## Scope

In scope:
- Deterministic peer-integrity and retry-timeout markers in live transport fault-matrix validation/policy outputs.
- Policy fail-closed checks for tampered peer reason markers.
- Docs-contract parity checks for peer reason matrix markers.
- RED/GREEN policy + contract-lane test coverage.

Out of scope:
- Gossip protocol redesign.
- Runtime transport architecture redesign.

## Acceptance Criteria

AC-1: Peer-integrity and retry-timeout reason markers are emitted deterministically in validation and policy outputs.

AC-2: Policy checker fails closed with deterministic mismatch reasons when peer reason markers drift.

AC-3: Contract lane enforces docs parity for peer reason matrix markers from release checklist and Kolme devnet ops docs.

AC-4: Unit/Functional/Integration/Regression coverage is present and passing.

## Conformance Cases

- C-01 (AC-1, Functional): validation emits deterministic peer reason markers (`peer_integrity_fail_closed_reason_code`, timeout/budget reason markers, taxonomy version).
- C-02 (AC-1, Integration): policy report mirrors deterministic peer reason markers.
- C-03 (AC-2, Regression): tampered timeout/budget marker fails with deterministic policy reason code mismatch.
- C-04 (AC-2, Regression): tampered peer-integrity reason marker fails with deterministic policy reason code mismatch.
- C-05 (AC-3, Integration): contract lane fails when required peer reason docs markers drift.
- C-06 (AC-4, Integration): validation/policy/contract-lane suites and workspace regression suite pass.

## Success Metrics

- Repeated runs produce stable peer reason markers and normalized reason output.
- Docs drift for peer reason matrix is rejected deterministically by contract lane.
