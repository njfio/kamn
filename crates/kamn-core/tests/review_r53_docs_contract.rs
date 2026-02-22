use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r53.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

fn parse_marker_lines(doc: &str) -> BTreeMap<String, String> {
    let mut markers = BTreeMap::new();
    for raw_line in doc.lines() {
        let trimmed = raw_line.trim();
        let Some(candidate) = trimmed.strip_prefix("- ") else {
            continue;
        };
        let Some((key, value)) = candidate.split_once('=') else {
            continue;
        };
        markers.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    markers
}

fn parse_required_keys_for_r53(readme: &str) -> BTreeSet<String> {
    let mut required = BTreeSet::new();
    let mut in_required_block = false;

    for raw_line in readme.lines() {
        let trimmed = raw_line.trim();

        if trimmed.starts_with("Required marker keys") {
            in_required_block = true;
            continue;
        }
        if trimmed.starts_with("Optional ")
            || trimmed == "Contract invariants:"
            || trimmed.starts_with("This schema is enforced")
        {
            in_required_block = false;
            continue;
        }
        if !in_required_block {
            continue;
        }

        let Some(marker_line) = trimmed.strip_prefix("- `") else {
            continue;
        };
        let Some(marker_line) = marker_line.strip_suffix('`') else {
            continue;
        };
        let Some((key, _)) = marker_line.split_once('=') else {
            continue;
        };
        required.insert(key.replace("r<release>", "r53"));
    }

    required
}

fn parse_marker_value<'a>(markers: &'a BTreeMap<String, String>, key: &str) -> &'a str {
    markers
        .get(key)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("missing marker {key}"))
}

fn parse_marker_usize(markers: &BTreeMap<String, String>, key: &str) -> usize {
    parse_marker_value(markers, key)
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("marker {key} should be an unsigned integer"))
}

fn parse_marker_f64(markers: &BTreeMap<String, String>, key: &str) -> f64 {
    parse_marker_value(markers, key)
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("marker {key} should be a float"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn top_level_spec_dir_count() -> usize {
    fs::read_dir(repo_root().join("specs"))
        .expect("specs dir should be readable")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .count()
}

fn doc_contract_test_file_count() -> usize {
    fs::read_dir(repo_root().join("crates").join("kamn-core").join("tests"))
        .expect("kamn-core test dir should be readable")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return false;
            };
            name.ends_with("_docs.rs") || name.contains("docs_contract")
        })
        .count()
}

#[test]
fn functional_r53_required_review_marker_keys_present() {
    let markers = parse_marker_lines(DOC);
    let required = parse_required_keys_for_r53(REVIEW_MARKER_README);

    assert!(
        required.len() >= 70,
        "README required-key set unexpectedly small for R53: {}",
        required.len()
    );

    let missing = required
        .iter()
        .filter(|key| !markers.contains_key((*key).as_str()))
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "R53 review doc missing required marker keys: {}",
        missing.join(", ")
    );
}

