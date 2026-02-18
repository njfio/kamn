# ADR: Lane Registry Source of Truth for Manifest and Wrapper Artifacts

## Status

Accepted (2026-02-18)

## Context

The repository contains a large number of manifest-backed contract lanes and symlink wrappers.
Manual maintenance of those artifacts causes deterministic drift risk:

- manifest payloads can diverge from expected lane metadata,
- wrapper symlink wiring can fall out of sync,
- CI behavior can change without an auditable, centralized change record.

To keep shell-surface governance enforceable, generation and drift detection must be deterministic
and fail closed.

## Decision

Adopt `scripts/framework/lane_registry.json` as the canonical source of truth for:

1. `scripts/framework/manifests/*.json` payload content.
2. Wrapper symlink targets for manifest-backed lane entrypoints.

Enforce this decision with:

- `scripts/framework/generate_lane_artifacts.py --mode check|render`
- `scripts/framework/check_lane_registry_drift.sh`
- contract tests:
  - `scripts/framework/test_lane_registry_generation.sh`
  - `scripts/framework/test_check_lane_registry_drift.sh`
  - `scripts/framework/test_contract_framework.sh`

Drift checks are fail-closed and emit deterministic reason taxonomy markers:

- `kamn.framework.lane-registry-drift-reason-taxonomy.v1`
- `lane_registry_manifest_drift_detected`
- `lane_registry_wrapper_drift_detected`
- `lane_registry_schema_mismatch`
- `lane_registry_artifact_missing`

## Alternatives Considered

### Manual manifest and wrapper maintenance

Rejected. This does not scale for shell-surface growth and repeatedly introduces stale-artifact drift.

### Per-lane ad hoc generators

Rejected. This fragments policy and makes drift contracts inconsistent across lane families.

## Consequences

Positive:

- single auditable source for generated lane artifacts,
- deterministic CI drift detection with stable reason taxonomy markers,
- lower maintenance overhead for manifest-backed wrappers.

Tradeoffs:

- registry updates are required whenever generated lane metadata changes,
- contributor workflow must use generator/checker contracts instead of direct manual edits.

## Validation and Traceability

- Source registry: `scripts/framework/lane_registry.json`
- Generator: `scripts/framework/generate_lane_artifacts.py`
- Drift checker: `scripts/framework/check_lane_registry_drift.sh`
- Docs contract: `docs/architecture/lane-registry-generation.md`

Regression markers:

- `Regression: #4829`
- `Regression: #4830`
- `Regression: #4883`
