# Kolme Upstream Compatibility Snapshot Contract (Issue #777)

This research note defines a deterministic, low-cost compatibility snapshot contract
for Kolme upstream assumptions used by KAMN release gating.

## Snapshot Schema

Compatibility snapshots use:

- `schema_version`: `kamn.kolme.compatibility-snapshot.v1`
- `kolme_repository.owner`, `kolme_repository.repo`
- `upstream.release_tag`, `upstream.commit_sha`
- `docs_contracts[]`:
  - `path`
  - `sha256`
- `protocols[]`:
  - `name`
  - `version`

Deterministic comparison rules:

- Doc and protocol arrays are canonicalized by key (`path`, `name`) before compare.
- Comparison reports changed fields using stable dotted paths.
- Any changed field forces fail-closed drift status.

## Drift Checker Commands

- Checker:
  - `python3 scripts/kolme/check_snapshot_drift.py --baseline-file fixtures/kolme_compatibility/snapshot_baseline.json --candidate-file fixtures/kolme_compatibility/snapshot_candidate_match.json --output-json /tmp/kolme-drift-report.json`
- PR contract lane:
  - `bash scripts/kolme/run_snapshot_drift_contract_lane.sh`

## CI Scope and Cost Controls

- Fast gate runs only fixture-based deterministic checks for this contract lane.
- No remote Kolme fetch is required in PR path.
- Runtime budget guard is enforced in `run_snapshot_drift_contract_lane.sh` (45s).

## Drift Remediation

When drift is detected:

1. Confirm whether upstream change is expected.
2. Update baseline snapshot fixtures and this document with rationale.
3. Re-run contract lane before merge.

## Regression Guard

- Sync manager version drift and docs contract hash drift remain fail-closed (`Regression: #775`).

## Local Validation

Run from repository root:

```bash
bash scripts/kolme/test_check_snapshot_drift.sh
bash scripts/kolme/test_run_snapshot_drift_contract_lane.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
