use std::fs;
use std::path::{Path, PathBuf};

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");
const NON_REGRESSION_COUNT_FORMULA: &str =
    "rg --files crates/kamn-core/tests | rg '_docs\\\\.rs$|docs_contract' | wc -l";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

fn current_doc_contract_test_file_count() -> usize {
    let tests_dir = repo_root().join("crates").join("kamn-core").join("tests");
    fs::read_dir(tests_dir)
        .expect("kamn-core tests directory should be readable")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return false;
            };
            name.ends_with("_docs.rs") || name.contains("docs_contract")
        })
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

#[test]
fn functional_r50_doc_contract_consolidation_markers_present() {
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_doc_contract_non_regression_schema_version=kamn.review.doc-contract-non-regression-ratchet.v1"
    ));
    assert!(
        REVIEW_MARKER_README
            .contains("r<release>_review_doc_contract_non_regression_baseline_test_file_count=<integer>")
    );
    assert!(
        REVIEW_MARKER_README
            .contains("r<release>_review_doc_contract_non_regression_max_test_file_count=<integer>")
    );
    assert!(
        REVIEW_MARKER_README.contains(
            "r<release>_review_doc_contract_non_regression_count_formula=rg --files crates/kamn-core/tests | rg '_docs\\\\.rs$|docs_contract' | wc -l"
        )
    );
    assert!(
        REVIEW_MARKER_README.contains("current doc_contract_test_file_count <= non_regression_max")
    );

    assert!(DOC.contains(
        "r50_review_doc_contract_consolidation_schema_version=kamn.review.doc-contract-suite-consolidation-plan.v1"
    ));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_baseline_test_file_count=82"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_target_test_file_cap=74"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_required_reduction=8"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_tranche_count=2"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_min_reduction_per_tranche=4"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_issue_cap_per_tranche=2"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_target_release=r53"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_status=active"));
    assert!(DOC.contains(
        "r50_review_doc_contract_non_regression_schema_version=kamn.review.doc-contract-non-regression-ratchet.v1"
    ));
    assert!(DOC.contains("r50_review_doc_contract_non_regression_baseline_test_file_count=94"));
    assert!(DOC.contains("r50_review_doc_contract_non_regression_max_test_file_count=94"));
    assert!(DOC.contains(&format!(
        "r50_review_doc_contract_non_regression_count_formula={NON_REGRESSION_COUNT_FORMULA}"
    )));
    assert!(DOC.contains(
        "Doc-contract consolidation contract active (R50.19) with 2 tranches at minimum 4 reductions each toward <=74 files."
    ));
}

#[test]
fn integration_r50_doc_contract_consolidation_markers_are_consistent() {
    let baseline =
        parse_marker_usize("r50_review_doc_contract_consolidation_baseline_test_file_count");
    let target_cap =
        parse_marker_usize("r50_review_doc_contract_consolidation_target_test_file_cap");
    let required_reduction =
        parse_marker_usize("r50_review_doc_contract_consolidation_required_reduction");
    let tranche_count = parse_marker_usize("r50_review_doc_contract_consolidation_tranche_count");
    let min_reduction_per_tranche =
        parse_marker_usize("r50_review_doc_contract_consolidation_min_reduction_per_tranche");
    let issue_cap_per_tranche =
        parse_marker_usize("r50_review_doc_contract_consolidation_issue_cap_per_tranche");
    let non_regression_baseline =
        parse_marker_usize("r50_review_doc_contract_non_regression_baseline_test_file_count");
    let non_regression_max =
        parse_marker_usize("r50_review_doc_contract_non_regression_max_test_file_count");
    let current_doc_contract_test_file_count = current_doc_contract_test_file_count();

    assert!(
        baseline > target_cap,
        "baseline must be greater than target cap"
    );
    assert_eq!(baseline.saturating_sub(target_cap), required_reduction);
    assert!(tranche_count > 0, "tranche count must be positive");
    assert!(
        tranche_count.saturating_mul(min_reduction_per_tranche) >= required_reduction,
        "tranche plan must cover required reduction"
    );
    assert!(
        issue_cap_per_tranche <= 2,
        "issue cap per tranche must remain tightly bounded"
    );
    assert!(
        non_regression_baseline <= non_regression_max,
        "non-regression baseline count must be <= max"
    );
    assert_eq!(
        non_regression_baseline, non_regression_max,
        "non-regression max should stay locked to baseline while remediation is active"
    );
    assert!(
        current_doc_contract_test_file_count <= non_regression_max,
        "current doc-contract test-file count must not exceed non-regression max"
    );
}
