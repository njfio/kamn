use std::{fs, path::PathBuf};

fn repo_file(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_file(path))
        .unwrap_or_else(|_| panic!("expected file to be readable: {path}"))
}

fn line_count(contents: &str) -> usize {
    contents.lines().count()
}

#[test]
fn run_contract_root_is_reduced_to_shell_budget() {
    let root = read("src/run_contract.rs");
    assert!(
        line_count(&root) <= 180,
        "run_contract.rs should be reduced below 180 lines; got {}",
        line_count(&root)
    );
}

#[test]
fn run_contract_root_declares_extracted_modules() {
    let root = read("src/run_contract.rs");
    for marker in [
        "mod evidence_io;",
        "mod external_runtime;",
        "mod orchestration;",
        "#[cfg(test)]\nmod tests;",
    ] {
        assert!(
            root.contains(marker),
            "missing root extraction marker: {marker}"
        );
    }
}

#[test]
fn run_contract_root_no_longer_keeps_inline_monolith_sections() {
    let root = read("src/run_contract.rs");
    for marker in [
        "fn persist_run_evidence_bundle(",
        "fn probe_external_runtime(",
        "fn execute_selected_scenarios(",
        "mod tests {",
    ] {
        assert!(
            !root.contains(marker),
            "run_contract.rs should not keep inline monolith marker: {marker}"
        );
    }
}

#[test]
fn extracted_run_contract_modules_exist_and_stay_bounded() {
    for path in [
        "src/run_contract/evidence_io.rs",
        "src/run_contract/external_runtime.rs",
        "src/run_contract/orchestration.rs",
        "src/run_contract/tests.rs",
    ] {
        let contents = read(path);
        assert!(
            line_count(&contents) <= 200,
            "extracted file should stay within 200 lines: {path} has {}",
            line_count(&contents)
        );
    }
}
