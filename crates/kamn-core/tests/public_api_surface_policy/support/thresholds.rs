use crate::support::constants::{THRESHOLD_SCHEMA_VERSION, WAIVER_SCHEMA_VERSION};
use crate::support::fixtures::{parse_key_value_fixture, required_i64, required_value};
use crate::support::models::{PolicyThresholds, PolicyWaiver};
use crate::support::paths::{fail, read_file, repo_path};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn load_thresholds(path: &Path) -> PolicyThresholds {
    if !path.is_file() {
        fail(
            "threshold_fixture_missing",
            &format!("missing threshold fixture {}", path.display()),
        );
    }
    let map = parse_key_value_fixture(
        &read_file(path, "threshold_fixture_missing"),
        "threshold_fixture_invalid",
    );
    assert_threshold_schema(&map, path);
    let thresholds = PolicyThresholds {
        warn_total_delta_max: required_i64(&map, "warn_total_delta_max", "threshold_value_invalid"),
        fail_total_delta_max: required_i64(&map, "fail_total_delta_max", "threshold_value_invalid"),
        waiver_file: waiver_path(&map),
    };
    assert_threshold_order(&thresholds);
    thresholds
}

pub(crate) fn load_waiver(path: &Path) -> PolicyWaiver {
    let map = parse_key_value_fixture(
        &read_file(path, "waiver_fixture_invalid"),
        "waiver_fixture_invalid",
    );
    assert_waiver_schema(&map, path);
    let mitigation_issue =
        required_value(&map, "mitigation_issue", "waiver_missing_mitigation_issue");
    assert_mitigation_issue(mitigation_issue, path);
    let max_total_delta = required_i64(&map, "max_total_delta", "waiver_fixture_invalid");
    if max_total_delta < 0 {
        fail(
            "waiver_fixture_invalid",
            &format!("max_total_delta must be non-negative in {}", path.display()),
        );
    }
    PolicyWaiver {
        mitigation_issue: mitigation_issue.to_owned(),
        max_total_delta,
    }
}

fn assert_threshold_schema(map: &BTreeMap<String, String>, path: &Path) {
    let schema_version = required_value(map, "schema_version", "threshold_schema_mismatch");
    if schema_version != THRESHOLD_SCHEMA_VERSION {
        fail(
            "threshold_schema_mismatch",
            &format!("unexpected schema {} in {}", schema_version, path.display()),
        );
    }
}

fn waiver_path(map: &BTreeMap<String, String>) -> Option<std::path::PathBuf> {
    map.get("waiver_file")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "None")
        .map(repo_path)
}

fn assert_threshold_order(thresholds: &PolicyThresholds) {
    if thresholds.warn_total_delta_max > thresholds.fail_total_delta_max {
        fail(
            "threshold_value_invalid",
            &format!(
                "warn_total_delta_max ({}) must be <= fail_total_delta_max ({})",
                thresholds.warn_total_delta_max, thresholds.fail_total_delta_max
            ),
        );
    }
}

fn assert_waiver_schema(map: &BTreeMap<String, String>, path: &Path) {
    let schema_version = required_value(map, "schema_version", "waiver_schema_mismatch");
    if schema_version != WAIVER_SCHEMA_VERSION {
        fail(
            "waiver_schema_mismatch",
            &format!(
                "unexpected waiver schema {} in {}",
                schema_version,
                path.display()
            ),
        );
    }
}

fn assert_mitigation_issue(mitigation_issue: &str, path: &Path) {
    let valid = mitigation_issue.starts_with('#')
        && mitigation_issue
            .trim_start_matches('#')
            .chars()
            .all(|ch| ch.is_ascii_digit());
    if !valid {
        fail(
            "waiver_invalid_mitigation_issue",
            &format!(
                "mitigation_issue must be #<digits>, got {} in {}",
                mitigation_issue,
                path.display()
            ),
        );
    }
}
