# Spec — #4304 Subtask: RED Tests for TLS Evidence Completeness Failures and Stale-Artifact Rejection

Status: Reviewed
Priority: P1
Parent: #4298
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

TLS governance can silently regress if missing or stale artifacts are accidentally treated as valid.

## Scope

In scope:
- RED tests for missing TLS evidence, stale evidence, and malformed evidence handling.
- Deterministic assertions for fail-closed reason markers.

Out of scope:
- Implementing checker behavior changes.

## Acceptance Criteria

AC-1: Tests fail when required TLS evidence artifacts are absent.

AC-2: Tests fail when stale TLS evidence is accepted.

AC-3: Regression tests assert deterministic fail-closed reason markers.

## Conformance Cases

- C-01 (AC-1, Functional): missing TLS evidence path yields deterministic missing-artifact reason marker.
- C-02 (AC-2, Regression): stale evidence timestamp yields deterministic freshness-window reason marker.
- C-03 (AC-3, Regression): malformed evidence JSON yields deterministic invalid-json reason marker.

## Success Signals

- Tests fail before implementation and pass after reason projection/checker updates.
