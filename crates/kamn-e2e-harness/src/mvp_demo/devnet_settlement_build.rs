use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) const TARGET_DIR: &str = "target/mvp-demo-proof";

pub(super) fn build_node_binary(run_dir: &Path) -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["build", "-p", "kamn-node", "--bin", "kamn-node"])
        .current_dir(repo_root())
        .env("CARGO_TARGET_DIR", TARGET_DIR)
        .output()
        .map_err(|error| format!("failed to build kamn-node for MVP demo: {error}"))?;
    write_output_log(run_dir, "devnet-settlement-build-output.txt", &output)?;
    if output.status.success() {
        return Ok(());
    }
    Err("failed to build kamn-node for MVP demo".to_owned())
}

pub(super) fn node_binary_path() -> PathBuf {
    repo_root()
        .join(TARGET_DIR)
        .join("debug")
        .join(format!("kamn-node{}", std::env::consts::EXE_SUFFIX))
}

pub(super) fn write_output_log(
    run_dir: &Path,
    name: &str,
    output: &std::process::Output,
) -> Result<(), String> {
    let mut content = String::from("--- stdout ---\n");
    content.push_str(&String::from_utf8_lossy(output.stdout.as_slice()));
    content.push_str("\n--- stderr ---\n");
    content.push_str(&String::from_utf8_lossy(output.stderr.as_slice()));
    std::fs::write(run_dir.join("proof").join(name), content)
        .map_err(|error| format!("failed to write {name}: {error}"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
