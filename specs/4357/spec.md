# Spec: #4357 Multi-Signer Quorum Validation and Signature-Decision Reason Mapping

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

The real-node runtime integration policy checker validates multi-signer profile and quorum constraints, but it does not expose a dedicated deterministic signature-decision taxonomy output contract for profile/quorum decisions. Release and operator review workflows need stable, machine-readable signature-decision evidence.

## Scope

In scope:
- `scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py` signature-decision reason taxonomy outputs.
- Red/green coverage for quorum drift and signature-decision evidence stability.
- Ops documentation markers for multi-signer profile/quorum policy evidence.

Out of scope:
- New signing algorithms.
- External custody/KMS integrations.

## Acceptance Criteria

AC-1 Multi-signer profile/quorum checks remain deterministic and fail-closed.
Given real-node evidence, when profile/quorum contracts drift, the checker must return `NO-GO` with stable reason codes.

AC-2 Dedicated signature-decision taxonomy outputs are deterministic.
Given any checker output, the report must include:
- `signature_decision_reason_taxonomy_version`
- `signature_decision_reason_codes_csv`
- `signature_decision_reason_codes_value=none|<csv>`

AC-3 Signature-decision evidence maps to observed profile/quorum failures.
Given quorum/profile drift failures, `signature_decision_reason_codes_value` must include the relevant deterministic reason code(s).

AC-4 Integration coverage validates signer profile permutations and quorum drift.
Given primary/secondary profile fixtures and quorum-negative proofs, contract-lane tests must enforce deterministic behavior.

## Conformance Cases

- C-01 (AC-2, Conformance): GO report includes signature-decision taxonomy markers and `signature_decision_reason_codes_value=none`.
- C-02 (AC-1, AC-3, Regression): quorum linkage drift mutation fails with `runtime_signer_quorum_linkage_drift` and appears in `signature_decision_reason_codes_value`.
- C-03 (AC-1, AC-3, Regression): attestation quorum shortfall mutation fails with `runtime_signer_attestation_quorum_shortfall` and appears in signature-decision mapping.
- C-04 (AC-4, Integration): primary and secondary profile fixture runs preserve deterministic signature-decision outputs.

## Success Metrics / Observable Signals

- Existing real-node policy checker contract suite remains green.
- New signature-decision taxonomy assertions fail before implementation and pass after implementation.
- Ops configuration docs include signature-decision taxonomy markers and validation references.
