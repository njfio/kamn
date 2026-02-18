# Issue #4962 Plan

- Issue: #4962
- Status: Implemented

## Approach
- Implement deterministic archive migration utility with explicit dry-run/apply semantics.
- Extend archive-policy contract tests to validate tool-generated archive/pointer/index fixtures.
- Keep checker outputs deterministic with fail-closed reasoning.

## Affected Modules
- `scripts/ci/archive_completed_specs.py`
- `scripts/ci/test_check_spec_archive_policy.sh`

## Risks and Mitigations
- Risk: tool-generated output shape drifts from checker expectations.
- Mitigation: tool-generated fixtures are explicitly validated in contract tests.

## Interface Contract
- Preserve deterministic archive output markers consumed by archive-policy checks.

## ADR
- Not required (tooling implementation within existing governance architecture).
