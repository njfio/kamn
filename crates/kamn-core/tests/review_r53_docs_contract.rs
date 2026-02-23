use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r53.md");
const DOC_R54: &str = include_str!("../../../docs/review/gaps-and-issues-r54.md");
const DOC_R55: &str = include_str!("../../../docs/review/gaps-and-issues-r55.md");
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

fn workspace_contract_test_file_count() -> usize {
    let mut count = 0usize;
    let crates_dir = repo_root().join("crates");
    let crate_entries = fs::read_dir(&crates_dir)
        .unwrap_or_else(|_| panic!("crates dir should be readable: {}", crates_dir.display()));
    for crate_entry in crate_entries.filter_map(|entry| entry.ok()) {
        let tests_dir = crate_entry.path().join("tests");
        if !tests_dir.is_dir() {
            continue;
        }
        let test_entries = fs::read_dir(&tests_dir)
            .unwrap_or_else(|_| panic!("tests dir should be readable: {}", tests_dir.display()));
        for test_entry in test_entries.filter_map(|entry| entry.ok()) {
            let path = test_entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.contains("contract") {
                count = count.saturating_add(1);
            }
        }
    }
    count
}

fn collect_rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|_| panic!("source root should be readable: {}", root.display()));
    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn is_test_cfg_attribute(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#[cfg(") && trimmed.contains("test") && !trimmed.contains("not(test)")
}

#[derive(Clone, Copy, Debug, Default)]
struct BraceScanState {
    block_comment_depth: usize,
    string_delimiter: Option<u8>,
    raw_string_hashes: Option<usize>,
    escape_next: bool,
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn starts_char_literal(bytes: &[u8], index: usize) -> bool {
    if index >= bytes.len() || bytes[index] != b'\'' {
        return false;
    }
    if index + 1 >= bytes.len() {
        return false;
    }
    let next = bytes[index + 1];
    !(next.is_ascii_alphabetic() || next == b'_')
}

fn parse_raw_string_start(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    if index >= bytes.len() {
        return None;
    }

    let prefix_len = if bytes[index] == b'r' {
        1
    } else if index + 1 < bytes.len() && bytes[index] == b'b' && bytes[index + 1] == b'r' {
        2
    } else {
        return None;
    };

    if index > 0 && is_ident_byte(bytes[index - 1]) {
        return None;
    }

    let mut cursor = index + prefix_len;
    let mut hash_count = 0usize;
    while cursor < bytes.len() && bytes[cursor] == b'#' {
        hash_count += 1;
        cursor += 1;
    }

    if cursor < bytes.len() && bytes[cursor] == b'"' {
        Some((hash_count, cursor + 1))
    } else {
        None
    }
}

fn line_brace_counts(line: &str, state: &mut BraceScanState) -> (i64, i64) {
    let bytes = line.as_bytes();
    let mut index = 0usize;
    let mut open_count = 0_i64;
    let mut close_count = 0_i64;

    while index < bytes.len() {
        if let Some(hash_count) = state.raw_string_hashes {
            if bytes[index] == b'"' {
                let mut cursor = index + 1;
                let mut matched = true;
                for _ in 0..hash_count {
                    if cursor >= bytes.len() || bytes[cursor] != b'#' {
                        matched = false;
                        break;
                    }
                    cursor += 1;
                }
                if matched {
                    state.raw_string_hashes = None;
                    index = cursor;
                    continue;
                }
            }
            index += 1;
            continue;
        }

        if let Some(delimiter) = state.string_delimiter {
            if state.escape_next {
                state.escape_next = false;
                index += 1;
                continue;
            }
            if bytes[index] == b'\\' {
                state.escape_next = true;
                index += 1;
                continue;
            }
            if bytes[index] == delimiter {
                state.string_delimiter = None;
            }
            index += 1;
            continue;
        }

        if state.block_comment_depth > 0 {
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
                state.block_comment_depth += 1;
                index += 2;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                state.block_comment_depth = state.block_comment_depth.saturating_sub(1);
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }

        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'/' {
            break;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            state.block_comment_depth += 1;
            index += 2;
            continue;
        }

        if let Some((hash_count, next_index)) = parse_raw_string_start(bytes, index) {
            state.raw_string_hashes = Some(hash_count);
            state.escape_next = false;
            index = next_index;
            continue;
        }

        if index + 1 < bytes.len()
            && bytes[index] == b'b'
            && bytes[index + 1] == b'"'
            && (index == 0 || !is_ident_byte(bytes[index - 1]))
        {
            state.string_delimiter = Some(b'"');
            state.escape_next = false;
            index += 2;
            continue;
        }
        if index + 1 < bytes.len()
            && bytes[index] == b'b'
            && bytes[index + 1] == b'\''
            && (index == 0 || !is_ident_byte(bytes[index - 1]))
        {
            state.string_delimiter = Some(b'\'');
            state.escape_next = false;
            index += 2;
            continue;
        }
        if bytes[index] == b'"' {
            state.string_delimiter = Some(bytes[index]);
            state.escape_next = false;
            index += 1;
            continue;
        }
        if bytes[index] == b'\'' && starts_char_literal(bytes, index) {
            state.string_delimiter = Some(bytes[index]);
            state.escape_next = false;
            index += 1;
            continue;
        }

        if bytes[index] == b'{' {
            open_count += 1;
        } else if bytes[index] == b'}' {
            close_count += 1;
        }
        index += 1;
    }

    (open_count, close_count)
}

fn skip_cfg_test_item(lines: &[&str], mut index: usize) -> usize {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }

