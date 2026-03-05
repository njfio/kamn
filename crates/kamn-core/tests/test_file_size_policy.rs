use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const THRESHOLD_SCHEMA_VERSION: &str = "kamn.ci.test-file-size-policy-thresholds.v1";
const BASELINE_SCHEMA_VERSION: &str = "kamn.ci.test-file-size-policy-baseline.v1";

#[derive(Debug)]
struct Thresholds {
    soft_warn_lines: usize,
    severe_refactor_lines: usize,
    hard_fail_lines: usize,
    max_soft_warn_count: usize,
    max_severe_count: usize,
    max_hard_fail_count: usize,
}

#[derive(Debug)]
struct Baseline {
    test_file_total: usize,
    soft_warn_count: usize,
    severe_count: usize,
    hard_fail_count: usize,
    severe_allowlist: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn load_env_map(path: &Path) -> HashMap<String, String> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let mut map = HashMap::new();
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once('=').unwrap_or_else(|| {
            panic!(
                "invalid env line in {} at {}: {}",
                path.display(),
                line_idx + 1,
                line
            )
        });
        map.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    map
}

fn required_value(map: &HashMap<String, String>, key: &str) -> String {
    map.get(key)
        .cloned()
        .unwrap_or_else(|| panic!("missing required key `{key}`"))
}

fn required_usize(map: &HashMap<String, String>, key: &str) -> usize {
    required_value(map, key)
        .parse::<usize>()
        .unwrap_or_else(|error| {
            panic!("invalid usize for key `{key}`: {error}");
        })
}

fn load_thresholds(root: &Path) -> Thresholds {
    let threshold_path = root.join(".ci/test_file_size_policy_thresholds.env");
    let map = load_env_map(&threshold_path);
    let schema_version = required_value(&map, "schema_version");
    assert_eq!(
        schema_version, THRESHOLD_SCHEMA_VERSION,
        "unexpected threshold schema version"
    );
    Thresholds {
        soft_warn_lines: required_usize(&map, "soft_warn_lines"),
        severe_refactor_lines: required_usize(&map, "severe_refactor_lines"),
        hard_fail_lines: required_usize(&map, "hard_fail_lines"),
        max_soft_warn_count: required_usize(&map, "max_soft_warn_count"),
        max_severe_count: required_usize(&map, "max_severe_count"),
        max_hard_fail_count: required_usize(&map, "max_hard_fail_count"),
    }
}

fn load_baseline(root: &Path) -> Baseline {
    let baseline_path = root.join("fixtures/ci/test_file_size_policy_baseline.env");
    let map = load_env_map(&baseline_path);
    let schema_version = required_value(&map, "schema_version");
    assert_eq!(
        schema_version, BASELINE_SCHEMA_VERSION,
        "unexpected baseline schema version"
    );
    let allowlist_csv = required_value(&map, "severe_allowlist_csv");
    Baseline {
        test_file_total: required_usize(&map, "test_file_total"),
        soft_warn_count: required_usize(&map, "soft_warn_count"),
        severe_count: required_usize(&map, "severe_count"),
        hard_fail_count: required_usize(&map, "hard_fail_count"),
        severe_allowlist: allowlist_csv
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect(),
    }
}

fn collect_test_files(path: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", path.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_test_files(&entry_path, out);
            continue;
        }
        if entry_path.extension().is_some_and(|ext| ext == "rs")
            && entry_path
                .components()
                .any(|component| component.as_os_str() == "tests")
        {
            out.push(entry_path);
        }
    }
}

fn all_test_file_lines(root: &Path) -> Vec<(String, usize)> {
    let mut files = Vec::new();
    collect_test_files(&root.join("crates"), &mut files);
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let relative = path
                .strip_prefix(root)
                .expect("file should be under repo root")
                .to_string_lossy()
                .replace('\\', "/");
            if relative.contains("/tests/support/") {
                return None;
            }
            let lines = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .count();
            Some((relative, lines))
        })
        .collect()
}

fn offender_paths(files: &[(String, usize)], threshold: usize) -> Vec<String> {
    files
        .iter()
        .filter(|(_, lines)| *lines > threshold)
        .map(|(path, _)| path.clone())
        .collect()
}

#[test]
fn spec_c01_test_file_size_policy_files_have_expected_schema_versions() {
    let root = repo_root();
    let _ = load_thresholds(&root);
    let _ = load_baseline(&root);
}

#[test]
fn spec_c02_first_wave_command_contract_monolith_is_below_severe_threshold() {
    let root = repo_root();
    let thresholds = load_thresholds(&root);
    let command_contract = root.join("crates/kamn-e2e-harness/tests/command_contract.rs");
    let split_target = root.join("crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs");

    let command_lines = fs::read_to_string(&command_contract)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", command_contract.display()))
        .lines()
        .count();
    assert!(
        command_lines <= thresholds.severe_refactor_lines,
        "first-wave severe threshold exceeded for {}: line_count={} threshold={}",
        command_contract.display(),
        command_lines,
        thresholds.severe_refactor_lines
    );
    assert!(
        split_target.is_file(),
        "missing split target {}",
        split_target.display()
    );
}

#[test]
fn spec_c03_severe_offender_allowlist_matches_baseline() {
    let root = repo_root();
    let thresholds = load_thresholds(&root);
    let baseline = load_baseline(&root);
    let files = all_test_file_lines(&root);

    let mut offenders = offender_paths(&files, thresholds.severe_refactor_lines);
    let mut allowlist = baseline.severe_allowlist.clone();
    offenders.sort();
    allowlist.sort();
    assert_eq!(
        offenders, allowlist,
        "severe offender allowlist drift: expected {:?} got {:?}",
        allowlist, offenders
    );
}

#[test]
fn spec_c04_oversized_test_counts_are_within_budget() {
    let root = repo_root();
    let thresholds = load_thresholds(&root);
    let baseline = load_baseline(&root);
    let files = all_test_file_lines(&root);

    let soft_count = files
        .iter()
        .filter(|(_, lines)| *lines > thresholds.soft_warn_lines)
        .count();
    let severe_count = files
        .iter()
        .filter(|(_, lines)| *lines > thresholds.severe_refactor_lines)
        .count();
    let hard_count = files
        .iter()
        .filter(|(_, lines)| *lines > thresholds.hard_fail_lines)
        .count();

    assert_eq!(
        files.len(),
        baseline.test_file_total,
        "test file inventory drift"
    );
    assert_eq!(
        soft_count, baseline.soft_warn_count,
        "soft oversized count drift"
    );
    assert_eq!(
        severe_count, baseline.severe_count,
        "severe oversized count drift"
    );
    assert_eq!(
        hard_count, baseline.hard_fail_count,
        "hard oversized count drift"
    );

    assert!(
        soft_count <= thresholds.max_soft_warn_count,
        "soft oversized budget exceeded: count={} max={}",
        soft_count,
        thresholds.max_soft_warn_count
    );
    assert!(
        severe_count <= thresholds.max_severe_count,
        "severe oversized budget exceeded: count={} max={}",
        severe_count,
        thresholds.max_severe_count
    );
    assert!(
        hard_count <= thresholds.max_hard_fail_count,
        "hard oversized budget exceeded: count={} max={}",
        hard_count,
        thresholds.max_hard_fail_count
    );
}
