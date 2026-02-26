use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BASELINE_SCHEMA_VERSION: &str = "kamn.ci.production-expect-surface-baseline.v1";
const THRESHOLD_SCHEMA_VERSION: &str = "kamn.ci.production-expect-surface-thresholds.v1";
const REASON_TAXONOMY_VERSION: &str = "kamn.ci.production-expect-surface-reason-taxonomy.v1";
const REASON_CODES_CSV: &str = "baseline_file_missing,baseline_file_invalid,baseline_schema_invalid,baseline_value_invalid,threshold_file_missing,threshold_file_invalid,threshold_schema_invalid,threshold_value_invalid,census_command_failed,census_value_invalid,expect_delta_exceeded,expect_threshold_exceeded_unwaived";

#[derive(Debug, Clone)]
struct Baseline {
    production_rs_file_count: i64,
    production_expect_count: i64,
}

#[derive(Debug, Clone)]
struct Thresholds {
    allowed_expect_delta_max: i64,
}

#[derive(Debug, Clone)]
struct CurrentSurface {
    production_rs_file_count: i64,
    production_expect_count: i64,
}

#[derive(Debug, Clone)]
struct Evaluation {
    final_decision: &'static str,
    reason_codes: Vec<&'static str>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn fail(reason_code: &str, detail: &str) -> ! {
    panic!(
        "reason_taxonomy_version={} reason_codes_csv={} reason_code={} detail={}",
        REASON_TAXONOMY_VERSION, REASON_CODES_CSV, reason_code, detail
    );
}

fn read_file(path: &Path, reason_code: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| fail(reason_code, &format!("{}: {}", path.display(), error)))
}

fn parse_key_value_fixture(raw: &str, reason_code: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (index, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once('=').unwrap_or_else(|| {
            fail(
                reason_code,
                &format!("line {} missing key=value form", index + 1),
            )
        });
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() {
            fail(reason_code, &format!("line {} has empty key", index + 1));
        }
        map.insert(key.to_owned(), value.to_owned());
    }
    map
}

fn required_value<'a>(map: &'a BTreeMap<String, String>, key: &str, reason_code: &str) -> &'a str {
    map.get(key)
        .map(String::as_str)
        .unwrap_or_else(|| fail(reason_code, &format!("missing required key {}", key)))
}

fn required_i64(map: &BTreeMap<String, String>, key: &str, reason_code: &str) -> i64 {
    required_value(map, key, reason_code)
        .parse::<i64>()
        .unwrap_or_else(|error| {
            fail(
                reason_code,
                &format!("key {} must parse as integer: {}", key, error),
            )
        })
}

fn load_baseline(path: &Path) -> Baseline {
    if !path.is_file() {
        fail(
            "baseline_file_missing",
            &format!("baseline fixture is missing: {}", path.display()),
        );
    }
    let raw = read_file(path, "baseline_file_invalid");
    let map = parse_key_value_fixture(&raw, "baseline_file_invalid");
    let schema_version = required_value(&map, "schema_version", "baseline_schema_invalid");
    if schema_version != BASELINE_SCHEMA_VERSION {
        fail(
            "baseline_schema_invalid",
            &format!(
                "unexpected baseline schema version {} in {}",
                schema_version,
                path.display()
            ),
        );
    }
    let production_rs_file_count =
        required_i64(&map, "production_rs_file_count", "baseline_value_invalid");
    let production_expect_count =
        required_i64(&map, "production_expect_count", "baseline_value_invalid");
    if production_rs_file_count < 0 || production_expect_count < 0 {
        fail(
            "baseline_value_invalid",
            "baseline counts must be non-negative",
        );
    }
    Baseline {
        production_rs_file_count,
        production_expect_count,
    }
}

