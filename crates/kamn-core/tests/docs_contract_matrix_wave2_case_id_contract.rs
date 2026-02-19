use std::collections::BTreeSet;

fn repo_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

fn harness_source() -> String {
    std::fs::read_to_string(repo_path("tests/docs_contract_matrix_wave2_harness.rs"))
        .expect("docs_contract_matrix_wave2_harness.rs should exist")
}

fn parse_case_ids(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("case_id: \"")
                .and_then(|rest| rest.strip_suffix("\","))
                .map(ToOwned::to_owned)
        })
        .collect()
}

#[test]
fn regression_wave2_case_ids_are_unique_and_stable() {
    // Regression: #5217
    let case_ids = parse_case_ids(harness_source().as_str());
    assert_eq!(case_ids.len(), 13, "wave2 case-id inventory drifted");

    let unique_count = case_ids.iter().collect::<BTreeSet<_>>().len();
    assert_eq!(
        unique_count,
        case_ids.len(),
        "wave2 case IDs must remain unique"
    );
    assert!(
        case_ids
            .iter()
            .all(|case_id| case_id.ends_with("_contract_markers")),
        "wave2 case IDs must retain the *_contract_markers naming contract"
    );
}
