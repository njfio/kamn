use std::fs;

const SIGNER_EMULATOR_ROOT_MARKERS: [&str; 10] = [
    "#[path = \"signer_backend/signer_emulator_cases.rs\"]",
    "mod signer_emulator_cases;",
    "signer_emulator_cases::run_performance_signer_emulator_contract_lane_stays_within_budget();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_comparator_allows_exact_boundary();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_parser_rejects_invalid_override();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_parser_uses_local_default_when_unset();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set();",
    "signer_emulator_cases::run_performance_signer_emulator_bulk_signing_deep_lane();",
    "fn performance_signer_emulator_contract_lane_stays_within_budget()",
    "fn performance_signer_emulator_bulk_signing_deep_lane()",
];

const SIGNER_EMULATOR_CASES_MARKERS: [&str; 6] = [
    "pub(super) fn run_performance_signer_emulator_contract_lane_stays_within_budget(",
    "pub(super) fn run_regression_signer_emulator_budget_comparator_allows_exact_boundary(",
    "pub(super) fn run_regression_signer_emulator_budget_parser_rejects_invalid_override(",
    "pub(super) fn run_regression_signer_emulator_budget_parser_uses_local_default_when_unset(",
    "pub(super) fn run_regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set(",
    "pub(super) fn run_performance_signer_emulator_bulk_signing_deep_lane(",
];

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_signer_emulator_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/signer_backend.rs");
    let cases = read_repo_file("tests/signer_backend/signer_emulator_cases.rs");

    for marker in SIGNER_EMULATOR_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root signer-backend contract should contain signer-emulator delegation marker: {marker}"
        );
    }

    for marker in SIGNER_EMULATOR_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "signer-emulator cases module should define marker: {marker}"
        );
    }
}