fn load_thresholds(path: &Path) -> Thresholds {
    if !path.is_file() {
        fail(
            "threshold_file_missing",
            &format!("threshold fixture is missing: {}", path.display()),
        );
    }
    let raw = read_file(path, "threshold_file_invalid");
    let map = parse_key_value_fixture(&raw, "threshold_file_invalid");
    let schema_version = required_value(&map, "schema_version", "threshold_schema_invalid");
    if schema_version != THRESHOLD_SCHEMA_VERSION {
        fail(
            "threshold_schema_invalid",
            &format!(
                "unexpected threshold schema version {} in {}",
                schema_version,
                path.display()
            ),
        );
    }
    let reason_taxonomy_version =
        required_value(&map, "reason_taxonomy_version", "threshold_schema_invalid");
    if reason_taxonomy_version != REASON_TAXONOMY_VERSION {
        fail(
            "threshold_schema_invalid",
            &format!(
                "unexpected reason taxonomy version {} in {}",
                reason_taxonomy_version,
                path.display()
            ),
        );
    }
    let reason_codes_csv = required_value(&map, "reason_codes_csv", "threshold_schema_invalid");
    if reason_codes_csv != REASON_CODES_CSV {
        fail(
            "threshold_schema_invalid",
            "reason_codes_csv marker mismatch in threshold fixture",
        );
    }

    let allowed_expect_delta_max =
        required_i64(&map, "allowed_expect_delta_max", "threshold_value_invalid");
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

fn tracked_source_files() -> Vec<PathBuf> {
    let output = Command::new("git")
        .arg("ls-tree")
        .arg("-r")
        .arg("--name-only")
        .arg("HEAD")
        .arg("crates")
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|error| fail("census_command_failed", &format!("git ls-tree: {}", error)));
    if !output.status.success() {
        fail(
            "census_command_failed",
            &format!("git ls-tree exited with status {}", output.status),
        );
    }
    let stdout = String::from_utf8(output.stdout).unwrap_or_else(|error| {
        fail(
            "census_command_failed",
            &format!("git ls-tree output is not utf8: {}", error),
        )
    });
    let mut files = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.ends_with(".rs") {
            continue;
        }
        if !trimmed.contains("/src/") || trimmed.contains("/tests/") {
            continue;
        }
        if trimmed.ends_with("_tests.rs") {
            continue;
        }
        files.push(repo_path(trimmed));
    }
    files.sort();
    files
}

fn current_surface() -> CurrentSurface {
    let files = tracked_source_files();
    let production_rs_file_count = files.len() as i64;
    let mut production_expect_count = 0_i64;
    for file in files {
        let raw = fs::read_to_string(&file).unwrap_or_else(|error| {
            fail(
                "census_value_invalid",
                &format!("failed to read source file {}: {}", file.display(), error),
            )
        });
        production_expect_count += raw.match_indices(".expect(").count() as i64;
    }
    if production_rs_file_count < 0 || production_expect_count < 0 {
        fail(
            "census_value_invalid",
            "production source census produced negative counts",
        );
    }
    CurrentSurface {
        production_rs_file_count,
        production_expect_count,
    }
}

fn evaluate_policy(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
) -> Evaluation {
    let expect_delta = current.production_expect_count - baseline.production_expect_count;
    let mut fail_reasons = Vec::new();
    if expect_delta > thresholds.allowed_expect_delta_max {
        fail_reasons.push("expect_delta_exceeded");
    }
    if fail_reasons.is_empty() {
        return Evaluation {
            final_decision: "GO",
            reason_codes: vec!["none"],
        };
    }
    fail_reasons.push("expect_threshold_exceeded_unwaived");
    Evaluation {
        final_decision: "NO-GO",
        reason_codes: fail_reasons,
    }
}

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
    assert!(
        current.production_expect_count > 0,
        "expected at least one expect() call in current production census for regression fixture"
    );
    let baseline = Baseline {
        production_rs_file_count: current.production_rs_file_count,
        production_expect_count: current.production_expect_count - 1,
    };
    let thresholds = Thresholds {
        allowed_expect_delta_max: 0,
    };
    let evaluation = evaluate_policy(&baseline, &thresholds, &current);
    assert_eq!(evaluation.final_decision, "NO-GO");
    assert!(evaluation.reason_codes.contains(&"expect_delta_exceeded"));
}
