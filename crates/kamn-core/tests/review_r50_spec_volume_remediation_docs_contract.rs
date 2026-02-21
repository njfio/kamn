const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");

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
        .unwrap_or_else(|_| panic!("marker {marker_key} should be a number: {value}"))
}

#[test]
fn functional_r50_spec_volume_remediation_markers_present() {
    assert!(DOC.contains(
        "r50_review_spec_volume_remediation_schema_version=kamn.review.spec-volume-remediation-plan.v1"
    ));
    assert!(DOC.contains("r50_review_spec_volume_remediation_baseline_spec_dirs=750"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_module_count=92"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_target_ratio_max=7.7"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_target_spec_dir_max=708"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_required_reduction=42"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_tranche_count=3"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_min_reduction_per_tranche=14"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_issue_cap_per_tranche=2"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_target_release=r53"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_status=active"));
    assert!(DOC.contains(
        "Spec-volume guardrail remediation contract active (R50.18) with 3 tranches at minimum 14 reductions each toward <=7.7 ratio."
    ));
}

#[test]
fn integration_r50_spec_volume_remediation_markers_are_consistent() {
    let baseline_spec_dirs =
        parse_marker_usize("r50_review_spec_volume_remediation_baseline_spec_dirs");
    let module_count = parse_marker_usize("r50_review_spec_volume_remediation_module_count");
    let target_ratio_max = parse_marker_f64("r50_review_spec_volume_remediation_target_ratio_max");
    let target_spec_dir_max =
        parse_marker_usize("r50_review_spec_volume_remediation_target_spec_dir_max");
    let required_reduction =
        parse_marker_usize("r50_review_spec_volume_remediation_required_reduction");

    let tranche_count = parse_marker_usize("r50_review_spec_volume_remediation_tranche_count");
    let min_reduction_per_tranche =
        parse_marker_usize("r50_review_spec_volume_remediation_min_reduction_per_tranche");
    let issue_cap_per_tranche =
        parse_marker_usize("r50_review_spec_volume_remediation_issue_cap_per_tranche");

    let computed_target_spec_dir_max = (target_ratio_max * module_count as f64).floor() as usize;
    assert_eq!(computed_target_spec_dir_max, target_spec_dir_max);
    assert_eq!(
        baseline_spec_dirs.saturating_sub(target_spec_dir_max),
        required_reduction
    );
    assert!(tranche_count > 0, "tranche count must be positive");
    assert!(
        tranche_count.saturating_mul(min_reduction_per_tranche) >= required_reduction,
        "tranche plan must cover required reduction"
    );
    assert!(
        issue_cap_per_tranche <= 2,
        "per-tranche issue cap must remain tightly bounded"
    );
}
