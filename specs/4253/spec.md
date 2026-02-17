# Spec — #4253 Task: Finality Evidence Convergence Checker Across Partition Lane and Promotion Artifacts

Status: Implemented
Priority: P1
Parent: #4250
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

The process-isolated libp2p convergence flow validates lane and policy artifacts independently, but
does not enforce deterministic evidence-link convergence between the contract-lane report, policy
report, and source summary report. Promotion reason mapping can drift without a dedicated
convergence verifier.

## Scope

In scope:
- Add deterministic evidence-convergence verification for libp2p process-isolated convergence artifacts.
- Add deterministic promotion-decision reason mapping markers to policy outputs.
- Fail closed on missing links, payload tamper, and reason-mapping drift.
- Add red/regression tests and contract-lane integration for convergence verification.
- Update runbook/checklist/planning docs and doc-contract tests for marker parity.

Out of scope:
- Consensus algorithm changes.
- New transport runtime behavior beyond governance/checker contracts.

## Acceptance Criteria

AC-1: A dedicated convergence checker validates report/policy/source-report linkage deterministically.

AC-2: Missing or tampered evidence artifacts fail closed with stable reasons.

AC-3: Promotion decision reason mapping is deterministic and parity-validated in policy and convergence outputs.

AC-4: Contract-lane output, docs, and regression tests include convergence markers and fail-closed reasons.

## Conformance Cases

- C-01 (AC-1, Functional): GO-path convergence check returns deterministic markers and `final_decision=GO`.
- C-02 (AC-2, Regression): missing evidence link fails with `libp2p_finality_evidence_link_missing:<field>`.
- C-03 (AC-2, Regression): tampered payload fails with `libp2p_finality_evidence_payload_tamper_detected:<field>`.
- C-04 (AC-3, Regression): promotion decision mapping drift fails with `libp2p_finality_promotion_decision_reason_mapping_mismatch`.
- C-05 (AC-4, Integration): contract-lane and docs parity include convergence checker command + taxonomy markers.

## Success Signals

- New convergence checker emits deterministic taxonomy markers with stable reason ordering.
- Contract-lane appends convergence status and reason-mapping parity fields.
- Red tests from #4259 fail before implementation and pass after implementation.
