use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r53.md");
const DOC_R54: &str = include_str!("../../../docs/review/gaps-and-issues-r54.md");
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

fn parse_key_value_lines(doc: &str) -> BTreeMap<String, String> {
    let mut markers = BTreeMap::new();
    for raw_line in doc.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        markers.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    markers
}

fn parse_marker_hex_u64(markers: &BTreeMap<String, String>, key: &str) -> u64 {
    let raw = parse_marker_value(markers, key);
    let hex = raw.trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(hex, 16)
        .unwrap_or_else(|_| panic!("marker {key} should be a hex u64 value"))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn parse_release_from_review_path(path: &str) -> Option<u32> {
    let file = path.rsplit('/').next()?;
    let stem = file.strip_suffix(".md")?;
    let release = stem.strip_prefix("gaps-and-issues-r")?;
    release.parse::<u32>().ok()
}

fn tracked_review_docs() -> Vec<PathBuf> {
    let output = Command::new("git")
        .current_dir(repo_root())
        .args(["ls-files", "docs/review"])
        .output()
        .expect("git should be available for tracked review-doc discovery");
    assert!(
        output.status.success(),
        "git ls-files docs/review failed with status {:?}",
        output.status.code()
    );

    let mut docs = String::from_utf8(output.stdout)
        .expect("git ls-files output should be valid UTF-8")
        .lines()
        .filter(|line| line.starts_with("docs/review/gaps-and-issues-r") && line.ends_with(".md"))
        .map(|line| repo_root().join(line))
        .collect::<Vec<_>>();
    docs.sort();
    docs
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

    assert_eq!(
        parse_marker_value(
            &markers,
            "r53_review_post_publication_portable_agent_reconciliation_schema_version",
        ),
        "kamn.review.portable-agent-post-publication-reconciliation.v1"
    );
    assert_eq!(
        parse_marker_value(&markers, "r53_review_portable_agent_snapshot_status"),
        "stalled"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r53_review_portable_agent_post_publication_status",
        ),
        "advanced_after_query_surfaces"
    );
    let portable_agent_issue =
        parse_marker_usize(&markers, "r53_review_portable_agent_post_publication_issue");
    let portable_agent_pr =
        parse_marker_usize(&markers, "r53_review_portable_agent_post_publication_pr");
    assert!(portable_agent_issue > 0);
    assert!(portable_agent_pr > 0);

    let mcp_snapshot = parse_marker_usize(
        &markers,
        "r53_review_portable_agent_snapshot_mcp_tool_count",
    );
    let mcp_post = parse_marker_usize(
        &markers,
        "r53_review_portable_agent_post_publication_mcp_tool_count",
    );
    let mcp_delta = parse_marker_usize(
        &markers,
        "r53_review_portable_agent_post_publication_delta_mcp_tools",
    );
    assert!(mcp_post >= mcp_snapshot);
    assert_eq!(mcp_post - mcp_snapshot, mcp_delta);

    let cli_snapshot = parse_marker_usize(
        &markers,
        "r53_review_portable_agent_snapshot_cli_subcommand_count",
    );
    let cli_post = parse_marker_usize(
        &markers,
        "r53_review_portable_agent_post_publication_cli_subcommand_count",
    );
    let cli_delta = parse_marker_usize(
        &markers,
        "r53_review_portable_agent_post_publication_delta_cli_subcommands",
    );
    assert!(cli_post >= cli_snapshot);
    assert_eq!(cli_post - cli_snapshot, cli_delta);

    assert!(DOC.contains(
        "| **Critical** | Governance loop at 99% — 4th consecutive governance-dominated cycle | Process | Fundamental process change needed | **SEVERELY WORSENED** |"
    ));
    assert!(DOC.contains(
        "| **High** | Post-publication reconciliation meta-loop | R52 doc: 283→415 lines, 9 post-pub sections, 60 markers | Stop post-pub appending | **NEW** |"
    ));
}

#[test]
fn regression_r53_review_document_freeze_baseline_is_enforced() {
    let freeze_path = repo_root()
        .join("docs")
        .join("review")
        .join("gaps-and-issues-r53.freeze");
    let freeze_doc = fs::read_to_string(&freeze_path).unwrap_or_else(|_| {
        panic!(
            "r53 freeze baseline file missing: {}",
            freeze_path.display()
        )
    });
    let freeze_markers = parse_key_value_lines(&freeze_doc);

    assert_eq!(
        parse_marker_value(&freeze_markers, "r53_review_freeze_schema_version"),
        "kamn.review.document-freeze.v1"
    );
    assert_eq!(
        parse_marker_value(&freeze_markers, "r53_review_freeze_status"),
        "frozen"
    );

    let expected_line_count = parse_marker_usize(&freeze_markers, "r53_review_freeze_line_count");
    let expected_appendix_section_count =
        parse_marker_usize(&freeze_markers, "r53_review_freeze_appendix_section_count");
    let expected_last_non_empty_line =
        parse_marker_value(&freeze_markers, "r53_review_freeze_last_non_empty_line");
    let expected_fnv = parse_marker_hex_u64(&freeze_markers, "r53_review_freeze_fnv1a64_hex");

    let current_line_count = DOC.lines().count();
    let current_appendix_section_count = DOC
        .lines()
        .filter(|line| line.starts_with("### 11."))
        .count();
    let current_last_non_empty_line = DOC
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("r53 review doc should contain non-empty lines");
    let current_fnv = fnv1a64(DOC.as_bytes());

    assert_eq!(current_line_count, expected_line_count);
    assert_eq!(
        current_appendix_section_count,
        expected_appendix_section_count
    );
    assert_eq!(current_last_non_empty_line, expected_last_non_empty_line);
    assert_eq!(current_fnv, expected_fnv);
}

