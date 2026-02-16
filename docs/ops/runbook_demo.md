# Local Demo Startup Drift Runbook

This runbook documents fail-closed handling for local multi-process demo startup
dependency drift and orchestration instability in the Kolme process lifecycle lane.

## Scope

- Lane under governance:
  - `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh`
- Policy checker:
  - `python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py`
- Contract-lane regression test:
  - `bash scripts/kolme/test_run_local_kolme_fork_process_lifecycle_contract_lane.sh`

## Startup Dependency Drift

The checker must fail closed when downstream startup phases report success while
upstream prerequisites are not satisfied.

Required fail-closed markers include:

- `startup_dependency_drift:readiness_without_process_start`
- `startup_dependency_drift:integration_without_readiness`
- `startup_dependency_drift:integration_without_process_start`

These markers cover:

- readiness success accepted after process start failure,
- integration success accepted before readiness success, and
- integration success accepted while process start is not successful.

## Orchestration Instability

The checker must fail closed when orchestration evidence becomes unstable:

- duplicate check IDs in the same report:
  - `check_id_duplicate:<check-id>`
- non-deterministic phase ordering across the primary orchestration chain:
  - `check_sequence_mismatch`

## Reason Taxonomy and Evidence Normalization

Lifecycle summaries now include deterministic reason taxonomy and normalized
evidence sections:

- `reason_taxonomy.schema_version=kamn.kolme.local-fork-process-lifecycle.reason-taxonomy.v1`
  - `overall`
  - `startup`
  - `readiness`
  - `integration`
  - `teardown`
- `normalized_evidence.schema_version=kamn.kolme.local-fork-process-lifecycle.evidence-normalization.v1`
  - `primary_check_order`
  - `checks_by_id`

Fail-closed mismatch markers include:

- `reason_taxonomy_overall_mismatch`
- `reason_taxonomy_startup_mismatch`
- `reason_taxonomy_readiness_mismatch`
- `reason_taxonomy_integration_mismatch`
- `reason_taxonomy_teardown_mismatch`
- `normalized_evidence_primary_check_order_mismatch`
- `normalized_evidence_status_mismatch:<check-id>`
- `normalized_evidence_reason_code_mismatch:<check-id>`
- `normalized_evidence_command_mismatch:<check-id>`

## Local Validation Commands

```bash
# Run full contract-lane regression (dry-run safe path by default)
bash scripts/kolme/test_run_local_kolme_fork_process_lifecycle_contract_lane.sh

# Validate a generated lifecycle summary directly
python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py \
  --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json
```

## CI Cost Boundary

- Keep startup drift and orchestration instability checks in dry-run contract lanes.
- Reserve local heavy run-mode execution for explicit opt-in:
  - `KAMN_KOLME_LOCAL_HEAVY=1`
