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

## Crash-Recovery Replay Idempotency Taxonomy and Runbook Marker Parity Contracts (Issue #4237)

Sqlite crash-recovery replay idempotency taxonomy markers and runbook marker declarations must stay
synchronized so policy checks fail closed under drift.

Required checker/runbook parity markers:

- `replay_idempotency_taxonomy_mapping_status=verified`
- `runbook_marker_parity_status=verified`
- `replay_idempotency_runbook_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-replay-idempotency-runbook-reason-taxonomy.v1`
- `replay_idempotency_runbook_reason_codes_csv=replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch`
- `replay_idempotency_runbook_reason_code=none|<reason>`

Fail-closed drift reasons:

- `replay_idempotency_taxonomy_mapping_drift_detected`
- `runbook_marker_parity_mismatch`

Validation command:

- `bash scripts/runtime/check_sqlite_crash_recovery_live_policy.sh --report-file /tmp/sqlite-crash-recovery-live-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/sqlite-crash-recovery-live-policy-report.json`

- `Regression: #4242`
- `Regression: #4243`

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

## Service API Axum Protocol Taxonomy and Runbook Marker Parity Contracts (Issue #4267)

Service API axum ingress protocol taxonomy markers and runbook marker declarations must remain
synchronized to keep fail-closed remediation deterministic.

Required checker/runbook parity markers:

- `protocol_taxonomy_mapping_status=verified`
- `runbook_marker_parity_status=verified`
- `protocol_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-taxonomy-runbook-reason-taxonomy.v1`
- `protocol_taxonomy_runbook_reason_codes_csv=protocol_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch`
- `protocol_compliance_reason_taxonomy_version=kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1`
- `protocol_compliance_reason_codes_csv=method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected`
- `request_validation_reason_taxonomy_version=kamn.runtime.service-api-request-validation-reason-taxonomy.v1`
- `request_validation_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid`
- `error_envelope_reason_taxonomy_version=kamn.runtime.service-api-error-envelope-reason-taxonomy.v1`
- `error_envelope_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found`
- `service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1`
- `service_api_axum_protocol_mismatch_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation`

Fail-closed drift reasons:

- `protocol_taxonomy_mapping_drift_detected`
- `runbook_marker_parity_mismatch`

Validation command:

- `bash scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh --output-json /tmp/service-api-axum-ingress-contract-lane-report.json --policy-output-json /tmp/service-api-axum-ingress-policy-report.json`

- `Regression: #4272`
- `Regression: #4273`

## Fork-Choice Finality Taxonomy and Runbook Marker Parity Contracts (Issue #4252)

Fork-choice finality reconciliation taxonomy markers and runbook marker declarations must remain
synchronized to keep partition-recovery promotion decisions deterministic.

Required checker/runbook parity markers:

- `finality_taxonomy_mapping_status=verified`
- `runbook_marker_parity_status=verified`
- `convergence_reason_taxonomy_version=kamn.runtime.libp2p-convergence-reason-taxonomy.v1`
- `convergence_reason_codes_csv=fork_choice_stale_block_height`
- `finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1`
- `finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch`
- `promotion_decision_reason_mapping_status=verified`
- `promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1`
- `promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation`
- `promotion_decision_reason_code=none|<reason>`
- `libp2p_finality_evidence_convergence_status=verified`
- `libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1`
- `libp2p_finality_evidence_reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch`

Fail-closed drift reasons:

- `finality_taxonomy_mapping_drift_detected`
- `runbook_marker_parity_mismatch`
- `libp2p_finality_evidence_link_missing:source_report_file`
- `libp2p_finality_evidence_payload_tamper_detected:<field>`
- `libp2p_finality_promotion_decision_reason_mapping_mismatch`

Validation commands:

- `bash scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh --report-file /tmp/libp2p-convergence-process-isolated-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/libp2p-convergence-process-isolated-live-policy.json`
- `bash scripts/runtime/check_libp2p_convergence_process_isolated_live_evidence_convergence.sh --report-file /tmp/libp2p-convergence-process-isolated-live-contract-lane-report.json --policy-file /tmp/libp2p-convergence-process-isolated-live-policy.json --output-json /tmp/libp2p-convergence-process-isolated-live-convergence-report.json`
- `bash scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh --output-json /tmp/libp2p-convergence-process-isolated-live-contract-lane-report.json --policy-output-json /tmp/libp2p-convergence-process-isolated-live-policy.json --summary-output-json /tmp/libp2p-convergence-process-isolated-live-summary.json --convergence-output-json /tmp/libp2p-convergence-process-isolated-live-convergence-report.json`

- `Regression: #4257`
- `Regression: #4258`
- `Regression: #4259`
- `Regression: #4260`

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
