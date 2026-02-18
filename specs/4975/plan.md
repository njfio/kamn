# Issue #4975 Plan

- Issue: #4975
- Status: Implemented

## Approach (Implemented)
1. Publish first-wave archived-spec mapping report at `specs/archive/index.md`.
2. Extend archive policy checker to enforce report presence, entry parity, and count parity.
3. Extend checker contract tests with explicit missing-index fail-closed case.
4. Verify policy checker output and lifecycle synchronization.

## Affected Modules
- `specs/archive/index.md`
- `scripts/ci/check_spec_archive_policy.sh`
- `scripts/ci/test_check_spec_archive_policy.sh`
- `specs/4975/spec.md`
- `specs/4975/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigation:
  - Fail closed when archive index report is missing or inconsistent.
  - Keep checker output deterministic via reason taxonomy and explicit metrics markers.
  - Keep first-wave index mapping sorted and explicit by issue id/path.

## Interface Contract
- No protocol/wire-format changes without explicit approval.
- Updated checker reason taxonomy CSV includes archive-index parity failure classes:
  - `spec_archive_index_missing`
  - `spec_archive_index_entry_missing`
  - `spec_archive_index_count_mismatch`
- New marker: `index_entry_count`.

## ADR
- No ADR required (no dependency/protocol/architecture boundary change).
