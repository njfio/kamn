# Issue #4959 Plan

- Issue: #4959
- Status: Implemented

## Approach
- Deliver first-wave scope through two merged subtasks:
  - `#4970` runtime/kolme wave activation and wrapper compaction.
  - `#4969` canary/ci/deploy/governance wave activation and parity proofs.
- Keep deletion-manifest and superseded-inventory outputs deterministic with fail-closed reason taxonomy.
- Verify post-merge parity with stale-reference and CI command-surface contract suites.

## Affected Modules
- `fixtures/ci/superseded_script_deletion_manifest.json`
- `fixtures/ci/superseded_script_inventory_baseline.json`
- `fixtures/ci/kolme_manifest_migration_contract_groups.json`
- `fixtures/ci/superseded_script_lane_ownership.json`
- `scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- `scripts/ci/test_check_stale_script_references.sh`
- `scripts/kolme/run_contract_lane_dispatch.sh`
- `scripts/kolme/contract_lane_dispatch_impl.py`

## Risks and Mitigations
- Risk: manifest drift from inventory baseline.
  - Mitigation: deterministic generator parity test + fail-closed checker.
- Risk: stale references to actually-deleted paths.
  - Mitigation: stale-reference detector enforced in CI and validated in regression suite.

## Interface Contract
- No protocol/wire-format changes.
- Preserve schema/reason-taxonomy compatibility for inventory/deletion/stale-reference outputs.

## ADR
- Not required (no architecture/dependency/protocol change).
