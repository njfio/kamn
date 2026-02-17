# Spec: #4362 RED Tests for Quorum Drift and Signature-Decision Outcome Stability

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

The real-node profile checker tests do not yet enforce dedicated signature-decision taxonomy outputs for profile/quorum decisions.

## Scope

In scope:
- RED test assertions for signature-decision taxonomy fields.
- Quorum drift failure mapping assertions.

Out of scope:
- Checker behavior changes (implemented in #4363).

## Acceptance Criteria

AC-1 GO fixture assertions require signature-decision taxonomy markers.
AC-2 Quorum drift failures require mapped signature-decision reason outputs.

## Conformance Cases

- C-01 (AC-1): GO report missing signature-decision taxonomy markers fails contract test.
- C-02 (AC-2): quorum linkage drift failing output requires `signature_decision_reason_codes_value` containing `runtime_signer_quorum_linkage_drift`.

## Success Metrics

- New tests fail before checker implementation and pass after implementation.