#[test]
fn regression_r54_plus_review_docs_enforce_post_publication_moratorium() {
    let policy_path = repo_root()
        .join("docs")
        .join("review")
        .join("post-publication-moratorium.policy");
    let policy_doc = fs::read_to_string(&policy_path)
        .unwrap_or_else(|_| panic!("moratorium policy file missing: {}", policy_path.display()));
    let policy = parse_key_value_lines(&policy_doc);

    assert_eq!(
        parse_marker_value(&policy, "review_post_publication_moratorium_schema_version"),
        "kamn.review.post-publication-moratorium.v1"
    );
    let effective_release_min = parse_marker_usize(
        &policy,
        "review_post_publication_moratorium_effective_release_min",
    ) as u32;
    let disallowed_heading_substring = parse_marker_value(
        &policy,
        "review_post_publication_moratorium_disallowed_heading_substring",
    );
    let disallowed_marker_substring = parse_marker_value(
        &policy,
        "review_post_publication_moratorium_disallowed_marker_substring",
    );

    for review_doc_path in tracked_review_docs() {
        let relative = review_doc_path
            .strip_prefix(repo_root())
            .expect("review doc should be under repo root")
            .to_string_lossy()
            .to_string();
        let Some(release) = parse_release_from_review_path(&relative) else {
            continue;
        };
        if release < effective_release_min {
            continue;
        }

        let doc = fs::read_to_string(&review_doc_path)
            .unwrap_or_else(|_| panic!("review doc should be readable: {}", relative));
        for (index, raw_line) in doc.lines().enumerate() {
            let line_no = index + 1;
            let trimmed = raw_line.trim();
            if trimmed.starts_with("### ") {
                assert!(
                    !trimmed.contains(disallowed_heading_substring),
                    "post-publication heading forbidden in {}:{}: {}",
                    relative,
                    line_no,
                    trimmed
                );
            }
            if let Some(marker_line) = trimmed.strip_prefix("- ") {
                if let Some((key, _value)) = marker_line.split_once('=') {
                    assert!(
                        !key.contains(disallowed_marker_substring),
                        "post-publication marker forbidden in {}:{}: {}",
                        relative,
                        line_no,
                        key
                    );
                }
            }
        }
    }
}

#[test]
fn regression_r54_plus_review_docs_enforce_governance_remediation_budget_policy() {
    let policy_path = repo_root()
        .join("docs")
        .join("review")
        .join("governance-remediation-budget.policy");
    let policy_doc = fs::read_to_string(&policy_path).unwrap_or_else(|_| {
        panic!(
            "governance remediation budget policy missing: {}",
            policy_path.display()
        )
    });
    let policy = parse_key_value_lines(&policy_doc);

    assert_eq!(
        parse_marker_value(
            &policy,
            "review_governance_remediation_budget_policy_schema_version"
        ),
        "kamn.review.governance-remediation-budget-policy.v1"
    );
    let effective_release_min = parse_marker_usize(
        &policy,
        "review_governance_remediation_budget_effective_release_min",
    ) as u32;
    let expected_marker_schema = parse_marker_value(
        &policy,
        "review_governance_remediation_budget_marker_schema_version",
    );
    let policy_budget_max = parse_marker_f64(
        &policy,
        "review_governance_remediation_budget_max_commits_per_item",
    );
    let status_within = parse_marker_value(
        &policy,
        "review_governance_remediation_budget_status_within",
    );
    let status_over =
        parse_marker_value(&policy, "review_governance_remediation_budget_status_over");

    for review_doc_path in tracked_review_docs() {
        let relative = review_doc_path
            .strip_prefix(repo_root())
            .expect("review doc should be under repo root")
            .to_string_lossy()
            .to_string();
        let Some(release) = parse_release_from_review_path(&relative) else {
            continue;
        };
        if release < effective_release_min {
            continue;
        }

        let doc = fs::read_to_string(&review_doc_path)
            .unwrap_or_else(|_| panic!("review doc should be readable: {}", relative));
        let markers = parse_marker_lines(&doc);
        let key = |suffix: &str| format!("r{release}_review_governance_remediation_{suffix}");

        let marker_schema = parse_marker_value(&markers, &key("budget_schema_version"));
        assert_eq!(marker_schema, expected_marker_schema);

        let item_count = parse_marker_usize(&markers, &key("item_count"));
        let commit_count = parse_marker_usize(&markers, &key("commit_count"));
        let commits_per_item = parse_marker_f64(&markers, &key("commits_per_item"));
        let budget_max = parse_marker_f64(&markers, &key("budget_max_commits_per_item"));
        let budget_status = parse_marker_value(&markers, &key("budget_status"));

        assert!(
            (budget_max - policy_budget_max).abs() <= 0.001,
            "policy budget max mismatch for {}",
            relative
        );

        let computed_commits_per_item = if item_count == 0 {
            0.0
        } else {
            commit_count as f64 / item_count as f64
        };
        assert!(
            (computed_commits_per_item - commits_per_item).abs() <= 0.01,
            "commits-per-item marker mismatch for {}",
            relative
        );

        let expected_status = if commits_per_item <= policy_budget_max + 0.001 {
            status_within
        } else {
            status_over
        };
        assert_eq!(
            budget_status, expected_status,
            "budget status mismatch for {}",
            relative
        );
    }
}

