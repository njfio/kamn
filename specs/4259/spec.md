# Spec — #4259 Subtask: Red Tests for Finality Evidence Convergence Completeness and Tamper Rejection

Status: Reviewed
Priority: P1
Parent: #4253
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Current libp2p convergence tests do not fail on missing evidence links, artifact tamper, or
promotion-reason mapping drift at the evidence-convergence boundary.

## Scope

In scope:
- Add failing tests that assert evidence-convergence GO-path markers.
- Add tamper and missing-link tests that assert deterministic NO-GO reasons.

Out of scope:
- Runtime behavior changes beyond test additions.

## Acceptance Criteria

AC-1: Red tests fail before convergence checker and mapping wiring is implemented.

AC-2: Tests assert deterministic fail-closed reasons for link-missing and payload tamper paths.

AC-3: Tests assert deterministic fail-closed reason for promotion reason mapping drift.

## Conformance Cases

- C-01 (AC-1): expected convergence checker command/markers missing before implementation.
- C-02 (AC-2): missing source report link yields `libp2p_finality_evidence_link_missing:source_report_file`.
- C-03 (AC-2): tampered payload yields `libp2p_finality_evidence_payload_tamper_detected:<field>`.
- C-04 (AC-3): tampered promotion reason mapping yields `libp2p_finality_promotion_decision_reason_mapping_mismatch`.
