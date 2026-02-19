use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const BASELINE_SCHEMA_VERSION: &str = "kamn.node.main-tests-surface-budget-baseline.v1";
const REASON_TAXONOMY_VERSION: &str = "kamn.node.main-tests-surface-budget-reason-taxonomy.v1";
const REASON_CODES_CSV: &str = "budget_fixture_missing,budget_fixture_json_invalid,budget_fixture_schema_mismatch,budget_threshold_missing,budget_threshold_invalid,main_tests_shell_budget_exceeded,runtime_tests_shell_budget_exceeded,runtime_tests_fragment_budget_exceeded,runtime_tests_fragment_count_below_min";

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn fail(reason_code: &str, detail: &str) -> ! {
    panic!(
        "reason_taxonomy_version={} reason_codes_csv={} reason_code={} detail={}",
        REASON_TAXONOMY_VERSION, REASON_CODES_CSV, reason_code, detail
    );
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        fail(
            "budget_fixture_missing",
            &format!("{}: {}", path.display(), error),
        )
    })
}

fn file_line_count(path: &Path) -> usize {
    read_file(path).lines().count()
}

fn required_u64(value: &Value, key: &str) -> u64 {
    let raw = value.get(key).unwrap_or_else(|| {
        fail(
            "budget_threshold_missing",
            &format!("missing threshold key {}", key),
        )
    });
    raw.as_u64().unwrap_or_else(|| {
        fail(
            "budget_threshold_invalid",
            &format!("threshold key {} must be a non-negative integer", key),
        )
    })
}

#[test]
fn node_main_tests_surface_budget_baseline_contract_remains_within_thresholds() {
    let baseline_path =
        repo_path("../../fixtures/ci/main_tests_runtime_surface_budget_baseline.json");
    if !baseline_path.is_file() {
        fail(
            "budget_fixture_missing",
            &format!("baseline fixture not found: {}", baseline_path.display()),
        );
    }

    let baseline_raw = read_file(&baseline_path);
    let baseline_json: Value = serde_json::from_str(&baseline_raw).unwrap_or_else(|error| {
        fail(
            "budget_fixture_json_invalid",
            &format!("invalid baseline json: {}", error),
        )
    });

    let schema_version = baseline_json
        .get("schema_version")
        .and_then(Value::as_str)
        .unwrap_or_else(|| {
            fail(
                "budget_fixture_schema_mismatch",
                "baseline schema_version missing or not a string",
            )
        });
    if schema_version != BASELINE_SCHEMA_VERSION {
        fail(
            "budget_fixture_schema_mismatch",
            &format!(
                "unexpected baseline schema version {}, expected {}",
                schema_version, BASELINE_SCHEMA_VERSION
            ),
        );
    }

    let main_tests_shell_max_lines =
        required_u64(&baseline_json, "main_tests_shell_max_lines") as usize;
    let runtime_tests_shell_max_lines =
        required_u64(&baseline_json, "runtime_tests_shell_max_lines") as usize;
    let runtime_tests_fragment_max_lines =
        required_u64(&baseline_json, "runtime_tests_fragment_max_lines") as usize;
    let runtime_tests_fragment_min_count =
        required_u64(&baseline_json, "runtime_tests_fragment_min_count") as usize;

    let main_tests_shell_path = repo_path("src/main_tests.rs");
    let runtime_tests_shell_path = repo_path("src/main_tests/runtime_tests.rs");
    let runtime_tests_fragment_dir = repo_path("src/main_tests/runtime_tests");

    let main_tests_lines = file_line_count(&main_tests_shell_path);
    if main_tests_lines > main_tests_shell_max_lines {
        fail(
            "main_tests_shell_budget_exceeded",
            &format!(
                "{} has {} lines; max is {}",
                main_tests_shell_path.display(),
                main_tests_lines,
                main_tests_shell_max_lines
            ),
        );
    }

    let runtime_tests_shell_lines = file_line_count(&runtime_tests_shell_path);
    if runtime_tests_shell_lines > runtime_tests_shell_max_lines {
        fail(
            "runtime_tests_shell_budget_exceeded",
            &format!(
                "{} has {} lines; max is {}",
                runtime_tests_shell_path.display(),
                runtime_tests_shell_lines,
                runtime_tests_shell_max_lines
            ),
        );
    }

    let mut fragment_paths: Vec<PathBuf> = fs::read_dir(&runtime_tests_fragment_dir)
        .unwrap_or_else(|error| {
            fail(
                "runtime_tests_fragment_count_below_min",
                &format!(
                    "failed to read fragment directory {}: {}",
                    runtime_tests_fragment_dir.display(),
                    error
                ),
            )
        })
        .map(|entry| {
            entry
                .unwrap_or_else(|error| {
                    fail(
                        "runtime_tests_fragment_count_below_min",
                        &format!("failed to read fragment dir entry: {}", error),
                    )
                })
                .path()
        })
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    fragment_paths.sort();

    if fragment_paths.len() < runtime_tests_fragment_min_count {
        fail(
            "runtime_tests_fragment_count_below_min",
            &format!(
                "{} fragment count {} is below min {}",
                runtime_tests_fragment_dir.display(),
                fragment_paths.len(),
                runtime_tests_fragment_min_count
            ),
        );
    }

    for fragment_path in fragment_paths {
        let fragment_lines = file_line_count(&fragment_path);
        if fragment_lines > runtime_tests_fragment_max_lines {
            fail(
                "runtime_tests_fragment_budget_exceeded",
                &format!(
                    "{} has {} lines; max is {}",
                    fragment_path.display(),
                    fragment_lines,
                    runtime_tests_fragment_max_lines
                ),
            );
        }
    }
}
