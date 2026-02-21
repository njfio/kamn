# Issue #5505 Spec - R50 Spec-Volume Guardrail Remediation Contracts

- Status: Implemented
- Issue: #5505
- Parent: #5469
- Milestone: R50.18 Spec-volume guardrail remediation contracts

## Problem Statement
R50 records a guardrail breach: 750 spec directories at 92 modules (8.2:1), above the maximum 7.7:1 ratio.

## Scope
In scope:
- Add deterministic remediation-plan markers to the R50 review artifact.
- Add docs-contract tests validating remediation arithmetic and tranche-plan consistency.
- Update R50 priority/status text to reflect active remediation contract.

Out of scope:
- Deleting existing spec directories.
- Runtime/API code changes.

## Acceptance Criteria
- AC-1: R50 review doc includes a deterministic remediation-plan marker set for spec-volume breach.
- AC-2: Marker arithmetic is internally consistent (`required_reduction = baseline - target_max`; tranche plan satisfies reduction requirement).
- AC-3: Priority summary reflects active remediation contract status.
- AC-4: New docs-contract tests validate marker presence and arithmetic.
- AC-5: Targeted tests pass.

## Conformance Cases
- C-01 (AC-1): remediation schema/version and tranche markers exist.
- C-02 (AC-2): arithmetic checks pass for baseline/target/reduction and tranche floor math.
- C-03 (AC-3): priority summary row contains remediation-contract status language.
- C-04 (AC-4): docs-contract suite asserts marker presence + derived-value consistency.
- C-05 (AC-5): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` passes.

## Success Metrics / Observable Signals
- R50 artifact contains executable, deterministic remediation plan markers.
- CI-enforced docs-contract test detects any drift in remediation arithmetic.