#[test]
fn integration_r53_review_markers_are_consistent() {
    let markers = parse_marker_lines(DOC);

    let governance_count = parse_marker_usize(&markers, "governance_activity_commit_count");
    let feature_count = parse_marker_usize(&markers, "feature_activity_commit_count");
    let total_count = parse_marker_usize(&markers, "activity_total_commit_count");
    let governance_ratio = parse_marker_f64(&markers, "governance_activity_commit_ratio");
    let feature_ratio = parse_marker_f64(&markers, "feature_activity_commit_ratio");
    assert_eq!(governance_count + feature_count, total_count);
    assert!(total_count > 0);
    assert!((governance_ratio + feature_ratio - 1.0).abs() <= 0.001);
    assert!((governance_ratio - governance_count as f64 / total_count as f64).abs() <= 0.001);
    assert!((feature_ratio - feature_count as f64 / total_count as f64).abs() <= 0.001);

    assert_eq!(
        parse_marker_value(&markers, "review_snapshot_semantics_policy_schema_version"),
        "kamn.review.snapshot-semantics-policy.v1"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r53_review_branch_remote_head_count_contract_mode"
        ),
        "informational_only"
    );
    let chain_count = parse_marker_usize(
        &markers,
        "r53_review_branch_reconciliation_issue_chain_count",
    );
    let chain_max =
        parse_marker_usize(&markers, "r53_review_branch_reconciliation_issue_chain_max");
    assert_eq!(chain_max, 1);
    assert!(chain_count <= chain_max);

    let cleanup_pre =
        parse_marker_usize(&markers, "r53_review_branch_remote_head_count_pre_cleanup");
    let cleanup_deleted =
        parse_marker_usize(&markers, "r53_review_branch_remote_head_count_deleted");
    let cleanup_post =
        parse_marker_usize(&markers, "r53_review_branch_remote_head_count_post_cleanup");
    assert_eq!(cleanup_pre.saturating_sub(cleanup_post), cleanup_deleted);

    assert_eq!(
        parse_marker_value(
            &markers,
            "r53_review_workspace_quality_gate_command_post_publication"
        ),
        "cargo test --workspace --locked --all-features --no-fail-fast"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r53_review_activity_ratio_marker_parse_command_post_publication"
        ),
        "cargo test -p kamn-core --test release_review_activity_ratio_docs_contract"
    );

    let code_quality_workspace = parse_marker_value(
        &markers,
        "r53_review_code_quality_post_publication_workspace_gate_status",
    );
    let quality_gate_workspace = parse_marker_value(
        &markers,
        "r53_review_workspace_quality_gate_status_post_publication",
    );
    assert_eq!(code_quality_workspace, quality_gate_workspace);

    let branch_hygiene_snapshot =
        parse_marker_usize(&markers, "r53_review_branch_hygiene_snapshot_branch_count");
    let branch_hygiene_pre = parse_marker_usize(
        &markers,
        "r53_review_branch_hygiene_post_publication_pre_cleanup_count",
    );
    let branch_hygiene_post = parse_marker_usize(
        &markers,
        "r53_review_branch_hygiene_post_publication_post_cleanup_count",
    );
    assert!(branch_hygiene_post <= branch_hygiene_snapshot);
    assert_eq!(branch_hygiene_pre, cleanup_pre);
    assert_eq!(branch_hygiene_post, cleanup_post);

    let target_governance = parse_marker_f64(
        &markers,
        "r53_review_governance_feature_target_governance_ratio_max",
    );
    let target_feature = parse_marker_f64(
        &markers,
        "r53_review_governance_feature_target_feature_ratio_min",
    );
    assert!((target_governance + target_feature - 1.0).abs() <= 0.001);

    let feat_mislabeled = parse_marker_usize(
        &markers,
        "r53_review_feat_labeling_snapshot_mislabeled_feat_count",
    );
    let feat_total = parse_marker_usize(
        &markers,
        "r53_review_feat_labeling_snapshot_total_feat_count",
    );
    let feat_ratio = parse_marker_f64(
        &markers,
        "r53_review_feat_labeling_snapshot_mislabeled_ratio",
    );
    assert!(feat_total > 0);
    assert!((feat_ratio - feat_mislabeled as f64 / feat_total as f64).abs() <= 0.001);

    let reduction_pre = parse_marker_usize(
        &markers,
        "r53_review_spec_volume_reduction_tranche_pre_count",
    );
    let reduction_deleted = parse_marker_usize(
        &markers,
        "r53_review_spec_volume_reduction_tranche_deleted_count",
    );
    let reduction_post = parse_marker_usize(
        &markers,
        "r53_review_spec_volume_reduction_tranche_post_count",
    );
    assert_eq!(
        reduction_pre.saturating_sub(reduction_post),
        reduction_deleted
    );

    let guardrail_snapshot = parse_marker_usize(
        &markers,
        "r53_review_spec_volume_guardrail_snapshot_spec_dir_count",
    );
    let guardrail_post = parse_marker_usize(
        &markers,
        "r53_review_spec_volume_guardrail_post_publication_spec_dir_count",
    );
    let guardrail_ratio = parse_marker_f64(
        &markers,
        "r53_review_spec_volume_guardrail_post_publication_ratio",
    );
    let guardrail_ratio_max = parse_marker_f64(
        &markers,
        "r53_review_spec_volume_guardrail_target_ratio_max",
    );
    assert!(guardrail_post <= guardrail_snapshot);
    assert!(guardrail_ratio <= guardrail_ratio_max + 0.001);

    let non_regression_spec_dir_max = parse_marker_usize(
        &markers,
        "r53_review_spec_volume_non_regression_spec_dir_max",
    );
    assert!(top_level_spec_dir_count() <= non_regression_spec_dir_max);

    let non_regression_doc_max = parse_marker_usize(
        &markers,
        "r53_review_doc_contract_non_regression_max_test_file_count",
    );
    assert!(doc_contract_test_file_count() <= non_regression_doc_max);

    assert!(DOC.contains(
        "| **Critical** | Governance loop at 99% — 4th consecutive governance-dominated cycle | Process | Fundamental process change needed | **SEVERELY WORSENED** |"
    ));
    assert!(DOC.contains(
        "| **High** | Post-publication reconciliation meta-loop | R52 doc: 283→415 lines, 9 post-pub sections, 60 markers | Stop post-pub appending | **NEW** |"
    ));
}
