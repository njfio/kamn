# Plan: Issue #5975

## Approach
- Reuse existing CI telemetry scripts to add deterministic governance/runtime test ratio outputs.
- Introduce threshold policy checker and CI wiring.
- Consolidate duplicate high-volume doc-contract suites in one bounded wave.

## Affected Modules (Expected)
- `scripts/ci/*ratio*` and related workflow wiring
- `crates/kamn-core/tests/*` doc-contract suites
- `.github/workflows/ci-fast-gate.yml` / `.github/workflows/ci-deep-validate.yml`

## Risks / Mitigations
- Risk: accidental loss of required docs evidence checks.
  Mitigation: keep required marker inventory explicit and test-backed.

## Interfaces / Contracts
- Governance/runtime ratio telemetry schema.
- Policy gate reason code taxonomy.
