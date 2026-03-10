use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_UNIFIED_API_OBSERVABILITY_PAYLOAD_TAXONOMY_GATE_MARKERS_MARKERS: &[&str] = &[
    "## Unified API-Observability Payload Taxonomy Gate (Issue #4507)",
    "check_unified_api_observability_local_heavy_live_policy.sh --report-file /tmp/unified-api-observability-local-heavy-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/unified-api-observability-local-heavy-policy.json",
    "reason_taxonomy_version=kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1",
    "unified_api_observability_local_heavy_policy_correlation_schema_version_mismatch",
    "unified_api_observability_local_heavy_policy_correlation_required_fields_mismatch",
    "unified_api_observability_local_heavy_policy_correlation_id_propagation_mismatch",
    "correlation_schema_version=kamn.runtime.unified-api-observability-correlation-schema.v1",
    "correlation_required_fields_csv=correlation_id,trace_id,trace_parent,span_id,request_id",
    "correlation_id_propagation_status=verified",
];

#[test]
fn checklist_contains_unified_api_observability_payload_taxonomy_gate_markers() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_UNIFIED_API_OBSERVABILITY_PAYLOAD_TAXONOMY_GATE_MARKERS_MARKERS, "checklist_contains_unified_api_observability_payload_taxonomy_gate_markers");
}

const CHECKLIST_CONTAINS_PANIC_REPLACEMENT_REASON_TAXONOMY_AND_RUNTIME_EVIDENCE_GATE_MARKERS: &[&str] = &[
    "## Panic-Replacement Reason Taxonomy and Runtime Evidence Gate (Issue #4455)",
    "scripts/ci/check_no_production_expect.sh --root crates/kamn-node/src --output-json /tmp/no-production-expect-report.json",
    "panic_replacement_reason_taxonomy_version=kamn.ci.production-panic-replacement-reason-taxonomy.v1",
    "panic_replacement_reason_codes_csv=scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default",
    "panic_replacement_reason_codes_value=none|<csv>",
    "panic_replacement_reason_class=stable|panic_reachability|unsafe_fallback|mixed|configuration",
    "runtime_panic_replacement_evidence_status=verified|violation",
    "runtime_panic_replacement_evidence_violation_count=<n>",
    "runtime_panic_replacement_evidence_files_csv=none|<csv>",
    "runtime_panic_replacement_evidence_outputs_csv=runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv",
    "Regression: #4455",
];

#[test]
fn checklist_contains_panic_replacement_reason_taxonomy_and_runtime_evidence_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_PANIC_REPLACEMENT_REASON_TAXONOMY_AND_RUNTIME_EVIDENCE_GATE_MARKERS, "checklist_contains_panic_replacement_reason_taxonomy_and_runtime_evidence_gate");
}

const CHECKLIST_CONTAINS_DEPENDENCY_LICENSE_METADATA_DOCS_MISMATCH_GATE_MARKERS: &[&str] = &[
    "## Dependency-License Metadata/Docs Mismatch Gate (Issue #4456)",
    "scripts/ci/test_check_workspace_license_policy.sh",
    "scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh",
    "license_mismatch",
    "license_missing",
    "manifest_invalid_toml",
    "package_section_missing",
    "readme_webpki_roots_reference_missing",
    "readme_no_default_features_marker_missing",
    "ci_strategy_no_default_features_check_missing",
    "Regression: #4456",
];

#[test]
fn checklist_contains_dependency_license_metadata_docs_mismatch_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_DEPENDENCY_LICENSE_METADATA_DOCS_MISMATCH_GATE_MARKERS, "checklist_contains_dependency_license_metadata_docs_mismatch_gate");
}

const CHECKLIST_CONTAINS_MACHINE_READABLE_BUNDLE_CONTRACT_MARKERS: &[&str] = &[
    "## Machine-Readable Evidence Bundle Contract",
    "gonogo_evidence_contract.py",
    "generate_gonogo_evidence_bundle.sh",
    "check_gonogo_evidence_policy.sh",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract",
    "run_gonogo_evidence_deep_lane.sh",
];

#[test]
fn checklist_contains_machine_readable_bundle_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_MACHINE_READABLE_BUNDLE_CONTRACT_MARKERS, "checklist_contains_machine_readable_bundle_contract");
}

const CHECKLIST_CONTAINS_TLS_EVIDENCE_COMPLETENESS_FRESHNESS_GATE_MARKERS: &[&str] = &[
    "## TLS Evidence Completeness/Freshness Convergence Gate (Issue #4477)",
    "--tls-evidence-report-file /tmp/kamn-core-live-https-dependency-posture-report.json",
    "--tls-evidence-max-age-seconds 1800",
    "tls_evidence_reason_taxonomy_version=kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1",
    "tls_evidence_reason_codes_csv=gonogo_tls_evidence_file_missing,gonogo_tls_evidence_invalid_json,gonogo_tls_evidence_schema_mismatch,gonogo_tls_evidence_status_not_pass,gonogo_tls_evidence_reason_taxonomy_version_mismatch,gonogo_tls_evidence_freshness_window_exceeded",
    "tls_evidence_reason_codes_value=none|<csv>",
    "generator and policy-checker command output must both project this marker set.",
    "invalid TLS evidence JSON must reject with `gonogo_tls_evidence_invalid_json`",
    "tls_evidence_gate_final_decision=GO|NO-GO",
    "Regression: #4298",
    "Regression: #4477",
];

