use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const BASELINE_SCHEMA_VERSION: &str = "kamn.ci.shell-test-surface-ratio-baseline.v1";
const THRESHOLD_SCHEMA_VERSION: &str = "kamn.ci.shell-test-surface-ratio-thresholds.v1";
const WAIVER_SCHEMA_VERSION: &str = "kamn.ci.shell-test-surface-ratio-waiver.v1";
const REPORT_SCHEMA_VERSION: &str = "kamn.ci.shell-test-surface-ratio-report.v1";
const REASON_TAXONOMY_VERSION: &str = "kamn.ci.shell-test-surface-ratio-reason-taxonomy.v1";
const REASON_CODES_CSV: &str = "baseline_file_missing,baseline_file_invalid,baseline_schema_invalid,baseline_value_invalid,threshold_file_missing,threshold_file_invalid,threshold_schema_invalid,threshold_value_invalid,waiver_file_invalid,waiver_schema_invalid,waiver_missing_mitigation_issue,waiver_invalid_mitigation_issue,waiver_cap_exceeded,shell_test_file_delta_exceeded,ratio_delta_exceeded,ratio_fail_threshold_exceeded_unwaived,ratio_fail_threshold_waiver_applied";

#[derive(Debug, Clone)]
struct Baseline {
    shell_test_file_count: i64,
    rust_test_file_count: i64,
    shell_to_rust_ratio: f64,
}

#[derive(Debug, Clone)]
struct Thresholds {
    allowed_shell_test_file_delta_max: i64,
    allowed_ratio_delta_max: f64,
    waiver_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct Waiver {
    mitigation_issue: String,
    max_shell_test_file_delta: i64,
    max_ratio_delta: f64,
}

#[derive(Debug, Clone)]
struct CurrentSurface {
    shell_test_file_count: i64,
    rust_test_file_count: i64,
    shell_to_rust_ratio: f64,
}

#[derive(Debug, Clone)]
struct Evaluation {
    policy_status: &'static str,
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

fn required_f64(map: &BTreeMap<String, String>, key: &str, reason_code: &str) -> f64 {
    required_value(map, key, reason_code)
        .parse::<f64>()
        .unwrap_or_else(|error| {
            fail(
                reason_code,
                &format!("key {} must parse as float: {}", key, error),
            )
        })
}

fn optional_path(map: &BTreeMap<String, String>, key: &str) -> Option<PathBuf> {
    map.get(key).and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(repo_path(trimmed))
        }
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

    let shell_test_file_count =
        required_i64(&map, "shell_test_file_count", "baseline_value_invalid");
    let rust_test_file_count = required_i64(&map, "rust_test_file_count", "baseline_value_invalid");
    let shell_to_rust_ratio = required_f64(&map, "shell_to_rust_ratio", "baseline_value_invalid");
    if shell_test_file_count < 0 || rust_test_file_count <= 0 || shell_to_rust_ratio < 0.0 {
        fail(
            "baseline_value_invalid",
            "baseline counts and ratio must be non-negative and rust count must be > 0",
        );
    }

    Baseline {
        shell_test_file_count,
        rust_test_file_count,
        shell_to_rust_ratio,
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

    let allowed_shell_test_file_delta_max = required_i64(
        &map,
        "allowed_shell_test_file_delta_max",
        "threshold_value_invalid",
    );
    let allowed_ratio_delta_max =
        required_f64(&map, "allowed_ratio_delta_max", "threshold_value_invalid");
    if allowed_shell_test_file_delta_max < 0 || allowed_ratio_delta_max < 0.0 {
        fail(
            "threshold_value_invalid",
            "allowed deltas must be non-negative",
        );
    }

    Thresholds {
        allowed_shell_test_file_delta_max,
        allowed_ratio_delta_max,
        waiver_file: optional_path(&map, "waiver_file"),
    }
}

fn load_waiver(path: &Path) -> Waiver {
    let raw = read_file(path, "waiver_file_invalid");
    let map = parse_key_value_fixture(&raw, "waiver_file_invalid");
    let schema_version = required_value(&map, "schema_version", "waiver_schema_invalid");
    if schema_version != WAIVER_SCHEMA_VERSION {
        fail(
            "waiver_schema_invalid",
            &format!(
                "unexpected waiver schema version {} in {}",
                schema_version,
                path.display()
            ),
        );
    }

    let mitigation_issue =
        required_value(&map, "mitigation_issue", "waiver_missing_mitigation_issue").to_owned();
    if !mitigation_issue.starts_with('#')
        || mitigation_issue.len() <= 1
        || !mitigation_issue[1..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        fail(
            "waiver_invalid_mitigation_issue",
            &format!(
                "mitigation_issue must be #<digits>, found {}",
                mitigation_issue
            ),
        );
    }

    let max_shell_test_file_delta =
        required_i64(&map, "max_shell_test_file_delta", "waiver_file_invalid");
    let max_ratio_delta = required_f64(&map, "max_ratio_delta", "waiver_file_invalid");
    if max_shell_test_file_delta < 0 || max_ratio_delta < 0.0 {
        fail(
            "waiver_file_invalid",
            "waiver max deltas must be non-negative",
        );
    }

    Waiver {
        mitigation_issue,
        max_shell_test_file_delta,
        max_ratio_delta,
    }
}

fn walk_files(root: &Path, output: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        fail(
            "threshold_value_invalid",
            &format!("failed to read directory {}: {}", root.display(), error),
        )
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            fail(
                "threshold_value_invalid",
                &format!("failed to read dir entry in {}: {}", root.display(), error),
            )
        });
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, output);
            continue;
        }
        if path.is_file() {
            output.push(path);
        }
    }
}

