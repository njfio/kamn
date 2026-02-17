# Spec: #4363 Implement Quorum Checker Signature-Decision Taxonomy Outputs

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

The checker needs a dedicated deterministic signature-decision taxonomy (profile/quorum focus) to support stable operator evidence and release gating.

## Scope

In scope:
- Add signature-decision taxonomy constants and output projection.
- Map observed reason codes into deterministic signature-decision reason value output.
- Update docs marker references.

Out of scope:
- New quorum algorithms.

## Acceptance Criteria

AC-1 Checker emits stable signature-decision taxonomy metadata fields.
AC-2 Checker emits deterministic observed signature-decision reason value (`none|<csv>`).
AC-3 Quorum/profile failure reasons are reflected in signature-decision output value.

## Conformance Cases

- C-01 (AC-1): taxonomy version and codes CSV fields always present.
- C-02 (AC-2): GO output has `signature_decision_reason_codes_value=none`.
- C-03 (AC-3): quorum drift/shortfall outputs include corresponding reason in value CSV.

## Success Metrics

- Existing checker behavior remains fail-closed.
- New deterministic signature-decision markers are present in JSON + stdout contract output.
