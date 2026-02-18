# Issue #4974 Plan

- Issue: #4974
- Status: Implemented

## Approach (Implemented)
1. Implemented archive migration tool `scripts/ci/archive_completed_specs.py` with deterministic marker output and JSON report.
2. Added tool support for dry-run/apply modes, pointer generation, and archive index row maintenance.
3. Extended archive policy contract tests to validate tool-generated archive/pointer/index output.
4. Re-ran checker/contract suite to ensure active-tree placement policy remains green.

## Affected Modules
- `scripts/ci/archive_completed_specs.py`
- `scripts/ci/test_check_spec_archive_policy.sh`
- `scripts/ci/check_spec_archive_policy.sh` (validated parity contract)
- `specs/4974/spec.md`
- `specs/4974/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigation:
  - Fail closed on missing issue ids, missing required files, non-implemented spec status, and archive target collisions.
  - Keep deterministic reason taxonomy output in both tool and checker paths.
  - Validate tool output through existing archive policy checker in test fixtures.

## Interface Contract
- No protocol/wire-format changes without explicit approval.
- Tool reason taxonomy:
  - `kamn.ci.spec-archive-tool-reason-taxonomy.v1`
- Tool JSON schema:
  - `kamn.ci.spec-archive-tool-report.v1`

## ADR
- No ADR required (no dependency/protocol/architecture boundary change).
