use crate::support::constants::{
    BASELINE_SCHEMA_VERSION, REASON_CODES_CSV, REASON_TAXONOMY_VERSION,
    THRESHOLD_SCHEMA_VERSION,
};
use crate::support::paths::{read_file, repo_path};

#[test]
fn unit_fixtures_declare_expected_schema_markers() {
    let baseline_text = read_file(
        &repo_path("fixtures/ci/shell_test_surface_ratio_baseline.env"),
        "baseline_file_missing",
    );
    assert!(baseline_text.contains(&format!("schema_version={BASELINE_SCHEMA_VERSION}")));

    let threshold_text = read_file(
        &repo_path(".ci/shell_test_surface_ratio_thresholds.env"),
        "threshold_file_missing",
    );
    assert!(threshold_text.contains(&format!("schema_version={THRESHOLD_SCHEMA_VERSION}")));
    assert!(threshold_text.contains(&format!("reason_taxonomy_version={REASON_TAXONOMY_VERSION}")));
    assert!(threshold_text.contains(&format!("reason_codes_csv={REASON_CODES_CSV}")));
}
