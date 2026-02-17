# Kolme Devnet Ops (Compatibility Redirect)

Canonical operations guidance for Kolme local-heavy and combined runtime
validation lives in:

- `docs/planning/kolme-devnet-ops.md`

This compatibility path is kept so issue and PR references using
`docs/deploy/kolme_devnet_ops.md` remain valid.

## Release Evidence and Remediation Matrix

Release go/no-go evidence must include deterministic combined native libp2p +
Kolme markers from `kamn.runtime.go-no-go-gate-report.v1`:

- `combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1`
- `combined_transport_reason_codes=["fork_choice_stale_block_height"]`
- `combined_kolme_runtime_reason_code in {"not_run","live_runtime_integration_passed"}`
- `kolme_runtime_commit_failure_taxonomy_version=v1`
- `kolme_fixture_profile=real-node-non-synthetic-v1`
- `kolme_fixture_profile_version=v1`
- `combined_lane_marker_contract_status=verified`

Fail-closed remediation reason codes are emitted in milestone bundles by
`scripts/deploy/gonogo_evidence_contract.py` and must block promotion when
present, including:

- `milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch`
- `milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch`
- `milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch`
- `milestone_review_go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_mismatch`
- `milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch`
- `milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch`
- `milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch`
- `milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch`

## Drift Taxonomy and Runbook Marker Parity Contracts (Issue #4282)

Failover preflight drift governance remains deterministic only when checker taxonomy markers and
runbook marker declarations stay synchronized.

Required checker/runbook parity markers:

- `drift_taxonomy_mapping_status=verified`
- `runbook_marker_parity_status=verified`
- `drift_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1`
- `drift_taxonomy_runbook_reason_codes_csv=drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch`

Fail-closed drift reasons:

- `drift_taxonomy_mapping_drift_detected`
- `runbook_marker_parity_mismatch`

Validation command:

- `bash scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh check-policy --report-file /tmp/failover-sync-preflight-report.json --runbook-file docs/deploy/kolme_devnet_ops.md --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/failover-sync-preflight-policy.json`

- `Regression: #4287`
- `Regression: #4288`

## TLS Dependency-Posture Compatibility Markers

Live-HTTPS dependency posture must remain explicit in compatibility runbooks:

- Checker command:
  - `bash scripts/ci/check_kamn_core_live_https_dependency_posture.sh --output-json /tmp/kamn-core-live-https-dependency-posture-report.json`
- Deterministic policy markers:
  - `reason_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1`
- Fail-closed drift reason markers:
  - `rustls_pemfile_dependency_optional_flag_mismatch`
  - `webpki_roots_dependency_missing`
  - `webpki_roots_feature_mapping_missing`

- `Regression: #4108`
- `Regression: #4107`
