use std::fs;
use std::path::{Path, PathBuf};

const SDK_DIRECT_ROOT_SOURCE: &str = include_str!("../src/drivers/sdk_direct.rs");
const SDK_DIRECT_TRANCHE_MODULE_FILE: &str = "src/drivers/sdk_direct/live_probe_tranche_one.rs";
const SDK_DIRECT_TRANCHE_DIR: &str = "src/drivers/sdk_direct/live_probe_tranche_one";
const ROOT_STAGED_MAX_LINES: usize = 1_300;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_sdk_direct_root_declares_live_probe_tranche_module() {
    assert!(
        SDK_DIRECT_ROOT_SOURCE.contains("mod live_probe_tranche_one;"),
        "sdk_direct.rs must declare the extracted live_probe_tranche_one module"
    );
}

#[test]
fn regression_sdk_direct_root_removes_s01_through_s05_probe_definitions() {
    for marker in [
        "fn run_live_s01_discovery_probe()",
        "fn run_live_s02_direct_message_probe()",
        "fn run_live_s03_group_channel_probe()",
        "fn run_live_s04_task_lifecycle_probe()",
        "fn run_live_s05_escrow_settlement_probe()",
        "fn validate_live_s03_query_message_response(",
        "fn validate_live_s03_list_messages_response(",
        "fn validate_live_s05_release_escrow_receipt(",
    ] {
        assert!(
            !SDK_DIRECT_ROOT_SOURCE.contains(marker),
            "sdk_direct.rs must not keep tranche-one implementation marker: {marker}"
        );
    }
}

#[test]
fn regression_sdk_direct_tranche_module_file_exists() {
    let full_path = manifest_dir().join(SDK_DIRECT_TRANCHE_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted tranche module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_sdk_direct_tranche_layout_exists() {
    for relative_path in [
        "src/drivers/sdk_direct/live_probe_tranche_one.rs",
        "src/drivers/sdk_direct/live_probe_tranche_one/discovery_direct_message_probes.rs",
        "src/drivers/sdk_direct/live_probe_tranche_one/channel_task_probes.rs",
        "src/drivers/sdk_direct/live_probe_tranche_one/escrow_probe_support.rs",
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted sdk_direct tranche path missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_sdk_direct_root_respects_tranche_one_staged_line_budget() {
    let line_count = SDK_DIRECT_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "sdk_direct.rs should stay within the tranche-one staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn regression_sdk_direct_tranche_files_stay_within_line_budget() {
    let base_dir = manifest_dir().join(SDK_DIRECT_TRANCHE_DIR);
    let extracted_files = collected_rs_files(&base_dir);
    assert!(
        !extracted_files.is_empty(),
        "expected extracted sdk_direct tranche files under {}",
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
        "extracted sdk_direct tranche files exceed {EXTRACTED_MAX_LINES} LOC: {}",
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
    for entry in fs::read_dir(dir).expect("extracted sdk_direct tranche dir should be readable") {
        let entry = entry.expect("extracted sdk_direct tranche entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}
