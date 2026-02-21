# Issue #4034 Spec - License Policy Parity Checker Across Root Policy and Crate Manifests

- Status: Reviewed
- Issue: #4034
- Parent: #4028
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
Workspace license governance currently validates crate manifest `package.license` values against an expected SPDX string, but does not fail closed on drift between root `LICENSE` policy content and manifest policy enforcement.

## Scope
In scope:
- Enforce root `LICENSE` policy parity in the workspace license checker.
- Keep deterministic reason taxonomy and fail-closed mismatch behavior.
- Wire checker contract lane into CI tools command surface.
- Add docs parity assertions for updated checker command/marker contracts.

Out of scope:
- Changing project license model or legal policy text.
- SBOM/provenance release-go/no-go checks (handled in #4036/#4037).

## Acceptance Criteria
- AC-1: Checker fails closed on root policy file missing/mismatch and manifest license mismatch conditions with deterministic reason codes.
- AC-2: CI tooling includes license checker contract lane in fast/full command surfaces with docs marker parity.
- AC-3: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): checker emits deterministic taxonomy/reason CSV markers including root-policy reason codes.
- C-02 (Functional, AC-1): root policy drift fixture fails closed with deterministic root-policy mismatch reason code.
- C-03 (Integration, AC-2): CI tools command surface includes license checker contract lane in fast/full paths.
- C-04 (Regression, AC-1/AC-2): docs strategy marker drift for license-governance reason code CSV fails docs-contract tests.
- C-05 (Performance, AC-3): checker runtime remains bounded within low-cost CI budget.

## Success Metrics
- Root license policy drift and manifest drift both fail closed with deterministic taxonomy markers.
- CI command-surface coverage includes license checker lane in fast/full paths.
- Docs and code marker taxonomy remain synchronized by contract tests.
