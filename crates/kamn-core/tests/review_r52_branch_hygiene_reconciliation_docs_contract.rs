const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r52.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

fn parse_marker_value(marker_key: &str) -> String {
    let needle = format!("{marker_key}=");
    let line = DOC
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

fn parse_marker_usize(marker_key: &str) -> usize {
    let value = parse_marker_value(marker_key);
    value
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be an unsigned integer: {value}"))
}

#[test]
fn functional_r52_branch_hygiene_reconciliation_markers_present() {
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_post_publication_branch_cleanup_schema_version"));
    assert!(REVIEW_MARKER_README.contains("kamn.review.branch-hygiene-post-publication-cleanup.v1"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_remote_head_count_pre_cleanup=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_remote_head_count_deleted=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_remote_head_count_post_cleanup=<integer>"));

    assert!(DOC.contains(
        "r52_review_post_publication_branch_cleanup_schema_version=kamn.review.branch-hygiene-post-publication-cleanup.v1"
    ));
    assert!(DOC.contains("r52_review_branch_remote_head_count_baseline_snapshot=67"));
    assert!(DOC.contains("r52_review_branch_remote_head_count_pre_cleanup="));
    assert!(DOC.contains("r52_review_branch_remote_head_count_deleted="));
    assert!(DOC.contains("r52_review_branch_remote_head_count_post_cleanup="));
}

#[test]
fn integration_r52_branch_hygiene_reconciliation_counts_are_consistent() {
    let baseline = parse_marker_usize("r52_review_branch_remote_head_count_baseline_snapshot");
    let pre = parse_marker_usize("r52_review_branch_remote_head_count_pre_cleanup");
    let deleted = parse_marker_usize("r52_review_branch_remote_head_count_deleted");
    let post = parse_marker_usize("r52_review_branch_remote_head_count_post_cleanup");

    assert!(
        pre >= post,
        "post-cleanup branch count should not exceed pre-cleanup count"
    );
    assert_eq!(
        pre.saturating_sub(post),
        deleted,
        "deleted count should match pre/post delta"
    );
    assert!(
        post <= baseline,
        "post-cleanup count should not exceed R52 baseline snapshot"
    );
}

#[test]
fn functional_r52_post_publication_quality_gate_reconciliation_markers_present() {
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_post_publication_quality_gate_reconciliation_schema_version"));
    assert!(REVIEW_MARKER_README
        .contains("kamn.review.quality-gate-post-publication-reconciliation.v1"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_workspace_quality_gate_status_post_publication=<pass|fail>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_cli_compile_status_post_publication=<resolved|unresolved>"));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_activity_ratio_marker_parse_status_post_publication=<resolved|unresolved>"
    ));

    assert!(DOC.contains(
        "r52_review_post_publication_quality_gate_reconciliation_schema_version=kamn.review.quality-gate-post-publication-reconciliation.v1"
    ));
    assert!(DOC.contains("r52_review_workspace_quality_gate_status_post_publication=pass"));
    assert!(DOC.contains("r52_review_cli_compile_status_post_publication=resolved"));
    assert!(DOC.contains("r52_review_activity_ratio_marker_parse_status_post_publication=resolved"));
}

#[test]
fn integration_r52_post_publication_quality_gate_reconciliation_markers_are_consistent() {
    let schema = parse_marker_value(
        "r52_review_post_publication_quality_gate_reconciliation_schema_version",
    );
    let workspace_gate_status =
        parse_marker_value("r52_review_workspace_quality_gate_status_post_publication");
    let cli_compile_status = parse_marker_value("r52_review_cli_compile_status_post_publication");
    let activity_ratio_parse_status =
        parse_marker_value("r52_review_activity_ratio_marker_parse_status_post_publication");
    let workspace_gate_command =
        parse_marker_value("r52_review_workspace_quality_gate_command_post_publication");
    let activity_ratio_command =
        parse_marker_value("r52_review_activity_ratio_marker_parse_command_post_publication");

    assert_eq!(
        schema, "kamn.review.quality-gate-post-publication-reconciliation.v1",
        "schema version should remain fixed"
    );
    assert_eq!(
        workspace_gate_status, "pass",
        "workspace quality gate must remain resolved as pass"
    );
    assert_eq!(
        cli_compile_status, "resolved",
        "CLI compile error post-publication status must remain resolved"
    );
    assert_eq!(
        activity_ratio_parse_status, "resolved",
        "activity-ratio marker parser status must remain resolved"
    );
    assert_eq!(
        workspace_gate_command, "cargo test --workspace --locked --all-features --no-fail-fast",
        "workspace quality-gate evidence command should be deterministic"
    );
    assert_eq!(
        activity_ratio_command,
        "cargo test -p kamn-core --test release_review_activity_ratio_docs_contract",
        "activity-ratio evidence command should be deterministic"
    );
    assert!(
        DOC.contains("**As of:** R52 review, commit `8e0871cc` (2026-02-22)"),
        "as-of snapshot baseline should remain unchanged"
    );
    assert!(
        DOC.contains(
            "**Baseline snapshot:** commit `8e0871cc` | **Rust LOC:** 198,094 | **Tests:** 3,160 passed, 2 failed, 10 ignored (excl kamn-cli — see Section 1.1) | **Shell LOC:** 141,965"
        ),
        "baseline snapshot line should remain unchanged"
    );
}

#[test]
fn functional_r52_post_publication_priority_reconciliation_markers_present() {
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_post_publication_priority_reconciliation_schema_version"));
    assert!(REVIEW_MARKER_README
        .contains("kamn.review.priority-summary-post-publication-reconciliation.v1"));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_priority_critical_cli_compile_status_post_publication=<resolved|unresolved>"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_priority_medium_activity_ratio_marker_status_post_publication=<resolved|unresolved>"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_priority_high_spec_volume_guardrail_status_post_publication=<within_guardrail|breached>"
    ));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_priority_summary_snapshot_preserved=<true|false>"));

    assert!(DOC.contains(
        "r52_review_post_publication_priority_reconciliation_schema_version=kamn.review.priority-summary-post-publication-reconciliation.v1"
    ));
    assert!(
        DOC.contains("r52_review_priority_critical_cli_compile_status_post_publication=resolved")
    );
    assert!(DOC.contains(
        "r52_review_priority_medium_activity_ratio_marker_status_post_publication=resolved"
    ));
    assert!(DOC.contains(
        "r52_review_priority_high_spec_volume_guardrail_status_post_publication=within_guardrail"
    ));
    assert!(DOC.contains("r52_review_priority_summary_snapshot_preserved=true"));
}

#[test]
fn integration_r52_post_publication_priority_reconciliation_markers_are_consistent() {
    let schema =
        parse_marker_value("r52_review_post_publication_priority_reconciliation_schema_version");
    let critical_cli_status =
        parse_marker_value("r52_review_priority_critical_cli_compile_status_post_publication");
    let medium_activity_ratio_status = parse_marker_value(
        "r52_review_priority_medium_activity_ratio_marker_status_post_publication",
    );
    let high_spec_volume_status = parse_marker_value(
        "r52_review_priority_high_spec_volume_guardrail_status_post_publication",
    );
    let snapshot_preserved = parse_marker_value("r52_review_priority_summary_snapshot_preserved");

    let quality_gate_cli_status =
        parse_marker_value("r52_review_cli_compile_status_post_publication");
    let quality_gate_activity_ratio_status =
        parse_marker_value("r52_review_activity_ratio_marker_parse_status_post_publication");
    let guardrail_status =
        parse_marker_value("r52_review_spec_volume_guardrail_post_publication_status");

    assert_eq!(
        schema, "kamn.review.priority-summary-post-publication-reconciliation.v1",
        "schema version should remain fixed"
    );
    assert_eq!(
        critical_cli_status, quality_gate_cli_status,
        "priority critical CLI status should match quality-gate reconciliation status"
    );
    assert_eq!(
        medium_activity_ratio_status, quality_gate_activity_ratio_status,
        "priority medium activity-ratio marker status should match quality-gate reconciliation status"
    );
    assert_eq!(
        high_spec_volume_status, guardrail_status,
        "priority high spec-volume status should match guardrail reconciliation status"
    );
    assert_eq!(
        snapshot_preserved, "true",
        "priority summary snapshot-preservation marker should remain true"
    );

    assert!(
        DOC.contains("| **Critical** | kamn-cli compilation error on main | `command_activation_contract.rs` 15 type errors | Fix test or dispatch return type | **NEW** |"),
        "historical critical priority row should remain unchanged"
    );
    assert!(
        DOC.contains("| **High** | Spec volume (845, 9.2:1) severely exceeds 7.7 guardrail | 845 dirs / 92 modules | Stop new specs until ratio improves | **WORSENED** |"),
        "historical high-priority spec-volume row should remain unchanged"
    );
}

#[test]
fn functional_r52_post_publication_branch_hygiene_status_reconciliation_markers_present() {
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_post_publication_branch_hygiene_status_reconciliation_schema_version"
    ));
    assert!(REVIEW_MARKER_README
        .contains("kamn.review.branch-hygiene-status-post-publication-reconciliation.v1"));
    assert!(
        REVIEW_MARKER_README.contains("r<release>_review_branch_hygiene_snapshot_status=<text>")
    );
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_hygiene_snapshot_branch_count=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_hygiene_post_publication_pre_cleanup_count=<integer>"));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_branch_hygiene_post_publication_post_cleanup_count=<integer>"
    ));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_hygiene_post_publication_status=<text>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_hygiene_snapshot_rows_preserved=<true|false>"));

    assert!(DOC.contains(
        "r52_review_post_publication_branch_hygiene_status_reconciliation_schema_version=kamn.review.branch-hygiene-status-post-publication-reconciliation.v1"
    ));
    assert!(DOC.contains("r52_review_branch_hygiene_snapshot_status=slightly_worsened"));
    assert!(DOC.contains("r52_review_branch_hygiene_snapshot_branch_count=67"));
    assert!(DOC.contains("r52_review_branch_hygiene_post_publication_pre_cleanup_count=61"));
    assert!(DOC.contains("r52_review_branch_hygiene_post_publication_post_cleanup_count=61"));
    assert!(
        DOC.contains("r52_review_branch_hygiene_post_publication_status=improved_against_snapshot")
    );
    assert!(DOC.contains("r52_review_branch_hygiene_snapshot_rows_preserved=true"));
}

