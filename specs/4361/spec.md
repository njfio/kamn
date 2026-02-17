# Spec: #4361 Key-Source Reason Mapping and Strict Fallback Rejection

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

The runtime integration policy checker must publish stable, machine-readable key-source/fallback reason mapping outputs, while enforcing strict explicit key-source contracts.

## Scope

In scope:
- Policy checker enforcement updates for key-source contract version and command marker requirements.
- Deterministic taxonomy output fields for key-source/fallback reason reporting.
- Docs marker updates for operations and release evidence.

Out of scope:
- New runtime transport/provider behavior.

## Acceptance Criteria

AC-1 Policy enforces `runtime_signer_key_source_contract_version=v1` in summary and contracts payload.
AC-2 Policy enforces explicit runtime command signer key-source marker for real-node profile.
AC-3 Policy report emits deterministic key-source reason taxonomy markers and observed reason mapping.

## Conformance Cases

- C-01 (AC-1): summary missing key-source contract version => `runtime_signer_key_source_contract_version_missing`.
- C-02 (AC-1): contracts key-source contract-version drift => `runtime_signer_key_source_contract_version_contract_mismatch`.
- C-03 (AC-2): command marker missing => `runtime_commit_signer_key_source_marker_missing`.
- C-04 (AC-3): GO report outputs:
  - `key_source_reason_taxonomy_version=<stable>`
  - `key_source_reason_codes_csv=<stable>`
  - `key_source_reason_codes_value=none`
- C-05 (AC-3): NO-GO key-source violation output sets `key_source_reason_codes_value=<csv subset>`.

## Success Metrics

- Deterministic outputs visible in policy JSON and documented in key-management/release checklist references.
