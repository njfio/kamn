# Issue #4373 Spec

Status: Implemented
Priority: P1
Parent: #4370
Children: #4379, #4380

## Problem Statement

The signed-to-Kolme promotion contract must fail closed when simulated signing is accepted in production paths and must emit deterministic native secp256k1 signer evidence for release review.

## Scope

In scope:
- Signed-to-Kolme policy checker enforcement for native secp256k1 markers.
- Deterministic native-signer reason taxonomy outputs in policy reports.
- Contract-lane summary evidence fields required for deterministic signer-policy validation.
- Tests and docs updates that assert these contracts.

Out of scope:
- New signer algorithms and HSM integrations.
- Runtime cryptographic implementation changes in Rust signer backends.

## Acceptance Criteria

AC-1 (Fail-closed simulated signing):
- Given production/live signer validation input
- When simulated signing markers appear or native markers are missing
- Then policy output is `NO-GO` with deterministic reason codes.

AC-2 (Deterministic native signer evidence):
- Given valid run/dry-run summaries
- When the checker runs
- Then policy output includes stable native-signer taxonomy version/codes/value fields.

AC-3 (Stable evidence lineage):
- Given signed-to-Kolme run summaries
- When runtime signing evidence drifts from native secp256k1 contract values
- Then checker fails closed with deterministic mismatch reasons.

AC-4 (Regression safety):
- Given updated signer-policy contracts
- When script and docs contract suites run
- Then existing signed-to-Kolme/runtime evidence regressions remain green and new cases are covered.

## Conformance Cases

C-01 (AC-1, Functional/Conformance): simulated signing profile in runtime command yields `NO-GO` and `runtime_commit_simulated_signing_profile_detected`.

C-02 (AC-1, Functional/Conformance): missing native secp256k1 signing marker yields `NO-GO` and `runtime_commit_native_signing_profile_marker_missing`.

C-03 (AC-2, Unit/Functional): policy report includes:
- `native_signer_reason_taxonomy_version`
- `native_signer_reason_codes_csv`
- `native_signer_reason_codes_value`
with deterministic values.

C-04 (AC-3, Functional/Conformance): runtime signing profile drift in summary yields `NO-GO` and `runtime_signing_profile_mismatch`.

C-05 (AC-4, Integration/Regression): signed-to-Kolme contract lane run still passes with native signer markers present.

C-06 (AC-4, Regression): docs contract tests include new taxonomy markers and pass.

## Success Signals

- Signed-to-Kolme policy checker rejects simulated signer paths deterministically.
- Native signer taxonomy fields are emitted in stdout and JSON report consistently.
- Updated script and docs contract tests pass without flake.