#[test]
fn integration_r52_post_publication_branch_hygiene_status_reconciliation_markers_are_consistent() {
    let schema = parse_marker_value(
        "r52_review_post_publication_branch_hygiene_status_reconciliation_schema_version",
    );
    let snapshot_status = parse_marker_value("r52_review_branch_hygiene_snapshot_status");
    let snapshot_branch_count =
        parse_marker_usize("r52_review_branch_hygiene_snapshot_branch_count");
    let post_pre_count =
        parse_marker_usize("r52_review_branch_hygiene_post_publication_pre_cleanup_count");
    let post_post_count =
        parse_marker_usize("r52_review_branch_hygiene_post_publication_post_cleanup_count");
    let post_status = parse_marker_value("r52_review_branch_hygiene_post_publication_status");
    let snapshot_rows_preserved =
        parse_marker_value("r52_review_branch_hygiene_snapshot_rows_preserved");

    let cleanup_baseline =
        parse_marker_usize("r52_review_branch_remote_head_count_baseline_snapshot");
    let cleanup_pre = parse_marker_usize("r52_review_branch_remote_head_count_pre_cleanup");
    let cleanup_post = parse_marker_usize("r52_review_branch_remote_head_count_post_cleanup");

    assert_eq!(
        schema, "kamn.review.branch-hygiene-status-post-publication-reconciliation.v1",
        "schema version should remain fixed"
    );
    assert_eq!(
        snapshot_status, "slightly_worsened",
        "snapshot status should remain fixed to baseline wording"
    );
    assert_eq!(
        snapshot_branch_count, cleanup_baseline,
        "status reconciliation snapshot count should match cleanup baseline marker"
    );
    assert_eq!(
        post_pre_count, cleanup_pre,
        "status reconciliation pre-cleanup count should match cleanup marker"
    );
    assert_eq!(
        post_post_count, cleanup_post,
        "status reconciliation post-cleanup count should match cleanup marker"
    );
    assert!(
        post_post_count <= snapshot_branch_count,
        "post-publication branch count should not exceed snapshot baseline count"
    );
    assert_eq!(
        post_status, "improved_against_snapshot",
        "post-publication status should remain improved_against_snapshot"
    );
    assert_eq!(
        snapshot_rows_preserved, "true",
        "snapshot rows preserved marker should remain true"
    );

    assert!(
        DOC.contains("## 6. Branch Hygiene — SLIGHTLY WORSENED"),
        "historical branch-hygiene heading should remain unchanged"
    );
    assert!(
        DOC.contains(
            "| **Medium** | Branches (67, +6) | Trending up | Prune merged branches | **Slightly worsened** |"
        ),
        "historical priority row for branches should remain unchanged"
    );
}
