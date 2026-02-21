# Issue #5445 Spec - Migrate SBOM/Provenance Generator Lane to Rust Harness

- Status: Accepted
- Issue: #5445
- Parent: #4029
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
Issue `#4036` introduced a Python SBOM/provenance generator contract lane, but shell/python surface ratchets require migration of this lane to a Rust-native harness while preserving deterministic marker/schema contracts and fail-closed behavior.

## Scope
In scope:
- Add Rust-native SBOM/provenance generator harness binary.
- Preserve exact fixture semantics, reason taxonomy strings, output JSON schema markers, and boundary guards.
- Update tests and docs to reference Rust execution surface.
- Keep compatibility wrapper on existing script path that delegates to Rust harness.

Out of scope:
- Any expansion of go/no-go checker policy (`#4037`).
- New artifact schema fields or reason-code taxonomies.

## Acceptance Criteria
- AC-1: Rust harness emits deterministic SBOM/provenance schema/taxonomy markers and JSON payload parity with existing contract behavior.
- AC-2: Fail-closed profile drift, run-mode opt-in, and invalid profile boundaries remain unchanged.
- AC-3: Docs/test contracts reference Rust harness execution surface and remain parity-verified.
- AC-4: Unit, Functional, Integration, Regression, and Performance tests pass for migrated lane.

## Conformance Cases
- C-01 (Unit, AC-1): baseline profile emits deterministic marker/schema payload and GO decision.
- C-02 (Functional, AC-2): injected-drift profile fails closed with deterministic reason code.
- C-03 (Integration, AC-2): run mode enforces explicit local opt-in + ci-fast-gate FAIL boundary.
- C-04 (Regression, AC-2): invalid profile input fails closed with deterministic error.
- C-05 (Performance, AC-4): generator remains within bounded CI runtime budget.
- C-06 (Conformance docs, AC-3): strategy/ops docs include Rust command markers and tests pass.

## Success Metrics / Observable Signals
- Rust harness command succeeds for baseline profile and preserves marker parity.
- Existing contract tests pass after command-surface migration.
- Python surface for SBOM generator lane is reduced to thin compatibility shim.
