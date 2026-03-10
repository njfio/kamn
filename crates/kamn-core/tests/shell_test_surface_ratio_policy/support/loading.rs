use crate::support::constants::{
    BASELINE_SCHEMA_VERSION, REASON_CODES_CSV, REASON_TAXONOMY_VERSION, THRESHOLD_SCHEMA_VERSION,
    WAIVER_SCHEMA_VERSION,
};
use crate::support::fixtures::{
    optional_i64_with_default, optional_path, parse_key_value_fixture, required_f64, required_i64,
    required_value,
};
use crate::support::models::{Baseline, Thresholds, Waiver};
use crate::support::paths::{fail, read_file, repo_path};
use std::collections::BTreeMap;
use std::path::Path;

type FixtureMap = BTreeMap<String, String>;

pub(crate) fn load_baseline(path: &Path) -> Baseline {
    if !path.is_file() {
        fail(
            "baseline_file_missing",
            &format!("baseline fixture is missing: {}", path.display()),
        );
    }
    let map = load_fixture_map(path, "baseline_file_invalid");
    assert_schema_marker(
        &map,
        BASELINE_SCHEMA_VERSION,
        "baseline_schema_invalid",
        path,
    );
    let baseline = parse_baseline(&map);
    assert_baseline_values(&baseline);
    baseline
}

pub(crate) fn load_thresholds(path: &Path) -> Thresholds {
    if !path.is_file() {
        fail(
            "threshold_file_missing",
            &format!("threshold fixture is missing: {}", path.display()),
        );
    }
    let map = load_fixture_map(path, "threshold_file_invalid");
    assert_schema_marker(
        &map,
        THRESHOLD_SCHEMA_VERSION,
        "threshold_schema_invalid",
        path,
    );
    assert_threshold_markers(&map);
    let thresholds = parse_thresholds(&map);
    if thresholds.allowed_shell_test_file_delta_max < 0 || thresholds.allowed_ratio_delta_max < 0.0
    {
        fail(
            "threshold_value_invalid",
            "allowed deltas must be non-negative",
        );
    }
    thresholds
}

pub(crate) fn load_waiver(path: &Path) -> Waiver {
    let map = load_fixture_map(path, "waiver_file_invalid");
    assert_schema_marker(&map, WAIVER_SCHEMA_VERSION, "waiver_schema_invalid", path);
    let mitigation_issue =
        required_value(&map, "mitigation_issue", "waiver_missing_mitigation_issue").to_owned();
    assert_issue_marker(&mitigation_issue);
    let waiver = Waiver {
        mitigation_issue,
        max_shell_test_file_delta: required_i64(
            &map,
            "max_shell_test_file_delta",
            "waiver_file_invalid",
        ),
        max_ratio_delta: required_f64(&map, "max_ratio_delta", "waiver_file_invalid"),
    };
    if waiver.max_shell_test_file_delta < 0 || waiver.max_ratio_delta < 0.0 {
        fail(
            "waiver_file_invalid",
            "waiver max deltas must be non-negative",
        );
    }
    waiver
}

fn load_fixture_map(path: &Path, reason_code: &str) -> FixtureMap {
    parse_key_value_fixture(&read_file(path, reason_code), reason_code)
}

fn parse_baseline(map: &FixtureMap) -> Baseline {
    Baseline {
        shell_test_file_count: required_i64(map, "shell_test_file_count", "baseline_value_invalid"),
        rust_test_file_count: required_i64(map, "rust_test_file_count", "baseline_value_invalid"),
        docs_rust_test_file_count: optional_i64_with_default(
            map,
            "docs_rust_test_file_count",
            0,
            "baseline_value_invalid",
        ),
        shell_to_rust_ratio: required_f64(map, "shell_to_rust_ratio", "baseline_value_invalid"),
    }
}

fn parse_thresholds(map: &FixtureMap) -> Thresholds {
    Thresholds {
        allowed_shell_test_file_delta_max: required_i64(
            map,
            "allowed_shell_test_file_delta_max",
            "threshold_value_invalid",
        ),
        allowed_ratio_delta_max: required_f64(
            map,
            "allowed_ratio_delta_max",
            "threshold_value_invalid",
        ),
        waiver_file: optional_path(map, "waiver_file", repo_path),
    }
}

fn assert_schema_marker(map: &FixtureMap, expected: &str, reason_code: &str, path: &Path) {
    let value = required_value(map, "schema_version", reason_code);
    if value != expected {
        fail(
            reason_code,
            &format!("unexpected schema version {} in {}", value, path.display()),
        );
    }
}

fn assert_threshold_markers(map: &FixtureMap) {
    assert_text_marker(
        required_value(map, "reason_taxonomy_version", "threshold_schema_invalid"),
        REASON_TAXONOMY_VERSION,
        "threshold_schema_invalid",
        "unexpected reason taxonomy version",
    );
    assert_text_marker(
        required_value(map, "reason_codes_csv", "threshold_schema_invalid"),
        REASON_CODES_CSV,
        "threshold_schema_invalid",
        "reason_codes_csv marker mismatch in threshold fixture",
    );
}

fn assert_text_marker(value: &str, expected: &str, reason_code: &str, detail: &str) {
    if value != expected {
        fail(reason_code, detail);
    }
}

fn assert_baseline_values(baseline: &Baseline) {
    if baseline.shell_test_file_count < 0
        || baseline.rust_test_file_count <= 0
        || baseline.docs_rust_test_file_count < 0
        || baseline.shell_to_rust_ratio < 0.0
    {
        fail(
            "baseline_value_invalid",
            "baseline counts and ratio must be non-negative and rust count must be > 0",
        );
    }
}

fn assert_issue_marker(mitigation_issue: &str) {
    let valid = mitigation_issue.starts_with('#')
        && mitigation_issue.len() > 1
        && mitigation_issue[1..]
            .chars()
            .all(|character| character.is_ascii_digit());
    if !valid {
        fail(
            "waiver_invalid_mitigation_issue",
            &format!(
                "mitigation_issue must be #<digits>, found {}",
                mitigation_issue
            ),
        );
    }
}