    while index < lines.len() && lines[index].trim_start().starts_with("#[") {
        index += 1;
    }

    if index >= lines.len() {
        return index;
    }

    let mut scan_state = BraceScanState::default();
    let mut brace_depth = 0_i64;
    let mut saw_open_brace = false;
    while index < lines.len() {
        let line = lines[index];
        let (open_count, close_count) = line_brace_counts(line, &mut scan_state);
        if open_count > 0 {
            saw_open_brace = true;
        }
        brace_depth += open_count - close_count;
        index += 1;

        if saw_open_brace {
            if brace_depth <= 0 {
                return index;
            }
            continue;
        }

        if line.trim_end().ends_with(';') {
            return index;
        }
    }
    index
}

fn production_source_without_cfg_test_items(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut retained = Vec::with_capacity(lines.len());
    let mut index = 0usize;
    while index < lines.len() {
        if is_test_cfg_attribute(lines[index]) {
            index = skip_cfg_test_item(&lines, index + 1);
            continue;
        }
        retained.push(lines[index]);
        index += 1;
    }
    retained.join("\n")
}

fn production_expect_inventory_count() -> usize {
    let mut source_files = Vec::new();
    let crates_dir = repo_root().join("crates");
    let crate_entries = fs::read_dir(&crates_dir)
        .unwrap_or_else(|_| panic!("crates dir should be readable: {}", crates_dir.display()));
    for crate_entry in crate_entries.filter_map(|entry| entry.ok()) {
        let src_dir = crate_entry.path().join("src");
        if src_dir.is_dir() {
            collect_rust_files(&src_dir, &mut source_files);
        }
    }

    source_files
        .iter()
        .filter(|path| {
            let path_text = path.to_string_lossy();
            !path_text.contains("/main_tests/")
                && path.file_name().and_then(|name| name.to_str()) != Some("main_tests.rs")
                && !path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .ends_with("_tests.rs")
                && !path_text.contains("/runtime_tests")
                && !path_text.contains("/cli_tests")
                && !path_text.contains("/test_utils/")
                && !path_text.contains("/tests/")
        })
        .map(|path| {
            let source = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("source file should be readable: {}", path.display()));
            production_source_without_cfg_test_items(&source)
                .lines()
                .filter(|line| line.contains(".expect("))
                .count()
        })
        .sum()
}

#[test]
fn unit_cfg_test_item_skip_ignores_braces_inside_strings() {
    let source = r#"
fn production_before() {}

#[cfg(test)]
mod tests {
    fn brace_confuser_literal() -> &'static str {
        "}}}}"
    }

    #[test]
    fn test_only_expect() {
        let literal = brace_confuser_literal();
        let value = Some(literal.len()).expect("test-only expect");
        assert!(value > 0);
    }
}

fn production_after() -> usize { 42 }
"#;
    let stripped = production_source_without_cfg_test_items(source);
    assert!(stripped.contains("fn production_before() {}"));
    assert!(stripped.contains("fn production_after() -> usize { 42 }"));
    assert!(!stripped.contains("mod tests"));
    assert!(!stripped.contains("test-only expect"));
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
    let r55_markers = parse_marker_lines(DOC_R55);
    let spec_delta_base_cap = parse_marker_usize(
        &r55_markers,
        "r55_review_spec_volume_non_regression_base_cap",
    );
    let spec_delta_allowance = parse_marker_usize(
        &r55_markers,
        "r55_review_spec_volume_non_regression_delta_allowance",
    );
    let spec_effective_cap = parse_marker_usize(
        &r55_markers,
        "r55_review_spec_volume_non_regression_effective_cap",
    );
    assert_eq!(spec_delta_base_cap, non_regression_spec_dir_max);
    assert_eq!(
        spec_delta_base_cap.saturating_add(spec_delta_allowance),
        spec_effective_cap
    );
    assert!(top_level_spec_dir_count() <= spec_effective_cap);

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

