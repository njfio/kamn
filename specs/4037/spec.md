# Issue #4037 Spec - SBOM/Provenance Release Go-No-Go Checker and Docs Parity Contracts

- Status: Reviewed
- Issue: #4037
- Parent: #4029
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
Issue `#4036` delivers deterministic SBOM/provenance artifact generation, but release policy enforcement still lacks a dedicated fail-closed checker that validates required artifact markers and verifies strategy/ops docs parity for release guidance.

## Scope
In scope:
- Add deterministic SBOM/provenance release go/no-go checker script.
- Enforce fail-closed validation for required artifact markers and decision-contract fields.
- Enforce strategy/ops docs parity for required checker markers and commands.
- Add Rust contract tests covering unit/functional/integration/regression/performance behavior.
- Update CI strategy + ops docs with checker marker/command contracts.

Out of scope:
- External attestations/signing/notarization.
- Changes to the #4036 generator fixture schema.

## Acceptance Criteria
- AC-1: Checker fails closed when required artifact markers are missing/invalid.
- AC-2: Checker fails closed when strategy/ops docs parity markers drift.
- AC-3: Checker emits deterministic reason taxonomy markers and release decision outputs.
- AC-4: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): checker accepts baseline generator artifact and validates required marker contract fields.
- C-02 (Functional, AC-1): tampered/missing artifact marker(s) fail closed with deterministic reason code.
- C-03 (Integration, AC-2): generator artifact + checker + strategy/ops docs parity path passes with deterministic marker outputs.
- C-04 (Regression, AC-2): docs drift/missing marker fails closed with deterministic docs-parity reason code.
- C-05 (Performance, AC-4): checker execution stays within CI-smoke bounded runtime budget.

## Success Metrics / Observable Signals
- Release checker consistently emits stable taxonomy/version markers.
- Missing/invalid artifact markers always produce NO-GO outcomes with deterministic fail reasons.
- Docs parity drift is detected before release-gate promotion.
- Contract test suite remains green under `cargo test -p kamn-core --test sbom_provenance_release_gonogo_checker_contract -- --nocapture`.