fn current_surface() -> CurrentSurface {
    let mut shell_files = Vec::new();
    walk_files(&repo_path("scripts"), &mut shell_files);
    let shell_test_file_count = shell_files
        .iter()
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "sh")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("test_"))
        })
        .count() as i64;

    let mut crate_files = Vec::new();
    walk_files(&repo_path("crates"), &mut crate_files);
    let rust_test_file_count = crate_files
        .iter()
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "rs")
                && path
                    .iter()
                    .any(|component| component.to_string_lossy() == "tests")
        })
        .count() as i64;
    if rust_test_file_count <= 0 {
        fail(
            "threshold_value_invalid",
            "rust test file count must be > 0 when computing shell/rust ratio",
        );
    }
    let shell_to_rust_ratio = shell_test_file_count as f64 / rust_test_file_count as f64;

    CurrentSurface {
        shell_test_file_count,
        rust_test_file_count,
        shell_to_rust_ratio,
    }
}

fn evaluate_policy(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
) -> Evaluation {
    let shell_delta = current.shell_test_file_count - baseline.shell_test_file_count;
    let ratio_delta = current.shell_to_rust_ratio - baseline.shell_to_rust_ratio;

    let mut fail_reasons: Vec<&'static str> = Vec::new();
    if shell_delta > thresholds.allowed_shell_test_file_delta_max {
        fail_reasons.push("shell_test_file_delta_exceeded");
    }
    if ratio_delta > thresholds.allowed_ratio_delta_max {
        fail_reasons.push("ratio_delta_exceeded");
    }

    if fail_reasons.is_empty() {
        return Evaluation {
            policy_status: "within",
            final_decision: "GO",
            reason_codes: vec!["none"],
        };
    }

    if let Some(waiver_file) = &thresholds.waiver_file {
        if waiver_file.is_file() {
            let waiver = load_waiver(waiver_file);
            if shell_delta <= waiver.max_shell_test_file_delta
                && ratio_delta <= waiver.max_ratio_delta
            {
                let _ = &waiver.mitigation_issue;
                return Evaluation {
                    policy_status: "waiver-applied",
                    final_decision: "GO",
                    reason_codes: vec!["ratio_fail_threshold_waiver_applied"],
                };
            }
            fail_reasons.push("waiver_cap_exceeded");
        }
    }

    fail_reasons.push("ratio_fail_threshold_exceeded_unwaived");
    Evaluation {
        policy_status: "fail",
        final_decision: "NO-GO",
        reason_codes: fail_reasons,
    }
}

