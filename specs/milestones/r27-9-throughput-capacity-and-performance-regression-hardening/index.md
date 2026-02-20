# Milestone Spec Index — R27.9 Throughput Capacity and Performance Regression Hardening

Milestone: `R27.9 Throughput capacity and performance regression hardening` (GitHub milestone #43)

## Intent

Define deterministic performance baseline, capacity-governance, and regression guardrail contracts for runtime hot paths while keeping CI-fast cost bounded and local-heavy load validation explicit.

## Scope

In scope:
- Runtime/signing/transport hot-path benchmark fixture contracts.
- Performance smoke report generation and threshold enforcement.
- Capacity/load governance policy checks with deterministic reason taxonomy.
- Docs/runbook parity for performance and remediation markers.

Out of scope:
- Host-specific benchmark auto-calibration.
- New long-running benchmark executables in fast CI.

## Issue Hierarchy

- Epic: `#3993`
- Stories: `#3994`, `#3995`
- Tasks/Subtasks: `#3996`..`#4007`

## Validation Expectations

- Unit/functional/integration/regression coverage for performance fixture parsing, threshold gating, and docs-marker drift.
- CI-fast smoke commands remain bounded by policy.
- Local-heavy load lanes remain explicit opt-in.
