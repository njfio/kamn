use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DOC_R50: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");
const DOC_R52: &str = include_str!("../../../docs/review/gaps-and-issues-r52.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

fn current_spec_directory_count() -> usize {
    let output = Command::new("git")
        .current_dir(repo_root())
        .args(["ls-files", "specs"])
        .output()
        .expect("git should be available for tracked spec-dir discovery");
    assert!(
        output.status.success(),
        "git ls-files specs failed with status {:?}",
        output.status.code()
    );

    String::from_utf8(output.stdout)
        .expect("git ls-files output should be valid UTF-8")
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('/');
            let root = parts.next()?;
            if root != "specs" {
                return None;
            }
            let top_level = parts.next()?;
            Some(top_level.to_string())
        })
        .collect::<BTreeSet<_>>()
        .len()
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

fn parse_marker_usize(doc: &str, marker_key: &str) -> usize {
    let needle = format!("{marker_key}=");
    let line = doc
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

fn parse_marker_f64(doc: &str, marker_key: &str) -> f64 {
    let needle = format!("{marker_key}=");
    let line = doc
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

fn parse_marker_text(doc: &str, marker_key: &str) -> String {
    let needle = format!("{marker_key}=");
    let line = doc
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    line.split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .trim()
        .to_string()
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

    assert!(DOC_R50.contains(
        "r50_review_spec_volume_remediation_schema_version=kamn.review.spec-volume-remediation-plan.v1"
    ));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_baseline_spec_dirs=750"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_module_count=92"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_target_ratio_max=7.7"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_target_spec_dir_max=708"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_required_reduction=42"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_tranche_count=3"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_min_reduction_per_tranche=14"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_issue_cap_per_tranche=2"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_target_release=r53"));
    assert!(DOC_R50.contains("r50_review_spec_volume_remediation_status=active"));
    assert!(DOC_R50.contains(
        "r50_review_spec_volume_non_regression_schema_version=kamn.review.spec-volume-non-regression-ratchet.v1"
    ));
    assert!(DOC_R50.contains("r50_review_spec_volume_non_regression_baseline_spec_dirs=693"));
    assert!(DOC_R50.contains("r50_review_spec_volume_non_regression_baseline_module_count=93"));
    assert!(DOC_R50.contains("r50_review_spec_volume_non_regression_ratio_max=7.6"));
    assert!(DOC_R50.contains("r50_review_spec_volume_non_regression_spec_dir_max=693"));
    assert!(DOC_R50.contains(
        "Spec-volume guardrail remediation contract active (R50.18) with 3 tranches at minimum 14 reductions each toward <=7.7 ratio."
    ));
}

#[test]
fn integration_r50_spec_volume_remediation_markers_are_consistent() {
    let baseline_spec_dirs = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_remediation_baseline_spec_dirs",
    );
    let module_count =
        parse_marker_usize(DOC_R50, "r50_review_spec_volume_remediation_module_count");
    let target_ratio_max = parse_marker_f64(
        DOC_R50,
        "r50_review_spec_volume_remediation_target_ratio_max",
    );
    let target_spec_dir_max = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_remediation_target_spec_dir_max",
    );
    let required_reduction = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_remediation_required_reduction",
    );

    let tranche_count =
        parse_marker_usize(DOC_R50, "r50_review_spec_volume_remediation_tranche_count");
    let min_reduction_per_tranche = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_remediation_min_reduction_per_tranche",
    );
    let issue_cap_per_tranche = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_remediation_issue_cap_per_tranche",
    );
    let non_regression_baseline_spec_dirs = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_non_regression_baseline_spec_dirs",
    );
    let non_regression_baseline_module_count = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_non_regression_baseline_module_count",
    );
    let non_regression_ratio_max =
        parse_marker_f64(DOC_R50, "r50_review_spec_volume_non_regression_ratio_max");
    let non_regression_spec_dir_max = parse_marker_usize(
        DOC_R50,
        "r50_review_spec_volume_non_regression_spec_dir_max",
    );

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

#[test]
fn functional_r52_post_publication_spec_volume_reduction_markers_present() {
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_post_publication_spec_volume_reduction_schema_version=kamn.review.spec-volume-post-publication-reduction.v1"
    ));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_reduction_tranche_pre_count=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_reduction_tranche_deleted_count=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_reduction_tranche_post_count=<integer>"));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_spec_volume_reduction_tranche_pre_count - r<release>_review_spec_volume_reduction_tranche_deleted_count = r<release>_review_spec_volume_reduction_tranche_post_count"
    ));

    assert!(DOC_R52.contains(
        "r52_review_post_publication_spec_volume_reduction_schema_version=kamn.review.spec-volume-post-publication-reduction.v1"
    ));
    assert!(DOC_R52.contains("r52_review_spec_volume_reduction_tranche_pre_count=707"));
    assert!(DOC_R52.contains("r52_review_spec_volume_reduction_tranche_deleted_count=14"));
    assert!(DOC_R52.contains("r52_review_spec_volume_reduction_tranche_post_count=693"));
    assert!(DOC_R52
        .contains("r52_review_spec_volume_reduction_evidence_command_pre=find specs -mindepth 1 -maxdepth 1 -type d | wc -l"));
    assert!(DOC_R52.contains(
        "r52_review_spec_volume_reduction_evidence_command_post=find specs -mindepth 1 -maxdepth 1 -type d | wc -l"
    ));
}

