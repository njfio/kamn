use std::fs;
use std::path::{Path, PathBuf};

const SDK_DIRECT_ROOT_SOURCE: &str = include_str!("../src/drivers/sdk_direct.rs");
const SDK_DIRECT_TEST_MODULE_FILE: &str = "src/drivers/sdk_direct_tests.rs";
const SDK_DIRECT_TEST_DIR: &str = "src/drivers/sdk_direct_tests";
const ROOT_STAGED_MAX_LINES: usize = 1_800;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_sdk_direct_root_removes_inline_cfg_test_module() {
    assert!(
        !SDK_DIRECT_ROOT_SOURCE.contains("#[cfg(test)]\nmod tests {")
            && !SDK_DIRECT_ROOT_SOURCE.contains("#[cfg(test)]\r\nmod tests {"),
        "sdk_direct.rs must not keep the inline cfg(test) module"
    );
}

#[test]
fn regression_sdk_direct_root_declares_extracted_test_module() {
    assert!(
        SDK_DIRECT_ROOT_SOURCE.contains("mod sdk_direct_tests;"),
        "sdk_direct.rs must declare the extracted sdk_direct_tests submodule"
    );
}

#[test]
fn regression_sdk_direct_extracted_test_module_file_exists() {
    let full_path = manifest_dir().join(SDK_DIRECT_TEST_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted sdk_direct test module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_sdk_direct_extracted_test_layout_exists() {
    for relative_path in [
        "src/drivers/sdk_direct_tests/base_contract_tests.rs",
        "src/drivers/sdk_direct_tests/driver_path_contract_tests.rs",
        "src/drivers/sdk_direct_tests/live_probe_contract_tests.rs",
        "src/drivers/sdk_direct_tests/payload_and_budget_contract_tests.rs",
        "src/drivers/sdk_direct_tests/validator_contract_tests.rs",
        "src/drivers/sdk_direct_tests/support.rs",
        "src/drivers/sdk_direct_tests/support/env_support.rs",
        "src/drivers/sdk_direct_tests/support/script_fixture_support.rs",
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted sdk_direct path missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_sdk_direct_root_respects_staged_line_budget() {
    let line_count = SDK_DIRECT_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "sdk_direct.rs should stay within the staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn regression_sdk_direct_extracted_files_stay_within_line_budget() {
    let base_dir = manifest_dir().join(SDK_DIRECT_TEST_DIR);
    let extracted_files = collected_rs_files(&base_dir);
    assert!(
        !extracted_files.is_empty(),
        "expected extracted sdk_direct test files under {}",
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
        "extracted sdk_direct test files exceed {EXTRACTED_MAX_LINES} LOC: {}",
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
    for entry in fs::read_dir(dir).expect("extracted sdk_direct test dir should be readable") {
        let entry = entry.expect("extracted sdk_direct test entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}
