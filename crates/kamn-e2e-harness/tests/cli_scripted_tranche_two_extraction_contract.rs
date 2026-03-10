use std::fs;
use std::path::{Path, PathBuf};

const CLI_SCRIPTED_ROOT_SOURCE: &str = include_str!("../src/drivers/cli_scripted.rs");
const CLI_SCRIPTED_TRANCHE_MODULE_FILE: &str = "src/drivers/cli_scripted/live_probe_tranche_two.rs";
const CLI_SCRIPTED_TRANCHE_DIR: &str = "src/drivers/cli_scripted/live_probe_tranche_two";
const ROOT_STAGED_MAX_LINES: usize = 1_200;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_cli_scripted_root_declares_live_probe_tranche_two_module() {
    assert!(
        CLI_SCRIPTED_ROOT_SOURCE.contains("mod live_probe_tranche_two;"),
        "cli_scripted.rs must declare the extracted live_probe_tranche_two module"
    );
}

#[test]
fn regression_cli_scripted_root_removes_s06_through_s10_probe_definitions() {
    for marker in [
        "fn run_live_s06_cli_proof_verification_probe()",
        "fn run_live_s07_cli_replay_protection_probe()",
        "fn run_live_s08_cli_crash_recovery_probe()",
        "fn run_live_s09_cli_transport_failover_probe()",
        "fn run_live_s10_cli_topology_coherence_probe()",
    ] {
        assert!(
            !CLI_SCRIPTED_ROOT_SOURCE.contains(marker),
            "cli_scripted.rs must not keep tranche-two implementation marker: {marker}"
        );
    }
}

#[test]
fn regression_cli_scripted_tranche_two_module_file_exists() {
    let full_path = manifest_dir().join(CLI_SCRIPTED_TRANCHE_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted tranche-two module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_cli_scripted_tranche_two_layout_exists() {
    for relative_path in [
        "src/drivers/cli_scripted/live_probe_tranche_two.rs",
        "src/drivers/cli_scripted/live_probe_tranche_two/message_query_support.rs",
        "src/drivers/cli_scripted/live_probe_tranche_two/proof_replay_probes.rs",
        "src/drivers/cli_scripted/live_probe_tranche_two/recovery_failover_probes.rs",
        "src/drivers/cli_scripted/live_probe_tranche_two/recovery_failover_probes/crash_recovery_probe.rs",
        "src/drivers/cli_scripted/live_probe_tranche_two/recovery_failover_probes/transport_failover_probe.rs",
        "src/drivers/cli_scripted/live_probe_tranche_two/topology_coherence_probe.rs",
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted cli_scripted tranche-two path missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_cli_scripted_root_respects_tranche_two_staged_line_budget() {
    let line_count = CLI_SCRIPTED_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "cli_scripted.rs should stay within the tranche-two staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn regression_cli_scripted_tranche_two_files_stay_within_line_budget() {
    let base_dir = manifest_dir().join(CLI_SCRIPTED_TRANCHE_DIR);
    let extracted_files = collected_rs_files(&base_dir);
    assert!(
        !extracted_files.is_empty(),
        "expected extracted cli_scripted tranche-two files under {}",
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
        "extracted cli_scripted tranche-two files exceed {EXTRACTED_MAX_LINES} LOC: {}",
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
    for entry in fs::read_dir(dir).expect("extracted cli_scripted tranche-two dir should be readable") {
        let entry = entry.expect("extracted cli_scripted tranche-two entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}