#[test]
fn integration_r52_post_publication_spec_volume_reduction_markers_are_consistent() {
    let pre = parse_marker_usize(
        DOC_R52,
        "r52_review_spec_volume_reduction_tranche_pre_count",
    );
    let deleted = parse_marker_usize(
        DOC_R52,
        "r52_review_spec_volume_reduction_tranche_deleted_count",
    );
    let post = parse_marker_usize(
        DOC_R52,
        "r52_review_spec_volume_reduction_tranche_post_count",
    );
    let pre_command = parse_marker_text(
        DOC_R52,
        "r52_review_spec_volume_reduction_evidence_command_pre",
    );
    let post_command = parse_marker_text(
        DOC_R52,
        "r52_review_spec_volume_reduction_evidence_command_post",
    );

    assert_eq!(pre, 707, "tranche-12 pre-count marker should remain fixed");
    assert_eq!(
        deleted, 14,
        "tranche-12 deleted-count marker should remain fixed"
    );
    assert_eq!(
        pre.saturating_sub(post),
        deleted,
        "pre/post delta should equal deleted marker count"
    );
    assert_eq!(
        post, 693,
        "tranche-12 post-count marker should remain fixed"
    );
    assert_eq!(
        pre_command, post_command,
        "pre/post evidence commands should remain identical for direct count comparison"
    );
}

#[test]
fn functional_r52_post_publication_spec_volume_guardrail_reconciliation_markers_present() {
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_post_publication_spec_volume_guardrail_reconciliation_schema_version=kamn.review.spec-volume-guardrail-post-publication-reconciliation.v1"
    ));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_guardrail_snapshot_spec_dir_count=<integer>"));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_spec_volume_guardrail_post_publication_spec_dir_count=<integer>"
    ));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_guardrail_post_publication_ratio=<float>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_spec_volume_guardrail_target_ratio_max=<float>"));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_spec_volume_guardrail_post_publication_status=<within_guardrail|breached>"
    ));

    assert!(DOC_R52.contains(
        "r52_review_post_publication_spec_volume_guardrail_reconciliation_schema_version=kamn.review.spec-volume-guardrail-post-publication-reconciliation.v1"
    ));
    assert!(DOC_R52.contains("r52_review_spec_volume_guardrail_snapshot_spec_dir_count=845"));
    assert!(DOC_R52.contains("r52_review_spec_volume_guardrail_snapshot_module_count=92"));
    assert!(
        DOC_R52.contains("r52_review_spec_volume_guardrail_post_publication_spec_dir_count=693")
    );
    assert!(DOC_R52.contains("r52_review_spec_volume_guardrail_post_publication_module_count=92"));
    assert!(DOC_R52.contains("r52_review_spec_volume_guardrail_post_publication_ratio=7.5"));
    assert!(DOC_R52.contains("r52_review_spec_volume_guardrail_target_ratio_max=7.7"));
    assert!(DOC_R52
        .contains("r52_review_spec_volume_guardrail_post_publication_status=within_guardrail"));
    assert!(DOC_R52.contains("spec_volume_guardrail_target_status=severely_breached"));
}

#[test]
fn integration_r52_post_publication_spec_volume_guardrail_reconciliation_markers_are_consistent() {
    let snapshot_spec_dir_count = parse_marker_usize(
        DOC_R52,
        "r52_review_spec_volume_guardrail_snapshot_spec_dir_count",
    );
    let snapshot_module_count = parse_marker_usize(
        DOC_R52,
        "r52_review_spec_volume_guardrail_snapshot_module_count",
    );
    let post_publication_spec_dir_count = parse_marker_usize(
        DOC_R52,
        "r52_review_spec_volume_guardrail_post_publication_spec_dir_count",
    );
    let post_publication_module_count = parse_marker_usize(
        DOC_R52,
        "r52_review_spec_volume_guardrail_post_publication_module_count",
    );
    let post_publication_ratio = parse_marker_f64(
        DOC_R52,
        "r52_review_spec_volume_guardrail_post_publication_ratio",
    );
    let target_ratio_max =
        parse_marker_f64(DOC_R52, "r52_review_spec_volume_guardrail_target_ratio_max");
    let post_publication_status = parse_marker_text(
        DOC_R52,
        "r52_review_spec_volume_guardrail_post_publication_status",
    );

    assert_eq!(
        snapshot_spec_dir_count, 845,
        "snapshot spec-dir marker should remain fixed"
    );
    assert_eq!(
        snapshot_module_count, 92,
        "snapshot module-count marker should remain fixed"
    );
    assert_eq!(
        post_publication_spec_dir_count, 693,
        "post-publication spec-dir marker should remain fixed"
    );
    assert_eq!(
        post_publication_module_count, 92,
        "post-publication module-count marker should remain fixed"
    );
    assert!(
        post_publication_spec_dir_count <= snapshot_spec_dir_count,
        "post-publication spec-dir count should not exceed snapshot baseline"
    );
    assert!(
        post_publication_ratio <= target_ratio_max,
        "post-publication ratio must remain within guardrail max"
    );
    assert_eq!(
        post_publication_status, "within_guardrail",
        "post-publication guardrail status should remain within_guardrail"
    );

    let computed_ratio =
        post_publication_spec_dir_count as f64 / post_publication_module_count as f64;
    assert!(
        (computed_ratio - post_publication_ratio).abs() <= 0.05,
        "post-publication ratio marker should match counts with one-decimal precision"
    );
}

#[test]
fn regression_r50_spec_volume_non_regression_ignores_untracked_top_level_specs_dirs() {
    let specs_dir = repo_root().join("specs");
    let temp_dir = specs_dir.join(format!(
        "zz-untracked-spec-dir-contamination-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let baseline = current_spec_directory_count();
    fs::create_dir_all(&temp_dir).unwrap_or_else(|error| {
        panic!(
            "failed creating temp specs dir {}: {error}",
            temp_dir.display()
        )
    });
    let observed = current_spec_directory_count();
    let _ = fs::remove_dir_all(&temp_dir);

    assert_eq!(
        observed, baseline,
        "spec-dir non-regression count must ignore untracked top-level specs directories"
    );
}
