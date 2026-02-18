# Issue #4961 Plan

- Issue: #4961
- Status: Implemented

## Approach
- Land archive-policy governance doc with deterministic markers.
- Extend archive-policy checker tests to require marker presence and fail closed when absent.
- Wire marker into milestone index contract path.

## Affected Modules
- `docs/planning/spec-archive-policy.md`
- `scripts/ci/test_check_spec_archive_policy.sh`
- `specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md`

## Risks and Mitigations
- Risk: documentation-only policy drifts from enforcement checks.
- Mitigation: marker contract tests enforce parity.

## Interface Contract
- Preserve deterministic policy marker keys consumed by archive-policy checks.

## ADR
- Not required (governance-policy doc/check scope).
