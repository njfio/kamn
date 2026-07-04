use std::path::Path;

use super::fixture_parsing_support::{parse_key_value_fixture, required_i64, required_value};
use super::source_path_support::{fail, read_file};

pub const BASELINE_SCHEMA_VERSION: &str = "kamn.ci.production-expect-surface-baseline.v1";
pub const THRESHOLD_SCHEMA_VERSION: &str = "kamn.ci.production-expect-surface-thresholds.v1";
pub const REASON_TAXONOMY_VERSION: &str = "kamn.ci.production-expect-surface-reason-taxonomy.v1";
pub const REASON_CODES_CSV: &str = "baseline_file_missing,baseline_file_invalid,baseline_schema_invalid,baseline_value_invalid,threshold_file_missing,threshold_file_invalid,threshold_schema_invalid,threshold_value_invalid,census_command_failed,census_value_invalid,expect_delta_exceeded,expect_threshold_exceeded_unwaived";

#[derive(Debug, Clone)]
pub struct Baseline {
    pub production_rs_file_count: i64,
    pub production_expect_count: i64,
}

#[derive(Debug, Clone)]
pub struct Thresholds {
    pub allowed_expect_delta_max: i64,
}

#[derive(Debug, Clone)]
pub struct CurrentSurface {
    pub production_rs_file_count: i64,
    pub production_expect_count: i64,
}

#[derive(Debug, Clone)]
pub struct Evaluation {
    pub final_decision: &'static str,
    pub reason_codes: Vec<&'static str>,
}

pub fn load_baseline(path: &Path) -> Baseline {
    ensure_fixture_exists(path, "baseline_file_missing", "baseline fixture");
    let raw = read_file(path, "baseline_file_invalid");
    let map = parse_key_value_fixture(&raw, "baseline_file_invalid");
    assert_schema(
        required_value(&map, "schema_version", "baseline_schema_invalid"),
        BASELINE_SCHEMA_VERSION,
        "baseline_schema_invalid",
        path,
        "baseline",
    );
    build_baseline(&map)
}

pub fn load_thresholds(path: &Path) -> Thresholds {
    ensure_fixture_exists(path, "threshold_file_missing", "threshold fixture");
    let raw = read_file(path, "threshold_file_invalid");
    let map = parse_key_value_fixture(&raw, "threshold_file_invalid");
    assert_schema(
        required_value(&map, "schema_version", "threshold_schema_invalid"),
        THRESHOLD_SCHEMA_VERSION,
        "threshold_schema_invalid",
        path,
        "threshold",
    );
    assert_reason_markers(&map, path);
    build_thresholds(&map)
}

fn ensure_fixture_exists(path: &Path, reason_code: &str, label: &str) {
    if !path.is_file() {
        let display_path = path.display();
        fail(reason_code, &format!("{label} is missing: {display_path}"));
    }
}

fn assert_schema(actual: &str, expected: &str, reason_code: &str, path: &Path, label: &str) {
    if actual != expected {
        let display_path = path.display();
        fail(
            reason_code,
            &format!("unexpected {label} schema version {actual} in {display_path}",),
        );
    }
}

fn assert_reason_markers(map: &std::collections::BTreeMap<String, String>, path: &Path) {
    let actual_taxonomy =
        required_value(map, "reason_taxonomy_version", "threshold_schema_invalid");
    if actual_taxonomy != REASON_TAXONOMY_VERSION {
        let display_path = path.display();
        fail(
            "threshold_schema_invalid",
            &format!("unexpected reason taxonomy version {actual_taxonomy} in {display_path}",),
        );
    }
    let actual_csv = required_value(map, "reason_codes_csv", "threshold_schema_invalid");
    if actual_csv != REASON_CODES_CSV {
        fail(
            "threshold_schema_invalid",
            "reason_codes_csv marker mismatch in threshold fixture",
        );
    }
}

fn build_baseline(map: &std::collections::BTreeMap<String, String>) -> Baseline {
    let production_rs_file_count =
        required_i64(map, "production_rs_file_count", "baseline_value_invalid");
    let production_expect_count =
        required_i64(map, "production_expect_count", "baseline_value_invalid");
    assert_non_negative(
        production_rs_file_count,
        production_expect_count,
        "baseline_value_invalid",
        "baseline counts must be non-negative",
    );
    Baseline {
        production_rs_file_count,
        production_expect_count,
    }
}

fn build_thresholds(map: &std::collections::BTreeMap<String, String>) -> Thresholds {
    let allowed_expect_delta_max =
        required_i64(map, "allowed_expect_delta_max", "threshold_value_invalid");
    if allowed_expect_delta_max < 0 {
        fail(
            "threshold_value_invalid",
            "allowed expect delta must be non-negative",
        );
    }
    Thresholds {
        allowed_expect_delta_max,
    }
}

fn assert_non_negative(
    production_rs_file_count: i64,
    production_expect_count: i64,
    reason_code: &str,
    message: &str,
) {
    if production_rs_file_count < 0 || production_expect_count < 0 {
        fail(reason_code, message);
    }
}
