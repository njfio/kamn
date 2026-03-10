use std::fs;
use std::path::{Path, PathBuf};

const CLI_SCRIPTED_ROOT_SOURCE: &str = include_str!("../src/drivers/cli_scripted.rs");
const CLI_SCRIPTED_TEST_MODULE_FILE: &str = "src/drivers/cli_scripted_tests.rs";
const CLI_SCRIPTED_TEST_DIR: &str = "src/drivers/cli_scripted_tests";
const ROOT_STAGED_MAX_LINES: usize = 2_600;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_cli_scripted_root_removes_inline_cfg_test_module() {
    assert!(
        !CLI_SCRIPTED_ROOT_SOURCE.contains("#[cfg(test)]\nmod tests {")
            && !CLI_SCRIPTED_ROOT_SOURCE.contains("#[cfg(test)]\r\nmod tests {"),
        "cli_scripted.rs must not keep the inline cfg(test) module"
    );
}

#[test]
fn regression_cli_scripted_root_declares_extracted_test_module() {
    assert!(
        CLI_SCRIPTED_ROOT_SOURCE.contains("mod cli_scripted_tests;"),
        "cli_scripted.rs must declare the extracted cli_scripted_tests submodule"
    );
}

#[test]
fn regression_cli_scripted_extracted_test_module_file_exists() {
    let full_path = manifest_dir().join(CLI_SCRIPTED_TEST_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted cli_scripted test module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_cli_scripted_extracted_test_layout_exists() {
    for relative_path in [
        "src/drivers/cli_scripted_tests/base_contract_tests.rs",
        "src/drivers/cli_scripted_tests/continuity_probe_contract_tests.rs",
        "src/drivers/cli_scripted_tests/driver_path_contract_tests.rs",
        "src/drivers/cli_scripted_tests/live_probe_contract_tests.rs",
        "src/drivers/cli_scripted_tests/missing_binary_probe_contract_tests.rs",
        "src/drivers/cli_scripted_tests/missing_binary_probe_extended_contract_tests.rs",
        "src/drivers/cli_scripted_tests/payload_and_budget_contract_tests.rs",
        "src/drivers/cli_scripted_tests/rotation_batch_contract_tests.rs",
        "src/drivers/cli_scripted_tests/validator_contract_tests.rs",
        "src/drivers/cli_scripted_tests/support.rs",
        "src/drivers/cli_scripted_tests/support/env_support.rs",
        "src/drivers/cli_scripted_tests/support/script_fixture_support.rs",
        "src/drivers/cli_scripted_tests/support/script_source_support.rs",
        "src/drivers/cli_scripted_tests/support/update_support.rs",
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted cli_scripted path missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_cli_scripted_root_respects_staged_line_budget() {
    let line_count = CLI_SCRIPTED_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "cli_scripted.rs should stay within the staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn regression_cli_scripted_extracted_files_stay_within_line_budget() {
    let base_dir = manifest_dir().join(CLI_SCRIPTED_TEST_DIR);
    let extracted_files = collected_rs_files(&base_dir);
    assert!(
        !extracted_files.is_empty(),
        "expected extracted cli_scripted test files under {}",
        base_dir.display()
    );

    let offenders = extracted_files
        .into_iter()
        .filter_map(|path| {
            let line_count = fs::read_to_string(&path).ok()?.lines().count();
            (line_count > EXTRACTED_MAX_LINES).then(|| format!("{} ({line_count})", path.display()))
        })
        .collect::<Vec<String>>();

    assert!(
        offenders.is_empty(),
        "extracted cli_scripted test files exceed {EXTRACTED_MAX_LINES} LOC: {}",
        offenders.join(", ")
    );
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collected_rs_files(base_dir: &Path) -> Vec<PathBuf> {
    if !base_dir.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_rs_files_recursive(base_dir, &mut files);
    files
}

fn collect_rs_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("extracted cli_scripted test dir should be readable") {
        let entry = entry.expect("extracted cli_scripted test entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}
