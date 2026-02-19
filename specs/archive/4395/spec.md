# Spec — #4395 Subtask: Deterministic Peer Failure Reason Mapping and Multi-Process Validation Hooks

Status: Implemented
Priority: P1
Parent: #4388
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Peer transport governance requires deterministic peer failure reason mapping and auditable multi-process hook markers across validation, policy, and docs parity checks.

## Scope

In scope:
- Deterministic peer reason marker projection in run-lane and policy reports.
- Policy mismatch reasons for tampered peer markers.
- Docs-parity checks for peer reason matrix markers.
- Contract-lane output wiring for peer reason marker evidence.

Out of scope:
- Cluster orchestration redesign.

## Acceptance Criteria

AC-1: Validation and policy outputs include deterministic peer reason markers and taxonomy version.

AC-2: Policy checker fails closed on peer reason marker mismatch with deterministic reason codes.

AC-3: Contract lane verifies required peer reason matrix docs markers.

AC-4: Integration tests preserve deterministic taxonomy-to-gate flow.

## Conformance Cases

- C-01 (AC-1, Functional): validation emits deterministic peer reason markers.
- C-02 (AC-1, Integration): policy emits deterministic peer reason markers.
- C-03 (AC-2, Regression): timeout marker tamper fails with deterministic mismatch reason.
- C-04 (AC-2, Regression): peer-integrity marker tamper fails with deterministic mismatch reason.
- C-05 (AC-3, Integration): docs marker drift fails contract lane.
- C-06 (AC-4, Integration): contract-lane report and policy report preserve stable peer reason outputs.

## Success Metrics

- Stable peer reason markers across repeated runs.
- Deterministic fail-closed rejection on peer reason/dcos marker drift.
