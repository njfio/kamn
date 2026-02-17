# Spec: #4168 Deterministic Signer Configuration Validation Errors and Fallback Removal Mapping

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Operator remediation and policy automation require deterministic configuration error semantics for signer material validation. Missing or invalid signer material must map to stable policy reason outputs, and fallback signer behavior must remain fail-closed.

## Scope

In scope:
- Add deterministic signer configuration reason taxonomy output markers to deployment preflight policy checker output.
- Map missing signer material and invalid signer secret hex to explicit deterministic policy reasons.
- Preserve fallback signer prohibition behavior and deterministic reason mapping.
- Update operations docs and docs-contract tests for new signer-config taxonomy markers.

Out of scope:
- Runtime mode redesign.
- External custody/key-provider integration.

## Acceptance Criteria

AC-1 Missing signer material yields deterministic configuration error markers.
AC-2 Fallback signer paths remain removed from active pass paths and fail closed with deterministic mapping.
AC-3 Integration/docs contracts validate stable expected output markers.

## Conformance Cases

- C-01 (AC-1): checker output includes signer-config taxonomy markers:
  - `signer_config_reason_taxonomy_version`
  - `signer_config_reason_codes_csv`
  - `signer_config_reason_codes_value`.
- C-02 (AC-1): run-mode missing signer material produces `signer_secret_missing` in signer-config reason value.
- C-03 (AC-1): run-mode invalid signer secret hex produces `signer_secret_invalid_hex` in signer-config reason value.
- C-04 (AC-2/AC-3): fallback signer secret violation remains in signer-config mapping and docs/contracts stay synchronized.

## Success Metrics

- Deployment preflight checker tests pass with new signer-config mapping assertions.
- Ops configuration docs include deterministic signer-config contract section.
- Rust docs-contract tests enforce new section markers.
