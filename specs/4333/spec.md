# Spec — #4333 Subtask: implement shutdown checkpoint reconciliation checker and fail-closed timeout reason mapping

Status: Reviewed
Priority: P1
Parent: #4327
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

Shutdown evidence currently lacks an explicit reconciliation checker that validates completion-reason class, timeout reason-code mapping, and checkpoint-failure counters as one deterministic contract. Without this, drift can silently pass.

## Scope

In scope:
- Implement deterministic shutdown checkpoint reconciliation checker.
- Enforce fail-closed timeout reason mapping.
- Integrate checker into daemon/full shutdown execution path.
- Update ops/release docs with reconciliation failure taxonomy.

Out of scope:
- Redesign of observability telemetry model.
- New runtime modes or persistence systems.

## Acceptance Criteria

AC-1 (Given/When/Then): Given any daemon completion reason class (`graceful-shutdown`, `graceful-shutdown-timeout`, `tick-budget-exhausted`), when reconciliation validation runs, then reason-code and checkpoint counters are validated deterministically.

AC-2 (Given/When/Then): Given timeout completion outcomes, when reason mapping drifts from timeout policy or checkpoint counters drift, then execution fails closed with stable reason codes.

AC-3 (Given/When/Then): Given full-runtime shutdown execution, when timeout completion occurs, then report/log evidence preserves deterministic reconciliation fields.

AC-4 (Given/When/Then): Given docs and governance contracts, when shutdown reconciliation policy changes, then `docs/ops/configuration.md` and `docs/foundation/release-gonogo-checklist.md` are updated with reason taxonomy references.

## Conformance Cases

- C-01 (AC-1, Unit): classifier accepts valid graceful/timeout/not-signaled mappings.
- C-02 (AC-2, Unit): timeout reason-code drift returns deterministic fail-closed reason.
- C-03 (AC-2, Regression): checkpoint counter drift returns deterministic fail-closed reason.
- C-04 (AC-3, Integration): full runtime timeout flow preserves expected checkpoint evidence fields.
- C-05 (AC-4, Conformance): docs include required reconciliation taxonomy markers.

## Success Metrics / Observable Signals

- All C-01..C-05 tests pass with deterministic reason assertions.
- No regression in targeted `kamn-node` and docs contract tests.
