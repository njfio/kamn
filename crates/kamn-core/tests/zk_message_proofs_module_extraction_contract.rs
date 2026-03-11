use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/zk_message_proofs.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/zk_message_proofs/planning.rs",
    "src/zk_message_proofs/processor_admission.rs",
    "src/zk_message_proofs/validator_consensus.rs",
    "src/zk_message_proofs/watchdog_projection.rs",
    "src/zk_message_proofs/witness.rs",
    "src/zk_message_proofs/errors.rs",
    "src/zk_message_proofs/tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod planning;",
    "mod processor_admission;",
    "mod validator_consensus;",
    "mod watchdog_projection;",
    "mod witness;",
    "mod errors;",
    "#[cfg(test)]",
    "mod tests;",
];
const MOVED_MARKERS: &[&str] = &[
    "pub fn phase4_baseline_options(",
    "pub fn evaluate_zk_option(",
    "pub fn recommend_phase4_plan(",
    "impl ProcessorProofAdmissionEvaluator",
    "impl ValidatorProofConsensusEvaluator",
    "impl ProofWatchdogProjector",
    "pub fn build_message_witness(",
    "impl fmt::Display for ZkDesignError",
    "mod tests {",
];

#[test]
fn zk_message_proofs_root_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    assert_root_shell_budget(&root);
    assert_required_markers(&root);
    assert_moved_markers_removed(&root);
    assert_module_files_exist_and_fit_budget();
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_root_shell_budget(root: &str) {
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after extraction, found {lines}"
    );
}

fn assert_required_markers(root: &str) {
    for marker in REQUIRED_MARKERS {
        assert!(
            root.contains(marker),
            "missing root module marker: {marker}"
        );
    }
}

fn assert_moved_markers_removed(root: &str) {
    for marker in MOVED_MARKERS {
        assert!(
            !root.contains(marker),
            "moved marker still present in root: {marker}"
        );
    }
}

fn assert_module_files_exist_and_fit_budget() {
    for path in MODULE_FILES {
        let full = repo_path(path);
        assert!(
            full.exists(),
            "missing extracted module: {}",
            full.display()
        );
        let lines = fs::read_to_string(&full)
            .expect("read module")
            .lines()
            .count();
        assert!(
            lines <= MODULE_CAP,
            "extracted module exceeds {MODULE_CAP} lines: {}",
            full.display()
        );
    }
}