#[test]
fn checklist_contains_tls_evidence_completeness_freshness_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_TLS_EVIDENCE_COMPLETENESS_FRESHNESS_GATE_MARKERS, "checklist_contains_tls_evidence_completeness_freshness_gate");
}

const CHECKLIST_CONTAINS_AUDIT_INTEGRITY_CONVERGENCE_GATE_MARKERS: &[&str] = &[
    "## Audit-Trail Integrity/Tamper Convergence Gate (Issue #4466)",
    "--audit-integrity-report-file /tmp/sqlite-crash-recovery-live-policy-report.json",
    "--audit-integrity-max-age-seconds 1800",
    "audit_integrity_reason_taxonomy_version=kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1",
    "audit_integrity_reason_codes_csv=gonogo_audit_integrity_file_missing,gonogo_audit_integrity_invalid_json,gonogo_audit_integrity_schema_mismatch,gonogo_audit_integrity_status_not_ok,gonogo_audit_integrity_final_decision_not_go,gonogo_audit_integrity_policy_status_not_verified,gonogo_audit_integrity_reason_taxonomy_version_mismatch,gonogo_audit_integrity_reason_codes_csv_mismatch,gonogo_audit_integrity_freshness_window_exceeded",
    "audit_integrity_gate_final_decision=GO|NO-GO",
    "Regression: #4466",
];

#[test]
fn checklist_contains_audit_integrity_convergence_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_AUDIT_INTEGRITY_CONVERGENCE_GATE_MARKERS, "checklist_contains_audit_integrity_convergence_gate");
}

const CHECKLIST_CONTAINS_JOURNAL_APPEND_CHECKPOINT_INTEGRITY_GATE_MARKERS: &[&str] = &[
    "## Journal Append/Checkpoint Integrity Determinism Gate (Issues #4236, #4240, #4241)",
    "test_check_sqlite_crash_recovery_live_policy.sh",
    "test_validate_sqlite_crash_recovery_live.sh",
    "test_validate_sqlite_crash_recovery_live_contract_lane.sh",
    "append_checkpoint_integrity_status=verified",
    "append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1",
    "append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch",
    "sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch",
    "Regression: #4240",
    "Regression: #4241",
];

#[test]
fn checklist_contains_journal_append_checkpoint_integrity_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_JOURNAL_APPEND_CHECKPOINT_INTEGRITY_GATE_MARKERS, "checklist_contains_journal_append_checkpoint_integrity_gate");
}

const CHECKLIST_CONTAINS_REPLAY_IDEMPOTENCY_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS: &[&str] = &[
    "## Replay Idempotency Taxonomy/Runbook Parity Gate (Issues #4237, #4242, #4243)",
    "check_sqlite_crash_recovery_live_policy.sh --report-file /tmp/sqlite-crash-recovery-live-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/sqlite-crash-recovery-live-policy-report.json",
    "replay_idempotency_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
    "replay_idempotency_runbook_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-replay-idempotency-runbook-reason-taxonomy.v1",
    "replay_idempotency_runbook_reason_codes_csv=replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "replay_idempotency_runbook_reason_code=none|<reason>",
    "replay_idempotency_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_taxonomy_version_mismatch",
    "sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_codes_csv_mismatch",
    "Regression: #4242",
    "Regression: #4243",
];

#[test]
fn checklist_contains_replay_idempotency_taxonomy_runbook_parity_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_REPLAY_IDEMPOTENCY_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS, "checklist_contains_replay_idempotency_taxonomy_runbook_parity_gate");
}

const CHECKLIST_CONTAINS_CRASH_REPLAY_EVIDENCE_CONVERGENCE_AND_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Crash-Replay Evidence Convergence/Promotion Reason Mapping Gate (Issues #4238, #4244, #4245)",
    "test_check_sqlite_crash_recovery_live_evidence_convergence.sh",
    "check_sqlite_crash_recovery_live_evidence_convergence.sh --report-file /tmp/sqlite-crash-recovery-live-contract-lane-report.json --policy-file /tmp/sqlite-crash-recovery-live-policy-report.json --output-json /tmp/sqlite-crash-recovery-live-convergence-report.json",
    "sqlite_crash_replay_evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "sqlite_crash_replay_evidence_reason_taxonomy_version=kamn.runtime.sqlite-crash-replay-evidence-convergence-reason-taxonomy.v1",
    "sqlite_crash_replay_evidence_reason_codes_csv=sqlite_crash_replay_evidence_link_missing,sqlite_crash_replay_evidence_payload_tamper_detected,sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=sqlite_crash_recovery_policy_required_field_missing,sqlite_crash_recovery_policy_marker_missing,sqlite_crash_recovery_policy_reason_taxonomy_mismatch,sqlite_crash_recovery_policy_runtime_mode_contract_mismatch,replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_recovery_policy_expected_decision_mismatch,sqlite_crash_recovery_policy_violation",
    "promotion_decision_reason_code=none|<reason>",
    "sqlite_crash_replay_evidence_link_missing:source_report_file",
    "sqlite_crash_replay_evidence_payload_tamper_detected:<field>",
    "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    "Regression: #4244",
    "Regression: #4245",
];

#[test]
fn checklist_contains_crash_replay_evidence_convergence_and_mapping_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_CRASH_REPLAY_EVIDENCE_CONVERGENCE_AND_MAPPING_GATE_MARKERS, "checklist_contains_crash_replay_evidence_convergence_and_mapping_gate");
}
