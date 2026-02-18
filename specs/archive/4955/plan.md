# Issue #4955 Plan

- Issue: #4955
- Status: Implemented

## Approach
- Deliver story through three tasks:
  - `#4958` inventory + deletion-manifest contracts.
  - `#4959` first deletion-wave execution.
  - `#4960` stale-reference fail-closed detector and CI guard.
- Synchronize story lifecycle docs after task closures.

## Affected Modules
- `fixtures/ci/superseded_script_inventory_baseline.json`
- `fixtures/ci/superseded_script_deletion_manifest.json`
- `scripts/ci/superseded_script_inventory.py`
- `scripts/ci/stale_script_reference_detector.py`
- `scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- `scripts/ci/test_check_stale_script_references.sh`

## Risks and Mitigations
- Risk: deletion manifest and references drift apart.
- Mitigation: fail-closed stale-reference and manifest parity checks in CI.

## Interface Contract
- Preserve deletion-manifest schema and deterministic reason taxonomy outputs.

## ADR
- Not required.
