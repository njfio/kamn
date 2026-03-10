use super::super::DOC;

#[test]
fn doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers() {
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified|violation"));
    assert!(DOC.contains("ci_smoke_performance_report_step_missing"));
    assert!(DOC.contains("ci_smoke_threshold_check_step_missing"));
    assert!(DOC.contains("local_heavy_opt_in_boundary_missing"));
}

#[test]
fn doc_contains_incident_gonogo_boundary_governance_matrix() {
    assert!(DOC.contains("Incident go/no-go convergence and boundary governance"));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120"
    ));
    assert!(DOC.contains(
        "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900"
    ));
    assert!(DOC.contains("incident_gonogo_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("incident_gonogo_local_heavy_max_seconds=900"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
}

#[test]
fn doc_contains_incident_gonogo_boundary_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "incident_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-incident-boundary-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "incident_gonogo_boundary_reason_codes_csv=incident_gonogo_ci_smoke_seconds_exceeded,incident_gonogo_local_heavy_seconds_exceeded,incident_gonogo_local_heavy_opt_in_missing,incident_gonogo_evidence_convergence_mismatch"
    ));
    assert!(DOC.contains("incident_gonogo_ci_smoke_seconds_exceeded"));
    assert!(DOC.contains("incident_gonogo_local_heavy_seconds_exceeded"));
    assert!(DOC.contains("incident_gonogo_local_heavy_opt_in_missing"));
    assert!(DOC.contains("incident_gonogo_evidence_convergence_mismatch"));
    assert!(DOC.contains("Regression: #4471"));
}

#[test]
fn doc_contains_live_gonogo_boundary_governance_matrix() {
    assert!(DOC.contains("Live go/no-go convergence and boundary governance"));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120"
    ));
    assert!(DOC.contains(
        "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900"
    ));
    assert!(DOC.contains("live_gonogo_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("live_gonogo_local_heavy_max_seconds=900"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
}

#[test]
fn doc_contains_live_gonogo_boundary_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "live_gonogo_boundary_reason_codes_csv=live_gonogo_ci_smoke_seconds_exceeded,live_gonogo_local_heavy_seconds_exceeded,live_gonogo_local_heavy_opt_in_missing,live_gonogo_evidence_convergence_mismatch"
    ));
    assert!(DOC.contains("live_gonogo_ci_smoke_seconds_exceeded"));
    assert!(DOC.contains("live_gonogo_local_heavy_seconds_exceeded"));
    assert!(DOC.contains("live_gonogo_local_heavy_opt_in_missing"));
    assert!(DOC.contains("live_gonogo_evidence_convergence_mismatch"));
    assert!(DOC.contains(
        "deployment_safety_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("deployment_safety_gate_reason_codes_csv=none|<csv>"));
    assert!(DOC.contains("deployment_safety_gate_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "contracts.deployment_preflight_rotation_reason_taxonomy_version_required=kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("contracts.go_no_go_gate_ci_local_boundary_contract_required=true"));
    assert!(DOC.contains(
        "milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch"
    ));
    assert!(DOC.contains(
        "milestone_review_deployment_preflight_policy_rotation_reason_codes_value_mismatch"
    ));
    assert!(DOC.contains("milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch"));
    assert!(DOC.contains("Regression: #4442"));
}
