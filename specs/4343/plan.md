# Plan — #4343

Status: Reviewed

## Approach

- Extend `scripts/ci/test_check_kamn_core_missing_docs_policy.sh` with explicit assertions for deterministic allowlist delta evidence markers.
- Implement additive marker emission in `scripts/ci/check_kamn_core_missing_docs_policy.sh` by:
  - capturing throughput report JSON fields;
  - capturing velocity guard outputs/policy JSON;
  - emitting stable `missing_docs_*` markers on success and velocity-failure paths.
- Update `docs/ci/strategy.md` with the new marker contract.

## Affected Areas

- `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- `scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: brittle parsing of velocity output.
  - Mitigation: use JSON policy file fields as source-of-truth where possible.
- Risk: CI noise from additional output lines.
  - Mitigation: additive deterministic markers with fixed keys.

## Contract Notes

- Preserve existing `kamn-core missing-docs policy contract passed.` success banner.
- Preserve existing rustdoc navigation parity taxonomy markers for README drift.
