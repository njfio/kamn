use super::*;

const LIFECYCLE_CI_DRY_RUN_MARKERS: &[&str] = &[
    "## Lifecycle Artifact CI Dry-Run Governance Contract (Issue #4082)",
    "lifecycle_ci_dry_run_reason_taxonomy_version=kamn.ci.lifecycle-ci-dry-run-governance-reason-taxonomy.v1",
    "lifecycle_ci_dry_run_reason_codes_csv=lifecycle_ci_dry_run_argument_invalid,lifecycle_ci_dry_run_threshold_contract_violation,lifecycle_ci_dry_run_report_contract_violation,lifecycle_ci_dry_run_lifecycle_marker_parity_drift,lifecycle_ci_dry_run_go_no_go_marker_parity_drift,lifecycle_ci_dry_run_runtime_budget_exceeded,lifecycle_ci_dry_run_fast_mode_selector_drift,lifecycle_ci_dry_run_workflow_exclusion_drift,lifecycle_ci_dry_run_docs_marker_parity_drift,lifecycle_ci_dry_run_docs_remediation_marker_missing",
    "lifecycle_ci_dry_run_threshold_fixture_path=fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env",
    "lifecycle_ci_dry_run_max_seconds=120",
    "lifecycle_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture",
    "lifecycle_ci_dry_run_fast_mode_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh\" --mode run",
    "lifecycle_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/run_go_no_go_gate_lane.sh --mode run",
    "python3 scripts/ci/check_lifecycle_ci_dry_run_governance.py --lifecycle-artifact-bundle-file /tmp/lifecycle-artifact-integrity-baseline.json --go-no-go-gate-report-file /tmp/go-no-go-gate-report.json --threshold-file fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --output-json /tmp/lifecycle-ci-dry-run-governance-report.json",
    "cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture",
    "lifecycle_ci_dry_run_remediation_map_version=v1",
    "Regression: #4082",
];

const LIFECYCLE_CI_DRY_RUN_REMEDIATION_CODES: &[&str] = &[
    "lifecycle_ci_dry_run_argument_invalid",
    "lifecycle_ci_dry_run_threshold_contract_violation",
    "lifecycle_ci_dry_run_report_contract_violation",
    "lifecycle_ci_dry_run_lifecycle_marker_parity_drift",
    "lifecycle_ci_dry_run_go_no_go_marker_parity_drift",
    "lifecycle_ci_dry_run_runtime_budget_exceeded",
    "lifecycle_ci_dry_run_fast_mode_selector_drift",
    "lifecycle_ci_dry_run_workflow_exclusion_drift",
    "lifecycle_ci_dry_run_docs_marker_parity_drift",
    "lifecycle_ci_dry_run_docs_remediation_marker_missing",
];

#[test]
fn service_api_ops_configuration_contains_lifecycle_artifact_integrity_markers() {
    assert_doc_contains_all(&["## Tamper-Evident Lifecycle Artifact Integrity Contract (Issue #4081)", "lifecycle_artifact_integrity_schema_version=kamn.runtime.lifecycle-artifact-integrity-evidence.v1", "lifecycle_artifact_integrity_artifact_schema_version=kamn.runtime.lifecycle-artifact-integrity-schema.v1", "lifecycle_artifact_integrity_reason_taxonomy_version=kamn.runtime.lifecycle-artifact-integrity-reason-taxonomy.v1", "lifecycle_artifact_integrity_reason_codes_csv=lifecycle_artifact_required_field_missing,lifecycle_artifact_marker_mismatch,lifecycle_artifact_hash_mismatch,lifecycle_artifact_reason_taxonomy_mismatch,lifecycle_artifact_reason_codes_csv_mismatch,lifecycle_artifact_expected_decision_mismatch", "lifecycle_artifact_integrity_hash_fields_csv=payload_hash_sha256,integrity_hash_sha256,provenance_hash_sha256", "bash scripts/runtime/generate_lifecycle_artifact_integrity_evidence_bundle.sh --output-file /tmp/lifecycle-artifact-integrity-baseline.json --artifact-id lifecycle-artifact-baseline --lifecycle-stage retention --profile baseline --record-count 42 --ci-fast-gate PASS", "bash scripts/runtime/check_lifecycle_artifact_integrity_evidence_bundle.sh --bundle-file /tmp/lifecycle-artifact-integrity-baseline.json --expected-final-decision GO", "cargo test -p kamn-core --test lifecycle_artifact_integrity_contract -- --nocapture", "Regression: #4081"]);
}
#[test]
fn service_api_ops_configuration_contains_lifecycle_ci_dry_run_governance_markers() {
    assert_doc_contains_all(LIFECYCLE_CI_DRY_RUN_MARKERS);
    assert_doc_contains_prefixed_entries("lifecycle_ci_dry_run_remediation", LIFECYCLE_CI_DRY_RUN_REMEDIATION_CODES);
}
#[test]
fn service_api_ops_configuration_contains_realtime_presence_mode_and_guardrail_markers() {
    assert_doc_contains_all(&["## Realtime Presence Mode Gateway and Guardrail Contracts (Issues #5279, #5281, #5283)", "service_api_ws_presence_mode_status=verified", "service_api_ws_events_mode_header=x-kamn-events-mode", "service_api_ws_events_mode_presence_value=presence", "service_api_ws_presence_required_headers_csv=x-kamn-presence-owner-did,x-kamn-presence-target-agent-did,x-kamn-requester-agent-did", "service_api_ws_presence_optional_headers_csv=x-kamn-presence-target-owner-did,x-kamn-presence-gateway-node,x-kamn-presence-connected-since,x-kamn-presence-last-heartbeat,x-kamn-presence-capabilities", "service_api_ws_presence_fail_closed_reason_codes_csv=service_api_ws_events_mode_invalid,service_api_ws_presence_owner_did_header_missing,service_api_ws_presence_target_agent_did_header_missing,service_api_ws_presence_requester_agent_did_header_missing,m9_realtime_owner_scope_denied,m9_realtime_presence_visibility_denied", "service_api_ws_presence_event_type=m9.presence.snapshot", "service_api_ws_presence_transport_profile=websocket", "realtime_guardrail_burst_validation_status=verified", "replay_duplicate_reason_ordering_status=verified", "cargo test -p kamn-node integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic -- --exact", "cargo test -p kamn-node integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts -- --exact", "cargo test -p kamn-node regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable -- --exact", "Regression: #5283"]);
}
