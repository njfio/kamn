use std::fs;
use std::path::Path;

#[test]
fn spec_c03_verify_matrix_retains_explicit_error_path_assertions_for_result_surfaces() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/command_contract_verify_matrix.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    let required_markers = [
        "parse_command_args([\"run\", \"--mode\", \"sdk-direct\", \"--evidence-dir\"]).expect_err",
        "parse_scenario_csv(\"S-01,S-99\").expect_err",
        "execute_verify_contract(&config).expect_err",
    ];

    for marker in required_markers {
        assert!(
            content.contains(marker),
            "missing explicit error-path marker in verify matrix file: {marker}"
        );
    }
}
