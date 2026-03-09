#[path = "support/production_expect_surface_policy_support.rs"]
mod support;

use support::{
    count_expect_occurrences_excluding_cfg_test, current_surface, evaluate_policy,
    is_test_only_source_path, load_baseline, load_thresholds, read_file, repo_path, Baseline,
    CurrentSurface, Thresholds, BASELINE_SCHEMA_VERSION, REASON_CODES_CSV, REASON_TAXONOMY_VERSION,
    THRESHOLD_SCHEMA_VERSION,
};

#[test]
fn unit_fixtures_declare_expected_schema_markers() {
    let baseline_file = repo_path("fixtures/ci/production_expect_surface_baseline.env");
    let threshold_file = repo_path(".ci/production_expect_surface_thresholds.env");

    let baseline_text = read_file(&baseline_file, "baseline_file_missing");
    assert!(
        baseline_text.contains(&format!("schema_version={BASELINE_SCHEMA_VERSION}")),
        "baseline fixture must include expected schema marker"
    );
    let threshold_text = read_file(&threshold_file, "threshold_file_missing");
    assert!(
        threshold_text.contains(&format!("schema_version={THRESHOLD_SCHEMA_VERSION}")),
        "threshold fixture must include expected schema marker"
    );
    assert!(
        threshold_text.contains(&format!(
            "reason_taxonomy_version={REASON_TAXONOMY_VERSION}"
        )),
        "threshold fixture must include expected reason taxonomy marker"
    );
    assert!(
        threshold_text.contains(&format!("reason_codes_csv={REASON_CODES_CSV}")),
        "threshold fixture must include deterministic reason code CSV marker"
    );
}

#[test]
fn functional_production_expect_surface_non_regression_gate() {
    let baseline_file = repo_path("fixtures/ci/production_expect_surface_baseline.env");
    let threshold_file = repo_path(".ci/production_expect_surface_thresholds.env");
    let baseline = load_baseline(&baseline_file);
    let thresholds = load_thresholds(&threshold_file);
    let current = current_surface();
    let evaluation = evaluate_policy(&baseline, &thresholds, &current);

    assert_ne!(
        evaluation.final_decision, "NO-GO",
        "reason_taxonomy_version={} reason_codes_csv={} reason_codes={} production_rs_file_count={} production_expect_count={} baseline_production_rs_file_count={} baseline_production_expect_count={}",
        REASON_TAXONOMY_VERSION,
        REASON_CODES_CSV,
        evaluation.reason_codes.join(","),
        current.production_rs_file_count,
        current.production_expect_count,
        baseline.production_rs_file_count,
        baseline.production_expect_count,
    );
}

#[test]
fn regression_expect_surface_policy_fails_when_delta_exceeds_threshold() {
    let current = current_surface();
    let baseline = Baseline {
        production_rs_file_count: current.production_rs_file_count,
        production_expect_count: current.production_expect_count,
    };
    let simulated_current = CurrentSurface {
        production_rs_file_count: current.production_rs_file_count,
        production_expect_count: current.production_expect_count + 1,
    };
    let thresholds = Thresholds {
        allowed_expect_delta_max: 0,
    };
    let evaluation = evaluate_policy(&baseline, &thresholds, &simulated_current);
    assert_eq!(evaluation.final_decision, "NO-GO");
    assert!(evaluation.reason_codes.contains(&"expect_delta_exceeded"));
}

#[test]
fn regression_cfg_test_expect_calls_are_excluded_from_census() {
    let fixture = r#"
fn production_path() {
    let _ = Some(1).expect("count me");
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_only() {
        let _ = Some(2).expect("do not count me");
    }
}

#[cfg(test)]
fn helper_only() {
    let _ = Some(3).expect("do not count me");
}
"#;
    assert_eq!(count_expect_occurrences_excluding_cfg_test(fixture), 1);
}

#[test]
fn regression_nested_cfg_test_module_expect_calls_are_excluded_from_census() {
    let fixture = r#"
fn production_path() {
    let _ = Some(1).expect("count me");
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("fixture.rs");

    fn helper() -> &'static str {
        SOURCE
            .find("\n    }\n}")
            .expect("helper should stay test-only");
        "ok"
    }

    #[test]
    fn unit_only() {
        let _ = helper();
    }
}
"#;
    assert_eq!(count_expect_occurrences_excluding_cfg_test(fixture), 1);
}

#[test]
fn regression_cfg_test_char_literal_quotes_do_not_open_strings_in_census() {
    let fixture = r#"
fn production_path() {
    let _ = Some(1).expect("count me");
}

#[cfg(test)]
mod tests {
    fn helper(ch: char) {
        if ch == '"' {
            let _ = Some(2).expect("do not count me");
        }
    }
}
"#;
    assert_eq!(count_expect_occurrences_excluding_cfg_test(fixture), 1);
}

#[test]
fn regression_test_only_source_paths_are_excluded_from_census() {
    assert!(is_test_only_source_path(
        "crates/kamn-e2e-harness/src/drivers/sdk_direct.rs"
    ));
    assert!(is_test_only_source_path(
        "crates/kamn-node/src/main_tests/runtime_tests.rs"
    ));
    assert!(is_test_only_source_path(
        "crates/kamn-core/src/runtime_tests_snapshot_store.rs"
    ));
    assert!(is_test_only_source_path(
        "crates/kamn-node/src/service_api_endpoint/tests.rs"
    ));
    assert!(is_test_only_source_path(
        "crates/kamn-node/src/service_api_endpoint/test_fixture.rs"
    ));
    assert!(is_test_only_source_path(
        "crates/kamn-node/src/service_api_endpoint/foo_tests.rs"
    ));
    assert!(!is_test_only_source_path("crates/kamn-sdk/src/service.rs"));
}