#[test]
fn regression_r54_review_unresolved_item_closure_markers_are_consistent() {
    let markers = parse_marker_lines(DOC_R54);

    assert_eq!(
        parse_marker_value(&markers, "r54_review_unresolved_closure_schema_version"),
        "kamn.review.unresolved-item-closure.v1"
    );

    let unresolved_total = parse_marker_usize(&markers, "r54_review_unresolved_total_item_count");
    let unresolved_resolved =
        parse_marker_usize(&markers, "r54_review_unresolved_resolved_item_count");
    assert_eq!(unresolved_total, 6);
    assert_eq!(unresolved_total, unresolved_resolved);
    assert_eq!(
        parse_marker_value(&markers, "r54_review_unresolved_closure_status"),
        "all_resolved"
    );

    assert_eq!(
        parse_marker_value(&markers, "r54_review_unresolved_marker_inflation_status",),
        "resolved_via_moratorium_contract"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r54_review_unresolved_governance_commit_dominance_status",
        ),
        "resolved_via_governance_budget_contract"
    );
    assert_eq!(
        parse_marker_value(&markers, "r54_review_unresolved_branch_growth_status"),
        "resolved_via_branch_budget_contract"
    );
    assert_eq!(
        parse_marker_value(&markers, "r54_review_unresolved_doc_contract_growth_status"),
        "resolved_via_non_regression_cap"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r54_review_unresolved_kamn_core_module_stagnation_status",
        ),
        "resolved_via_activation_contract"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r54_review_unresolved_spec_hygiene_contamination_status",
        ),
        "resolved_via_tracked_only_spec_count"
    );

    let branch_snapshot = parse_marker_usize(&markers, "r54_review_branch_growth_snapshot_count");
    let branch_target =
        parse_marker_usize(&markers, "r54_review_branch_growth_target_max_next_release");
    let branch_cleanup = parse_marker_usize(&markers, "r54_review_branch_growth_required_cleanup");
    assert!(branch_target < branch_snapshot);
    assert_eq!(
        branch_snapshot.saturating_sub(branch_target),
        branch_cleanup
    );
    assert_eq!(
        parse_marker_value(&markers, "r54_review_branch_growth_budget_status"),
        "active_cleanup_required"
    );

    let doc_contract_snapshot =
        parse_marker_usize(&markers, "r54_review_doc_contract_snapshot_test_file_count");
    let doc_contract_max = parse_marker_usize(
        &markers,
        "r54_review_doc_contract_non_regression_max_test_file_count",
    );
    assert_eq!(doc_contract_snapshot, doc_contract_max);
    assert!(doc_contract_test_file_count() <= doc_contract_max);
    assert_eq!(
        parse_marker_value(&markers, "r54_review_doc_contract_growth_resolution_status"),
        "cap_locked_no_new_file"
    );

    let module_snapshot =
        parse_marker_usize(&markers, "r54_review_kamn_core_module_snapshot_count");
    let module_target_min = parse_marker_usize(
        &markers,
        "r54_review_kamn_core_module_target_new_modules_next_release_min",
    );
    assert!(module_snapshot > 0);
    assert!(module_target_min >= 1);
    assert_eq!(
        parse_marker_value(&markers, "r54_review_kamn_core_module_activation_status"),
        "planned_for_r55"
    );

    assert_eq!(
        parse_marker_value(&markers, "r54_review_spec_hygiene_fix_schema_version"),
        "kamn.review.spec-hygiene-tracked-only-count.v1"
    );
    assert_eq!(
        parse_marker_value(&markers, "r54_review_spec_hygiene_fix_status"),
        "implemented"
    );
    assert!(parse_marker_usize(&markers, "r54_review_spec_hygiene_fix_issue") > 0);

    let disallowed_heading_count = DOC_R54
        .lines()
        .filter(|line| line.trim().starts_with("### "))
        .filter(|line| line.contains("Post-Publication"))
        .count();
    assert_eq!(disallowed_heading_count, 0);
}
