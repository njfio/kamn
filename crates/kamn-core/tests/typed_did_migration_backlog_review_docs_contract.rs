use std::collections::BTreeMap;

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r43.md");
const REQUIRED_SCHEMA_VERSION: &str = "kamn.typed-did-migration.inventory.v1";

fn parse_marker_lines(doc: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in doc.lines() {
        let trimmed = line.trim();
        let candidate = trimmed
            .strip_prefix("- ")
            .unwrap_or(trimmed)
            .trim_matches('`');
        let Some((key, value)) = candidate.split_once('=') else {
            continue;
        };
        let key = key.trim().trim_matches('`');
        if key.is_empty() {
            continue;
        }
        map.insert(key.to_owned(), value.trim().trim_matches('`').to_owned());
    }
    map
}

fn parse_issue_list(value: &str) -> Vec<&str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect()
}

#[test]
fn functional_typed_did_review_markers_present() {
    let markers = parse_marker_lines(DOC);

    for key in [
        "typed_did_migration_inventory_schema_version",
        "typed_did_migration_inventory_non_data_layer_module_count",
        "typed_did_migration_inventory_non_data_layer_did_string_callsite_count",
        "typed_did_migration_backlog_issue_ids",
        "typed_did_migration_wave_issue_ids",
        "typed_did_migration_wave_a_scope",
        "typed_did_migration_wave_b_scope",
        "typed_did_migration_wave_c_scope",
    ] {
        assert!(
            markers.contains_key(key),
            "R43 review doc missing required typed-DID marker `{key}`"
        );
    }

    assert_eq!(
        markers
            .get("typed_did_migration_inventory_schema_version")
            .map(String::as_str),
        Some(REQUIRED_SCHEMA_VERSION),
        "typed-DID marker schema version must remain stable"
    );
}

#[test]
fn integration_typed_did_issue_markers_use_issue_id_format() {
    let markers = parse_marker_lines(DOC);

    let backlog = markers
        .get("typed_did_migration_backlog_issue_ids")
        .expect("typed DID backlog issue marker must exist");
    let wave = markers
        .get("typed_did_migration_wave_issue_ids")
        .expect("typed DID wave issue marker must exist");

    let backlog_issues = parse_issue_list(backlog);
    let wave_issues = parse_issue_list(wave);

    assert!(
        !backlog_issues.is_empty(),
        "typed DID backlog issue marker must list at least one issue"
    );
    assert!(
        !wave_issues.is_empty(),
        "typed DID wave issue marker must list at least one issue"
    );

    for issue in backlog_issues.iter().chain(wave_issues.iter()) {
        assert!(
            issue.starts_with('#')
                && issue.len() > 1
                && issue.chars().skip(1).all(|ch| ch.is_ascii_digit()),
            "typed DID issue markers must use #<id> format: {issue}"
        );
    }

    for expected in ["#5228", "#5229", "#5230"] {
        assert!(
            wave_issues.contains(&expected),
            "typed DID wave issue marker must include {expected}"
        );
    }
}

#[test]
fn regression_typed_did_inventory_counts_are_parseable() {
    let markers = parse_marker_lines(DOC);

    let module_count: usize = markers
        .get("typed_did_migration_inventory_non_data_layer_module_count")
        .expect("module-count marker must exist")
        .parse()
        .expect("module-count marker must be parseable integer");
    let callsite_count: usize = markers
        .get("typed_did_migration_inventory_non_data_layer_did_string_callsite_count")
        .expect("callsite-count marker must exist")
        .parse()
        .expect("callsite-count marker must be parseable integer");

    assert!(
        module_count > 0,
        "typed DID inventory module count must be positive"
    );
    assert!(
        callsite_count >= module_count,
        "typed DID inventory callsite count should be >= module count"
    );
}
