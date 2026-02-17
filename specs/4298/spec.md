# Spec — #4298 Task: TLS Evidence Completeness Checker and Fail-Closed Release Reason Mapping

Status: Reviewed
Priority: P1
Parent: #4295
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Release promotion must prove TLS-hardening evidence is complete and fresh, with deterministic reason
projection when evidence is missing, stale, or malformed.

## Scope

In scope:
- TLS evidence completeness/freshness validation in go/no-go evidence checks.
- Deterministic fail-closed reason mapping and reason taxonomy outputs.
- Composition with release gate evidence contract lanes.

Out of scope:
- CA or PKI redesign.
- Full deploy orchestration changes outside checker composition.

## Acceptance Criteria

AC-1: TLS evidence checker validates required artifacts and freshness deterministically.

AC-2: Missing/stale/malformed evidence fails closed with stable reason taxonomy outputs.

AC-3: Release go/no-go checker surfaces TLS reason outputs in lane-consumable markers.

AC-4: Unit/Functional/Integration/Regression coverage exists for TLS evidence completeness behavior.

## Conformance Cases

- C-01 (AC-1, Functional): valid TLS evidence manifest yields pass marker and empty reason list.
- C-02 (AC-2, Regression): missing required TLS evidence fails with deterministic missing-artifact reason.
- C-03 (AC-2, Regression): stale TLS evidence timestamp fails with deterministic freshness-window reason.
- C-04 (AC-2, Regression): malformed TLS evidence JSON fails with deterministic invalid-json reason.
- C-05 (AC-3, Integration): release evidence contract lane emits TLS taxonomy version and reason markers.
- C-06 (AC-4, Unit/Functional): targeted parser/normalization tests assert deterministic reason ordering.

## Success Signals

- TLS checker outputs stable fail-closed reason values across repeated runs.
- CI smoke lane catches missing/stale evidence before promotion.
- Release gate artifacts expose normalized TLS reason markers for downstream checks.
