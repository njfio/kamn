# Plan: Issue #5978

## Approach
- Extend current shell/rust telemetry approach with governance/runtime test-ratio metric.
- Add policy checker script and CI wiring.
- Consolidate one high-volume doc-contract file cluster with preserved required markers.

## Affected Modules
- `scripts/ci/*`
- `.github/workflows/ci-fast-gate.yml`
- `crates/kamn-core/tests/*` (selected consolidation targets)

## Risks / Mitigations
- False positives from ratio metric definition.
  Mitigation: deterministic counting contract + fixture-based policy tests.

## Interfaces / Contracts
- Ratio telemetry report fields.
- Policy checker reason codes.
