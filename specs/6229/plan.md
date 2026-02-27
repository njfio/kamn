# Issue 6229 Plan

## Approach
1. Measure current critical-path coverage using the existing gate script to capture actual line/function percentages per target.
2. Ratchet thresholds upward conservatively but materially (set minima below measured baseline with safety headroom).
3. Keep schema and policy script unchanged to preserve deterministic reason taxonomy and fail-closed behavior.
4. Add/update rationale documentation for each threshold adjustment.
5. Re-run the coverage gate to validate deterministic pass/fail behavior under the new baseline.

## Affected Modules
- `.ci/critical-path-coverage-thresholds.json`
- `docs/architecture/adr-critical-path-assurance-gates.md` (rationale update)
- `specs/6229/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: thresholds raised too aggressively causing flaky CI.
  - Mitigation: set minima with measured headroom from current deterministic baseline.
- Risk: thresholds remain too weak to improve assurance.
  - Mitigation: require clear upward deltas across both core and node targets.

## Interfaces
- No API changes.
- CI interface unchanged: `scripts/ci/run_critical_path_coverage_gate.sh` + policy report schema.
