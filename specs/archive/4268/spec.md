# Spec — #4268 Task: Websocket Session Lifecycle Evidence Convergence Checker

Status: Implemented
Priority: P1
Parent: #4265
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Websocket session promotion decisions require deterministic convergence between validation summary evidence, policy checker outputs, and contract-lane artifacts. Without a dedicated convergence gate, missing links or tampered payloads can pass with ambiguous decision reasons.

## Scope

In scope:
- Evidence convergence checker for websocket session lifecycle artifacts.
- Deterministic fail-closed reason taxonomy for missing-link and tamper conditions.
- Deterministic promotion decision reason mapping markers.
- Contract-lane integration + docs contract updates.

Out of scope:
- Websocket protocol redesign.
- CI topology changes beyond this lane's evidence convergence.

## Acceptance Criteria

AC-1: Convergence checker validates websocket evidence linkage across report/policy/gate artifacts.

AC-2: Missing or tampered evidence fails closed with deterministic reason markers.

AC-3: Promotion decision reason mapping markers are deterministic and stable across runs.

AC-4: Functional/integration/regression coverage validates convergence pass/fail behavior.

## Conformance Cases

- C-01 (AC-1, Functional): valid websocket lane report + policy report converge with `GO`.
- C-02 (AC-2, Regression): missing `source_report_file` evidence link is rejected deterministically.
- C-03 (AC-2, Regression): tampered policy payload shape is rejected deterministically.
- C-04 (AC-3, Functional): promotion reason mapping markers are emitted with stable taxonomy/csv and `promotion_decision_reason_code`.
- C-05 (AC-4, Integration): websocket contract lane composes validation, policy, and convergence checker outputs.
- C-06 (AC-4, Conformance): docs and docs-contract tests enforce convergence command + marker parity.

## Success Signals

- Convergence checker emits stable reason ordering and deterministic `NO-GO` reasons.
- Websocket policy + lane reports include deterministic promotion reason mapping markers.
- Planning/release checklist docs remain synchronized via contract tests.
