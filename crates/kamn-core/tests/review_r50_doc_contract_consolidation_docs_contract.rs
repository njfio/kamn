use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

fn parse_bullet_markers(doc: &str) -> BTreeMap<String, String> {
    let mut markers = BTreeMap::new();
    for raw_line in doc.lines() {
        let mut trimmed = raw_line.trim();
        if let Some(value) = trimmed.strip_prefix("- ") {
            trimmed = value.trim();
        }
        if let Some(value) = trimmed
            .strip_prefix('`')
            .and_then(|value| value.strip_suffix('`'))
        {
            trimmed = value.trim();
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        markers.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    markers
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

fn current_doc_contract_test_file_count() -> usize {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    fs::read_dir(tests_dir)
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
fn functional_r50_doc_contract_non_regression_schema_is_documented() {
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_doc_contract_non_regression_schema_version=kamn.review.doc-contract-non-regression-ratchet.v1"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_doc_contract_non_regression_baseline_test_file_count=<integer>"
    ));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_doc_contract_non_regression_max_test_file_count=<integer>"));
}

#[test]
fn integration_r50_doc_contract_non_regression_markers_are_consistent() {
    let markers = parse_bullet_markers(DOC);
    assert_eq!(
        parse_marker_value(
            &markers,
            "r50_review_doc_contract_non_regression_schema_version"
        ),
        "kamn.review.doc-contract-non-regression-ratchet.v1"
    );
    let baseline = parse_marker_usize(
        &markers,
        "r50_review_doc_contract_non_regression_baseline_test_file_count",
    );
    let max = parse_marker_usize(
        &markers,
        "r50_review_doc_contract_non_regression_max_test_file_count",
    );
    assert!(baseline <= max);
    assert!(
        parse_marker_value(
            &markers,
            "r50_review_doc_contract_non_regression_count_formula"
        )
        .contains("rg --files crates/kamn-core/tests"),
        "count formula must remain deterministic and repository-local"
    );
}

#[test]
fn regression_r50_doc_contract_non_regression_cap_is_not_breached() {
    let markers = parse_bullet_markers(DOC);
    let cap = parse_marker_usize(
        &markers,
        "r50_review_doc_contract_non_regression_max_test_file_count",
    );
    let current = current_doc_contract_test_file_count();
    assert!(
        current <= cap,
        "doc contract test count {} exceeds cap {}",
        current,
        cap
    );
}