fn maybe_write_report(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
    evaluation: &Evaluation,
) {
    let output_file = match std::env::var("KAMN_SHELL_TEST_SURFACE_RATIO_REPORT") {
        Ok(value) => value,
        Err(_) => return,
    };
    let output_path = repo_path(output_file.trim());
    let shell_delta = current.shell_test_file_count - baseline.shell_test_file_count;
    let rust_delta = current.rust_test_file_count - baseline.rust_test_file_count;
    let ratio_delta = current.shell_to_rust_ratio - baseline.shell_to_rust_ratio;
    let reason_codes = evaluation.reason_codes.join(",");
    let waiver_path = thresholds
        .waiver_file
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "none".to_owned());
    let report = format!(
        "{{\n  \"schema_version\": \"{REPORT_SCHEMA_VERSION}\",\n  \"reason_taxonomy_version\": \"{REASON_TAXONOMY_VERSION}\",\n  \"reason_codes_csv\": \"{REASON_CODES_CSV}\",\n  \"policy_status\": \"{}\",\n  \"final_decision\": \"{}\",\n  \"reason_codes\": \"{}\",\n  \"baseline\": {{\n    \"shell_test_file_count\": {},\n    \"rust_test_file_count\": {},\n    \"shell_to_rust_ratio\": {:.6}\n  }},\n  \"current\": {{\n    \"shell_test_file_count\": {},\n    \"rust_test_file_count\": {},\n    \"shell_to_rust_ratio\": {:.6}\n  }},\n  \"delta\": {{\n    \"shell_test_file_delta\": {},\n    \"rust_test_file_delta\": {},\n    \"ratio_delta\": {:.6}\n  }},\n  \"thresholds\": {{\n    \"allowed_shell_test_file_delta_max\": {},\n    \"allowed_ratio_delta_max\": {:.6},\n    \"waiver_file\": \"{}\"\n  }}\n}}\n",
        evaluation.policy_status,
        evaluation.final_decision,
        reason_codes,
        baseline.shell_test_file_count,
        baseline.rust_test_file_count,
        baseline.shell_to_rust_ratio,
        current.shell_test_file_count,
        current.rust_test_file_count,
        current.shell_to_rust_ratio,
        shell_delta,
        rust_delta,
        ratio_delta,
        thresholds.allowed_shell_test_file_delta_max,
        thresholds.allowed_ratio_delta_max,
        waiver_path,
    );
    if let Err(error) = fs::write(&output_path, report) {
        fail(
            "threshold_value_invalid",
            &format!(
                "failed to write ratio report {}: {}",
                output_path.display(),
                error
            ),
        );
    }
}

#[test]
fn unit_fixtures_declare_expected_schema_markers() {
    let baseline_file = repo_path("fixtures/ci/shell_test_surface_ratio_baseline.env");
    let threshold_file = repo_path(".ci/shell_test_surface_ratio_thresholds.env");

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
fn functional_shell_test_surface_ratio_non_regression_gate() {
    let baseline_file = repo_path("fixtures/ci/shell_test_surface_ratio_baseline.env");
    let threshold_file = repo_path(".ci/shell_test_surface_ratio_thresholds.env");
    let baseline = load_baseline(&baseline_file);
    let thresholds = load_thresholds(&threshold_file);
    let current = current_surface();
    let evaluation = evaluate_policy(&baseline, &thresholds, &current);

    maybe_write_report(&baseline, &thresholds, &current, &evaluation);

    assert_ne!(
        evaluation.final_decision, "NO-GO",
        "reason_taxonomy_version={} reason_codes_csv={} reason_codes={} shell_test_file_count={} rust_test_file_count={} shell_to_rust_ratio={:.6} baseline_shell_test_file_count={} baseline_rust_test_file_count={} baseline_shell_to_rust_ratio={:.6}",
        REASON_TAXONOMY_VERSION,
        REASON_CODES_CSV,
        evaluation.reason_codes.join(","),
        current.shell_test_file_count,
        current.rust_test_file_count,
        current.shell_to_rust_ratio,
        baseline.shell_test_file_count,
        baseline.rust_test_file_count,
        baseline.shell_to_rust_ratio
    );
}

#[test]
fn regression_waiver_mitigation_issue_marker_must_match_issue_format() {
    let repo_tmp = repo_root().join("target/tmp/shell-test-surface-ratio");
    let _ = fs::remove_dir_all(&repo_tmp);
    fs::create_dir_all(&repo_tmp).expect("failed to create tmp ratio fixture directory");
    let invalid_waiver = repo_tmp.join("invalid-waiver.env");
    fs::write(
        &invalid_waiver,
        format!(
            "schema_version={}\nmitigation_issue=not-an-issue-id\nmax_shell_test_file_delta=10\nmax_ratio_delta=0.05\n",
            WAIVER_SCHEMA_VERSION
        ),
    )
    .expect("failed to write invalid waiver fixture");

    let panic_result = std::panic::catch_unwind(|| {
        let _ = load_waiver(&invalid_waiver);
    });
    assert!(
        panic_result.is_err(),
        "invalid waiver mitigation issue format must fail closed"
    );
}
