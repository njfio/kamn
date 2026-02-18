# Issue #4969 Plan

- Issue: #4969
- Status: Implemented

## Approach
- Extend migration-group fixture coverage to canary/ci/deploy/governance wrappers already migrated to manifest dispatch.
- Extend lane-ownership fixture so inventory generation produces deterministic ownership evidence.
- Regenerate baseline inventory from source fixtures and synchronize deletion manifest entries to the expanded set.
- Add explicit regression assertion for the targeted non-kolme wave entries in the manifest checker test.
- Verify with superseded-manifest, stale-reference, command-surface, and fast CI-tools suites.

## Affected Modules
- `fixtures/ci/kolme_manifest_migration_contract_groups.json`
- `fixtures/ci/superseded_script_lane_ownership.json`
- `fixtures/ci/superseded_script_inventory_baseline.json`
- `fixtures/ci/superseded_script_deletion_manifest.json`
- `scripts/ci/test_check_superseded_script_deletion_manifest.sh`

## Risks and Mitigations
- Risk: expanded manifest could introduce unknown-entry drift against inventory.
  - Mitigation: inventory regenerated from migration metadata, plus red/green checker evidence captured.
- Risk: stale-reference detector could fail if any newly-enforced deleted path is referenced.
  - Mitigation: scoped stale-reference contract suite executed post-change.

## Interface Contract
- No protocol/wire-format changes.
- Preserve checker schema/reason-taxonomy versions and fail-closed reason-code behavior.

## ADR
- Not required (no architectural dependency/protocol decision).
