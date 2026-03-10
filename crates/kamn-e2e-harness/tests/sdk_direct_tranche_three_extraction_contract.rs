use std::fs;
use std::path::{Path, PathBuf};

const SDK_DIRECT_ROOT_SOURCE: &str = include_str!("../src/drivers/sdk_direct.rs");
const SDK_DIRECT_TRANCHE_MODULE_FILE: &str = "src/drivers/sdk_direct/live_probe_tranche_three.rs";
const SDK_DIRECT_TRANCHE_DIR: &str = "src/drivers/sdk_direct/live_probe_tranche_three";
const ROOT_STAGED_MAX_LINES: usize = 400;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_sdk_direct_root_declares_live_probe_tranche_three_module() {
    assert!(
        SDK_DIRECT_ROOT_SOURCE.contains("mod live_probe_tranche_three;"),
        "sdk_direct.rs must declare the extracted live_probe_tranche_three module"
    );
}

#[test]
fn regression_sdk_direct_root_removes_s11_through_s15_probe_definitions() {
    for marker in [
        "fn run_live_s11_signer_rotation_probe()",
        "fn run_live_s12_retention_deletion_probe()",
        "fn run_live_s13_bridge_forwarding_probe()",
        "fn run_live_s14_batch_merkle_probe()",
        "fn run_live_s15_performance_smoke_probe()",
        "fn validate_s14_proof_response(",
    ] {
        assert!(
            !SDK_DIRECT_ROOT_SOURCE.contains(marker),
            "sdk_direct.rs must not keep tranche-three implementation marker: {marker}"
        );
    }
}

#[test]
fn regression_sdk_direct_tranche_three_module_file_exists() {
    let full_path = manifest_dir().join(SDK_DIRECT_TRANCHE_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted tranche module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_sdk_direct_tranche_three_layout_exists() {
    for relative_path in [
        "src/drivers/sdk_direct/live_probe_tranche_three.rs",
        "src/drivers/sdk_direct/live_probe_tranche_three/signer_rotation_probe.rs",
        "src/drivers/sdk_direct/live_probe_tranche_three/retention_deletion_probe.rs",
        "src/drivers/sdk_direct/live_probe_tranche_three/bridge_forwarding_probe.rs",
        "src/drivers/sdk_direct/live_probe_tranche_three/batch_merkle_probe.rs",
        "src/drivers/sdk_direct/live_probe_tranche_three/performance_smoke_probe.rs",
        "src/drivers/sdk_direct/live_probe_tranche_three/live_probe_support.rs",
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
fn regression_sdk_direct_root_respects_tranche_three_staged_line_budget() {
    let line_count = SDK_DIRECT_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "sdk_direct.rs should stay within the tranche-three staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn regression_sdk_direct_tranche_three_files_stay_within_line_budget() {
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
