use super::support::assert_doc_contains_all;

#[test]
fn doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers() {
    assert_doc_contains_all(
        &[
            "ci_smoke_local_heavy_boundary_status=verified|violation",
            "ci_smoke_performance_report_step_missing",
            "ci_smoke_threshold_check_step_missing",
            "local_heavy_opt_in_boundary_missing",
        ],
        "merge gate reliability boundary",
    );
}

#[test]
fn doc_contains_incident_gonogo_boundary_governance_matrix() {
    assert_doc_contains_all(
        &[
            "Incident go/no-go convergence and boundary governance",
            "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120",
            "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900",
            "incident_gonogo_ci_smoke_max_seconds=120",
            "incident_gonogo_local_heavy_max_seconds=900",
            "ci_smoke_lane_cost_profile=low",
            "local_heavy_lane_execution_mode=opt_in",
        ],
        "incident gonogo governance",
    );
}

#[test]
fn doc_contains_incident_gonogo_boundary_reason_taxonomy_markers() {
    assert_doc_contains_all(
        &[
            "incident_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-incident-boundary-reason-taxonomy.v1",
            "incident_gonogo_boundary_reason_codes_csv=incident_gonogo_ci_smoke_seconds_exceeded,incident_gonogo_local_heavy_seconds_exceeded,incident_gonogo_local_heavy_opt_in_missing,incident_gonogo_evidence_convergence_mismatch",
            "incident_gonogo_ci_smoke_seconds_exceeded",
            "incident_gonogo_local_heavy_seconds_exceeded",
            "incident_gonogo_local_heavy_opt_in_missing",
            "incident_gonogo_evidence_convergence_mismatch",
            "Regression: #4471",
        ],
        "incident gonogo reasons",
    );
}

#[test]
fn doc_contains_live_gonogo_boundary_governance_matrix() {
    assert_doc_contains_all(
        &[
            "Live go/no-go convergence and boundary governance",
            "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120",
            "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900",
            "live_gonogo_ci_smoke_max_seconds=120",
            "live_gonogo_local_heavy_max_seconds=900",
            "ci_smoke_lane_cost_profile=low",
            "local_heavy_lane_execution_mode=opt_in",
        ],
        "live gonogo governance",
    );
}

#[test]
fn doc_contains_live_gonogo_boundary_reason_taxonomy_markers() {
    assert_doc_contains_all(live_gonogo_reason_markers(), "live gonogo reasons");
}

fn live_gonogo_reason_markers() -> &'static [&'static str] {
    &[
        "live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1",
        "live_gonogo_boundary_reason_codes_csv=live_gonogo_ci_smoke_seconds_exceeded,live_gonogo_local_heavy_seconds_exceeded,live_gonogo_local_heavy_opt_in_missing,live_gonogo_evidence_convergence_mismatch",
        "live_gonogo_ci_smoke_seconds_exceeded",
        "live_gonogo_local_heavy_seconds_exceeded",
        "live_gonogo_local_heavy_opt_in_missing",
        "live_gonogo_evidence_convergence_mismatch",
        "deployment_safety_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1",
        "deployment_safety_gate_reason_codes_csv=none|<csv>",
        "deployment_safety_gate_reason_codes_value=none|<csv>",
        "contracts.deployment_preflight_rotation_reason_taxonomy_version_required=kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1",
        "contracts.go_no_go_gate_ci_local_boundary_contract_required=true",
        "milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch",
        "milestone_review_deployment_preflight_policy_rotation_reason_codes_value_mismatch",
        "milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch",
        "Regression: #4442",
    ]
}
