use std::fs;
use std::path::{Path, PathBuf};

const CLI_SCRIPTED_ROOT_SOURCE: &str = include_str!("../src/drivers/cli_scripted.rs");
const CLI_SCRIPTED_TRANCHE_MODULE_FILE: &str = "src/drivers/cli_scripted/live_probe_tranche_one.rs";
const CLI_SCRIPTED_TRANCHE_DIR: &str = "src/drivers/cli_scripted/live_probe_tranche_one";
const ROOT_STAGED_MAX_LINES: usize = 1_700;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_cli_scripted_root_declares_live_probe_tranche_module() {
    assert!(
        CLI_SCRIPTED_ROOT_SOURCE.contains("mod live_probe_tranche_one;"),
        "cli_scripted.rs must declare the extracted live_probe_tranche_one module"
    );
}

#[test]
fn regression_cli_scripted_root_removes_s01_through_s05_probe_definitions() {
    for marker in [
        "fn run_live_s01_cli_health_probe()",
        "fn run_live_s02_cli_direct_message_probe()",
        "fn run_live_s03_cli_group_channel_probe()",
        "fn run_live_s04_cli_task_lifecycle_probe()",
        "fn run_live_s05_cli_escrow_settlement_probe()",
    ] {
        assert!(
            !CLI_SCRIPTED_ROOT_SOURCE.contains(marker),
            "cli_scripted.rs must not keep tranche-one implementation marker: {marker}"
        );
    }
}

#[test]
fn regression_cli_scripted_tranche_module_file_exists() {
    let full_path = manifest_dir().join(CLI_SCRIPTED_TRANCHE_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted tranche module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_cli_scripted_tranche_layout_exists() {
    for relative_path in [
        "src/drivers/cli_scripted/live_probe_tranche_one.rs",
        "src/drivers/cli_scripted/live_probe_tranche_one/discovery_direct_message_probes.rs",
        "src/drivers/cli_scripted/live_probe_tranche_one/channel_task_probes.rs",
        "src/drivers/cli_scripted/live_probe_tranche_one/escrow_probe_support.rs",
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted cli_scripted tranche path missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_cli_scripted_root_respects_tranche_one_staged_line_budget() {
    let line_count = CLI_SCRIPTED_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "cli_scripted.rs should stay within the tranche-one staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn regression_cli_scripted_tranche_files_stay_within_line_budget() {
    let base_dir = manifest_dir().join(CLI_SCRIPTED_TRANCHE_DIR);
    let extracted_files = collected_rs_files(&base_dir);
    assert!(
        !extracted_files.is_empty(),
        "expected extracted cli_scripted tranche files under {}",
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
        "extracted cli_scripted tranche files exceed {EXTRACTED_MAX_LINES} LOC: {}",
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
    for entry in fs::read_dir(dir).expect("extracted cli_scripted tranche dir should be readable") {
        let entry = entry.expect("extracted cli_scripted tranche entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}
