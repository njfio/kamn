const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r48.md");

fn parse_marker_usize(marker_key: &str) -> usize {
    let needle = format!("{marker_key}=");
    let line = DOC
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    let value = line
        .split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .trim();
    value
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be an unsigned integer: {value}"))
}

fn parse_marker_f64(marker_key: &str) -> f64 {
    let needle = format!("{marker_key}=");
    let line = DOC
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    let value = line
        .split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .trim();
    value
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be a float: {value}"))
}

#[test]
fn functional_r48_review_spec_volume_and_coherence_markers_present() {
    assert!(DOC.contains(
        "coherence_contract_batching_policy_schema_version=kamn.review.coherence-contract-batching-policy.v1"
    ));
    assert!(DOC.contains("coherence_contract_batching_dimension_baseline=28"));
    assert!(DOC.contains("coherence_contract_batching_issue_baseline=28"));
    assert!(DOC.contains("coherence_contract_batching_target_bundle_count_min=5"));
    assert!(DOC.contains("coherence_contract_batching_target_bundle_count_max=8"));
    assert!(DOC.contains("coherence_contract_batching_target_issue_cap=8"));
    assert!(DOC.contains("coherence_contract_batching_expected_issue_reduction=20"));

    assert!(DOC.contains(
        "spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1"
    ));
    assert!(DOC.contains("spec_volume_guardrail_baseline_spec_directory_count=700"));
    assert!(DOC.contains("spec_volume_guardrail_baseline_module_count=91"));
    assert!(DOC.contains("spec_volume_guardrail_baseline_spec_to_module_ratio=7.7"));
    assert!(DOC.contains("spec_volume_guardrail_target_spec_to_module_ratio_max=7.7"));
    assert!(DOC.contains("spec_volume_guardrail_target_status=monitor"));
    assert!(DOC.contains(
        "spec_volume_guardrail_evidence_command_spec_dirs=find specs -maxdepth 1 -mindepth 1 -type d | wc -l"
    ));
    assert!(DOC.contains(
        "spec_volume_guardrail_evidence_command_module_exports=rg \"^pub mod \" crates/kamn-core/src/lib.rs | wc -l"
    ));
}

#[test]
fn integration_r48_review_spec_volume_and_coherence_markers_are_consistent() {
    let issue_baseline = parse_marker_usize("coherence_contract_batching_issue_baseline");
    let issue_cap = parse_marker_usize("coherence_contract_batching_target_issue_cap");
    let expected_reduction =
        parse_marker_usize("coherence_contract_batching_expected_issue_reduction");
    assert_eq!(
        issue_baseline.saturating_sub(issue_cap),
        expected_reduction,
        "coherence expected reduction should match baseline minus issue cap"
    );

    let spec_dir_count = parse_marker_usize("spec_volume_guardrail_baseline_spec_directory_count");
    let module_count = parse_marker_usize("spec_volume_guardrail_baseline_module_count");
    let ratio_reported = parse_marker_f64("spec_volume_guardrail_baseline_spec_to_module_ratio");
    let ratio_target_max =
        parse_marker_f64("spec_volume_guardrail_target_spec_to_module_ratio_max");

    let computed_ratio = spec_dir_count as f64 / module_count as f64;
    assert!(
        (computed_ratio - ratio_reported).abs() <= 0.1,
        "reported ratio should approximate computed baseline ratio"
    );
    assert!(
        ratio_reported <= ratio_target_max,
        "reported ratio should be <= target max"
    );
}
