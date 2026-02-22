use std::fs;
use std::path::{Path, PathBuf};

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

fn current_spec_directory_count() -> usize {
    let specs_dir = repo_root().join("specs");
    fs::read_dir(specs_dir)
        .expect("specs directory should be readable")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .count()
}

fn current_module_export_count() -> usize {
    let lib_rs = repo_root()
        .join("crates")
        .join("kamn-core")
        .join("src")
        .join("lib.rs");
    fs::read_to_string(lib_rs)
        .expect("kamn-core lib.rs should be readable")
        .lines()
        .filter(|line| line.trim_start().starts_with("pub mod "))
        .count()
}

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
    assert!(REVIEW_MARKER_README.contains("r<release>_review_spec_volume_non_regression_schema_version=kamn.review.spec-volume-non-regression-ratchet.v1"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_non_regression_spec_dir_max=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_non_regression_ratio_max=<float>"));
    assert!(REVIEW_MARKER_README.contains("current spec_dir_count <= non_regression_spec_dir_max"));
    assert!(
        REVIEW_MARKER_README.contains("current spec_to_module_ratio <= non_regression_ratio_max")
    );

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
        "r50_review_spec_volume_non_regression_schema_version=kamn.review.spec-volume-non-regression-ratchet.v1"
    ));
    assert!(DOC.contains("r50_review_spec_volume_non_regression_baseline_spec_dirs=822"));
    assert!(DOC.contains("r50_review_spec_volume_non_regression_baseline_module_count=92"));
    assert!(DOC.contains("r50_review_spec_volume_non_regression_ratio_max=9.0"));
    assert!(DOC.contains("r50_review_spec_volume_non_regression_spec_dir_max=822"));
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
    let non_regression_baseline_spec_dirs =
        parse_marker_usize("r50_review_spec_volume_non_regression_baseline_spec_dirs");
    let non_regression_baseline_module_count =
        parse_marker_usize("r50_review_spec_volume_non_regression_baseline_module_count");
    let non_regression_ratio_max =
        parse_marker_f64("r50_review_spec_volume_non_regression_ratio_max");
    let non_regression_spec_dir_max =
        parse_marker_usize("r50_review_spec_volume_non_regression_spec_dir_max");

    let current_spec_dirs = current_spec_directory_count();
    let current_module_count = current_module_export_count();
    let current_ratio = current_spec_dirs as f64 / current_module_count as f64;

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
    assert!(
        non_regression_baseline_spec_dirs <= non_regression_spec_dir_max,
        "non-regression baseline spec-dir count must be <= non-regression max"
    );
    assert_eq!(
        non_regression_baseline_spec_dirs, non_regression_spec_dir_max,
        "non-regression max should remain locked to baseline while remediation is active"
    );
    assert_eq!(
        non_regression_baseline_module_count, current_module_count,
        "non-regression module baseline should match current exported module count"
    );
    assert!(
        current_spec_dirs <= non_regression_spec_dir_max,
        "current spec-dir count must not exceed non-regression cap"
    );
    assert!(
        current_ratio <= non_regression_ratio_max,
        "current spec-to-module ratio must not exceed non-regression max"
    );
}
