use std::fs;
use std::path::{Path, PathBuf};

const CLI_SCRIPTED_ROOT_SOURCE: &str = include_str!("../src/drivers/cli_scripted.rs");
const CLI_SCRIPTED_TRANCHE_MODULE_FILE: &str =
    "src/drivers/cli_scripted/live_probe_tranche_three.rs";
const CLI_SCRIPTED_TRANCHE_DIR: &str = "src/drivers/cli_scripted/live_probe_tranche_three";
const ROOT_STAGED_MAX_LINES: usize = 400;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_cli_scripted_root_declares_live_probe_tranche_three_module() {
    assert!(
        CLI_SCRIPTED_ROOT_SOURCE.contains("mod live_probe_tranche_three;"),
        "cli_scripted.rs must declare the extracted live_probe_tranche_three module"
    );
}

#[test]
fn regression_cli_scripted_root_removes_s11_through_s15_probe_definitions() {
    for marker in [
        "fn run_live_s11_cli_signer_rotation_probe()",
        "fn run_live_s12_cli_retention_deletion_probe()",
        "fn run_live_s13_cli_bridge_forwarding_probe()",
        "fn run_live_s14_cli_batch_merkle_probe()",
        "fn run_live_s15_cli_performance_smoke_probe()",
    ] {
        assert!(
            !CLI_SCRIPTED_ROOT_SOURCE.contains(marker),
            "cli_scripted.rs must not keep tranche-three implementation marker: {marker}"
        );
    }
}

#[test]
fn regression_cli_scripted_tranche_three_module_file_exists() {
    let full_path = manifest_dir().join(CLI_SCRIPTED_TRANCHE_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted tranche-three module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_cli_scripted_tranche_three_layout_exists() {
    for relative_path in [
        "src/drivers/cli_scripted/live_probe_tranche_three.rs",
        "src/drivers/cli_scripted/live_probe_tranche_three/batch_merkle_probe.rs",
        "src/drivers/cli_scripted/live_probe_tranche_three/bridge_forwarding_probe.rs",
        "src/drivers/cli_scripted/live_probe_tranche_three/live_probe_support.rs",
        "src/drivers/cli_scripted/live_probe_tranche_three/performance_smoke_probe.rs",
        "src/drivers/cli_scripted/live_probe_tranche_three/retention_deletion_probe.rs",
        "src/drivers/cli_scripted/live_probe_tranche_three/signer_rotation_probe.rs",
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted cli_scripted tranche-three path missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_cli_scripted_root_respects_tranche_three_staged_line_budget() {
    let line_count = CLI_SCRIPTED_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "cli_scripted.rs should stay within the tranche-three staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn regression_cli_scripted_tranche_three_files_stay_within_line_budget() {
    let base_dir = manifest_dir().join(CLI_SCRIPTED_TRANCHE_DIR);
    let extracted_files = collected_rs_files(&base_dir);
    assert!(
        !extracted_files.is_empty(),
        "expected extracted cli_scripted tranche-three files under {}",
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
        "extracted cli_scripted tranche-three files exceed {EXTRACTED_MAX_LINES} LOC: {}",
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
    for entry in
        fs::read_dir(dir).expect("extracted cli_scripted tranche-three dir should be readable")
    {
        let entry = entry.expect("extracted cli_scripted tranche-three entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}