#[test]
fn regression_review_docs_tracked_spec_dir_count_ignores_untracked_top_level_specs_dirs() {
    let specs_dir = repo_root().join("specs");
    let temp_dir = specs_dir.join(format!(
        "zz-untracked-review-r53-spec-dir-contamination-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let baseline = top_level_spec_dir_count();
    fs::create_dir_all(&temp_dir).unwrap_or_else(|_| {
        panic!(
            "failed creating temp specs dir for tracked-only regression: {}",
            temp_dir.display()
        )
    });
    let observed = top_level_spec_dir_count();
    let _ = fs::remove_dir_all(&temp_dir);

    assert_eq!(
        observed, baseline,
        "tracked spec-dir counting must ignore untracked top-level specs directories"
    );
}

#[test]
fn regression_r55_review_unresolved_item_closure_markers_are_consistent() {
    let markers = parse_marker_lines(DOC_R55);

    assert_eq!(
        parse_marker_value(&markers, "r55_review_unresolved_closure_schema_version"),
        "kamn.review.unresolved-item-closure.v1"
    );

    let unresolved_total = parse_marker_usize(&markers, "r55_review_unresolved_total_item_count");
    let unresolved_resolved =
        parse_marker_usize(&markers, "r55_review_unresolved_resolved_item_count");
    assert_eq!(unresolved_total, 5);
    assert_eq!(unresolved_total, unresolved_resolved);
    assert_eq!(
        parse_marker_value(&markers, "r55_review_unresolved_closure_status"),
        "all_resolved"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_unresolved_governance_structural_coupling_status",
        ),
        "resolved_via_structural_coupling_budget_contract"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_unresolved_doc_contract_cap_breach_status",
        ),
        "resolved_via_workspace_cap_enforcement_contract"
    );
    assert_eq!(
        parse_marker_value(&markers, "r55_review_unresolved_node_kolme_freeze_status"),
        "resolved_via_runtime_scope_surface_activation"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_unresolved_production_expect_audit_status",
        ),
        "resolved_via_deterministic_expect_inventory_contract"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_unresolved_spec_hygiene_contamination_status",
        ),
        "resolved_via_tracked_only_count_enforcement"
    );

    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_governance_structural_coupling_schema_version",
        ),
        "kamn.review.governance-structural-coupling-budget.v1"
    );
    let non_merge_commit_count = parse_marker_usize(
        &markers,
        "r55_review_governance_structural_coupling_non_merge_commit_count",
    );
    let governance_commit_count = parse_marker_usize(
        &markers,
        "r55_review_governance_structural_coupling_governance_commit_count",
    );
    let governance_commit_ratio = parse_marker_f64(
        &markers,
        "r55_review_governance_structural_coupling_governance_commit_ratio",
    );
    let governance_target_ratio = parse_marker_f64(
        &markers,
        "r55_review_governance_structural_coupling_target_ratio_max_next_release",
    );
    assert!(non_merge_commit_count > 0);
    assert!(governance_commit_count <= non_merge_commit_count);
    assert!(
        (governance_commit_ratio - governance_commit_count as f64 / non_merge_commit_count as f64)
            .abs()
            <= 0.01
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_governance_structural_coupling_budget_status",
        ),
        if governance_commit_ratio <= governance_target_ratio + 0.001 {
            "within_target"
        } else {
            "active_reduction_contract"
        }
    );
    assert!(
        parse_marker_usize(
            &markers,
            "r55_review_governance_structural_coupling_mitigation_issue",
        ) > 0
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_governance_remediation_budget_schema_version",
        ),
        "kamn.review.governance-remediation-budget.v1"
    );
    let remediation_item_count =
        parse_marker_usize(&markers, "r55_review_governance_remediation_item_count");
    let remediation_commit_count =
        parse_marker_usize(&markers, "r55_review_governance_remediation_commit_count");
    let remediation_commits_per_item = parse_marker_f64(
        &markers,
        "r55_review_governance_remediation_commits_per_item",
    );
    let remediation_budget_max = parse_marker_f64(
        &markers,
        "r55_review_governance_remediation_budget_max_commits_per_item",
    );
    let computed_remediation_ratio = if remediation_item_count == 0 {
        0.0
    } else {
        remediation_commit_count as f64 / remediation_item_count as f64
    };
    assert!((computed_remediation_ratio - remediation_commits_per_item).abs() <= 0.01);
    assert_eq!(
        parse_marker_value(&markers, "r55_review_governance_remediation_budget_status"),
        if remediation_commits_per_item <= remediation_budget_max + 0.001 {
            "within_budget"
        } else {
            "over_budget"
        }
    );

    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_spec_volume_non_regression_delta_schema_version",
        ),
        "kamn.review.spec-volume-non-regression-delta-allowance.v1"
    );
    let spec_base_cap =
        parse_marker_usize(&markers, "r55_review_spec_volume_non_regression_base_cap");
    let spec_delta_allowance = parse_marker_usize(
        &markers,
        "r55_review_spec_volume_non_regression_delta_allowance",
    );
    let spec_effective_cap = parse_marker_usize(
        &markers,
        "r55_review_spec_volume_non_regression_effective_cap",
    );
    assert_eq!(
        spec_base_cap.saturating_add(spec_delta_allowance),
        spec_effective_cap
    );
    assert_eq!(
        parse_marker_value(&markers, "r55_review_spec_volume_non_regression_status"),
        if top_level_spec_dir_count() <= spec_effective_cap {
            "within_effective_cap"
        } else {
            "breached_effective_cap"
        }
    );

    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_workspace_contract_file_cap_schema_version",
        ),
        "kamn.review.workspace-contract-file-cap.v1"
    );
    let contract_snapshot = parse_marker_usize(
        &markers,
        "r55_review_workspace_contract_file_count_snapshot",
    );
    let contract_max = parse_marker_usize(
        &markers,
        "r55_review_workspace_contract_file_non_regression_max",
    );
    let contract_r54_lock =
        parse_marker_usize(&markers, "r55_review_workspace_contract_file_r54_lock");
    let contract_delta = parse_marker_usize(
        &markers,
        "r55_review_workspace_contract_file_breach_delta_vs_r54_lock",
    );
    assert_eq!(
        parse_marker_value(&markers, "r55_review_workspace_contract_file_count_formula",),
        "count(files in crates/*/tests/*.rs where filename contains 'contract')"
    );
    assert_eq!(workspace_contract_test_file_count(), contract_snapshot);
    assert!(workspace_contract_test_file_count() <= contract_max);
    assert_eq!(
        contract_snapshot.saturating_sub(contract_r54_lock),
        contract_delta
    );
    assert_eq!(
        parse_marker_value(&markers, "r55_review_workspace_contract_file_cap_status"),
        if contract_snapshot <= contract_r54_lock {
            "within_r54_lock"
        } else {
            "regressed_with_waiver"
        }
    );
    if contract_snapshot > contract_r54_lock {
        assert!(
            parse_marker_usize(
                &markers,
                "r55_review_workspace_contract_file_cap_mitigation_issue",
            ) > 0
        );
    }

    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_production_expect_inventory_schema_version",
        ),
        "kamn.review.production-expect-inventory.v1"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_production_expect_inventory_count_formula",
        ),
        "count(lines containing '.expect(' after stripping #[cfg(test)]-guarded items in crates/*/src/**/*.rs excluding /main_tests/, /runtime_tests, /cli_tests, /test_utils/, /tests/, main_tests.rs, and *_tests.rs)"
    );
    let reported_r55 = parse_marker_usize(
        &markers,
        "r55_review_production_expect_inventory_reported_count_r55",
    );
    let expect_snapshot = parse_marker_usize(
        &markers,
        "r55_review_production_expect_inventory_snapshot_count",
    );
    let expect_delta = parse_marker_usize(
        &markers,
        "r55_review_production_expect_inventory_delta_vs_r55",
    );
    let expect_target = parse_marker_usize(
        &markers,
        "r55_review_production_expect_inventory_target_max_next_release",
    );
    assert_eq!(production_expect_inventory_count(), expect_snapshot);
    assert_eq!(reported_r55.saturating_sub(expect_snapshot), expect_delta);
    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_production_expect_inventory_policy_status",
        ),
        if expect_snapshot <= expect_target {
            "within_target"
        } else {
            "active_reduction_contract"
        }
    );

    assert_eq!(
        parse_marker_value(&markers, "r55_review_spec_hygiene_fix_schema_version"),
        "kamn.review.spec-hygiene-tracked-only-count.v2"
    );
    assert_eq!(
        parse_marker_value(&markers, "r55_review_spec_hygiene_fix_status"),
        "implemented"
    );
    assert!(parse_marker_usize(&markers, "r55_review_spec_hygiene_fix_issue") > 0);

    assert_eq!(
        parse_marker_value(
            &markers,
            "r55_review_node_kolme_freeze_resolution_schema_version",
        ),
        "kamn.review.runtime-surface-reactivation.v1"
    );
    assert!(
        parse_marker_usize(
            &markers,
            "r55_review_node_freeze_reviews_since_last_real_change_before_resolution",
        ) >= 6
    );
    assert!(
        parse_marker_usize(
            &markers,
            "r55_review_kolme_freeze_reviews_since_last_real_change_before_resolution",
        ) >= 6
    );
    assert_eq!(
        parse_marker_value(&markers, "r55_review_node_kolme_freeze_resolution_status"),
        "implemented"
    );
    assert!(parse_marker_usize(&markers, "r55_review_node_kolme_freeze_resolution_issue") > 0);
}
