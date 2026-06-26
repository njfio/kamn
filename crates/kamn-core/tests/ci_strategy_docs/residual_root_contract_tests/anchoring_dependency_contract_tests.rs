use super::super::{
    dependency_license_metadata_governance_reason_codes,
    DEPENDENCY_LICENSE_METADATA_GOVERNANCE_REASON_CODES_CSV, DOC, OPS_DOC,
};
use super::support::{assert_doc_contains_all, assert_supply_chain_doc_marker};

const MESSAGE_ANCHORING_MARKERS: &[&str] = &[
    "anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1",
    "anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required",
    "ci_smoke_local_heavy_boundary_status=verified",
    "ci_smoke_lane_cost_profile=low",
    "local_heavy_lane_execution_mode=opt_in",
    "test_run_message_proof_anchoring_contract_lane.sh",
    "test_validate_message_proof_anchoring_live.sh",
];

const DEPENDENCY_LICENSE_METADATA_MARKERS: &[&str] = &[
    "metadata_governance_reason_taxonomy_version=kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1",
    "metadata_governance_reason_codes_value=none|<csv>",
    "metadata_governance_reason_class=stable|metadata_mismatch|configuration|boundary|mixed",
    "ci_smoke_local_heavy_boundary_status=verified|violation",
    "ci_smoke_lane_cost_profile=low|not-applicable",
    "local_heavy_lane_execution_mode=not_requested|opt_in|blocked",
    "python3 scripts/ci/check_workspace_license_policy.py --workspace-root . --expected-license Apache-2.0 --license-policy-file LICENSE --lane-profile ci-smoke",
    "python3 scripts/ci/check_workspace_license_policy.py --workspace-root . --expected-license Apache-2.0 --license-policy-file LICENSE --lane-profile local-heavy --local-heavy-opt-in",
];

#[test]
fn doc_contains_message_anchoring_ci_boundary_taxonomy_markers() {
    assert_doc_contains_all(MESSAGE_ANCHORING_MARKERS, "message anchoring");
}

#[test]
fn doc_contains_dependency_license_metadata_governance_taxonomy_and_boundary_markers() {
    assert!(DOC.contains(&format!(
        "metadata_governance_reason_codes_csv={DEPENDENCY_LICENSE_METADATA_GOVERNANCE_REASON_CODES_CSV}"
    )));
    assert_doc_contains_all(
        DEPENDENCY_LICENSE_METADATA_MARKERS,
        "dependency license metadata governance",
    );
}

#[test]
fn doc_contains_supply_chain_advisory_lane_markers() {
    for marker in [
        "supply_chain_advisory_lane_status=advisory_only",
        "supply_chain_advisory_tools_csv=trivy_fs,trivy_image,workspace_license_policy",
        "supply_chain_advisory_sbom_format=cyclonedx",
        "supply_chain_advisory_false_positive_controls=.trivyignore + workflow continue-on-error",
        "supply_chain_advisory_promotion_follow_up_issue=",
    ] {
        assert_supply_chain_doc_marker(marker);
    }
}

#[test]
fn doc_enforces_dependency_license_metadata_remediation_markers_cover_reason_codes() {
    assert!(DOC.contains("metadata_governance_remediation_map_version=v1"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_dependency_license_metadata_remediation_markers_cover_reason_codes -- --exact"
    ));
    for reason_code in dependency_license_metadata_governance_reason_codes() {
        assert!(
            DOC.contains(&format!("metadata_governance_remediation.{reason_code}=")),
            "missing dependency-license remediation marker for {reason_code}"
        );
        assert!(
            OPS_DOC.contains(&format!("metadata_governance_remediation.{reason_code}=")),
            "ops docs missing dependency-license remediation marker for {reason_code}"
        );
    }
}
