# Issue #4036 Spec - Deterministic SBOM/Provenance Artifact Generator and Schema Validation

- Status: Reviewed
- Issue: #4036
- Parent: #4029
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
`#4029` requires deterministic SBOM/provenance evidence generation, but there is no dedicated contract lane that emits a stable SBOM/provenance artifact schema and fail-closed reason markers for generator/profile drift.

## Scope
In scope:
- Add a deterministic SBOM/provenance generator contract script.
- Add fixture-driven schema/profile validation and deterministic reason taxonomy markers.
- Add Rust contract tests covering unit/functional/integration/regression/performance behavior.
- Add ops/strategy docs markers for generator schema and validation commands.

Out of scope:
- External signing/notarization/attestation services.
- Release go/no-go policy checker integration (handled in `#4037`).

## Acceptance Criteria
- AC-1: Generator output is deterministic and schema-valid for baseline profile.
- AC-2: Artifact markers include stable SBOM/provenance fields required for downstream go/no-go checks.
- AC-3: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): baseline profile emits deterministic schema/taxonomy/marker fields and JSON payload markers.
- C-02 (Functional, AC-1): injected-drift profile fails closed with deterministic reason code.
- C-03 (Integration, AC-2): run mode requires explicit local opt-in boundary and deterministic run-mode markers.
- C-04 (Regression, AC-1): invalid profile input fails closed with deterministic validation message.
- C-05 (Performance, AC-3): baseline generation remains within bounded CI-smoke budget.
- C-06 (Conformance docs, AC-2): strategy/ops docs include deterministic SBOM/provenance marker and command contracts.

## Success Metrics / Observable Signals
- Generator prints deterministic schema + reason markers on every run.
- Baseline profile remains `status=pass` with `final_decision=GO` and `reason_code=none`.
- Injected drift profile remains fail-closed with deterministic reason marker.
- Docs contract tests fail closed if SBOM/provenance marker blocks drift.
