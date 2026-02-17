# Spec — #4332 Subtask: add red tests for in-flight drain timeout violations and checkpoint reconciliation drift

Status: Reviewed
Priority: P1
Parent: #4327
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

The full-runtime shutdown contract currently validates marker presence but does not reject contradictory drain/timeout metadata. This allows graceful-shutdown evidence to claim a completed drain while encoded ticks imply timeout behavior, which can hide reconciliation drift.

## Scope

In scope:
- Add red tests for shutdown drain-timeout contract violations.
- Add red tests for checkpoint reconciliation reason/counter drift.
- Add regression assertions for deterministic fail-closed reason codes.

Out of scope:
- New storage persistence or checkpoint backend design.
- Redesign of daemon completion reason format.

## Acceptance Criteria

AC-1 (Given/When/Then): Given a graceful shutdown completion reason where `drain_ticks > timeout_ticks`, when the full supervisor stop classifier evaluates it, then it fails closed with a deterministic reason code.

AC-2 (Given/When/Then): Given shutdown completion metadata with invalid numeric reconciliation fields, when classifier validation runs, then it fails closed with deterministic reason codes.

AC-3 (Given/When/Then): Given checkpoint reason/counter parity drift, when reconciliation validation runs, then it fails closed with deterministic reason codes.

AC-4 (Given/When/Then): Given the checker implementation is applied, when targeted runtime tests execute, then all newly added red tests pass as deterministic regressions.

## Conformance Cases

- C-01 (AC-1, Unit): graceful shutdown with contradictory drain/timeout ticks is rejected.
- C-02 (AC-2, Unit): invalid `drain_ticks`/`timeout_ticks`/`ignored_signals` values are rejected.
- C-03 (AC-3, Unit): timeout reason-code mapping drift is rejected.
- C-04 (AC-3, Regression): checkpoint counters inconsistent with completion reason are rejected.
- C-05 (AC-4, Functional/Integration): full runtime timeout lane preserves deterministic checkpoint evidence markers after checker enforcement.

## Success Metrics / Observable Signals

- New tests for C-01..C-04 fail before implementation and pass after implementation.
- Runtime test lanes remain deterministic and green in targeted scope.
