use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_checker(args: &[&str]) -> Output {
    Command::new("python3")
        .arg("scripts/ci/check_workspace_license_policy.py")
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("failed to execute workspace license policy checker")
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed unexpectedly:\n{}",
        output_text(output)
    );
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context} succeeded unexpectedly:\n{}",
        output_text(output)
    );
}

fn write_text(path: &Path, text: &str) {
    fs::write(path, text)
        .unwrap_or_else(|error| panic!("failed to write fixture {}: {error}", path.display()));
}

#[test]
fn unit_workspace_license_policy_checker_accepts_root_policy_file_contract_markers() {
    let output = run_checker(&[
        "--workspace-root",
        ".",
        "--expected-license",
        "Apache-2.0",
        "--license-policy-file",
        "LICENSE",
        "--lane-profile",
        "ci-smoke",
    ]);
    assert_success(&output, "workspace license checker unit baseline");
    let text = output_text(&output);
    assert!(text.contains(
        "reason_taxonomy_version=kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1"
    ));
    assert!(text.contains("reason_codes_csv=none"));
    assert!(text.contains("ci_smoke_local_heavy_boundary_status=verified"));
}

#[test]
fn functional_workspace_license_policy_checker_rejects_root_policy_marker_drift() {
    let root = repo_root();
    let tmp = root.join("target/tmp/workspace-license-policy-functional");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("failed to create temporary fixture directory");

    let drifted_policy = tmp.join("LICENSE.drifted");
    let baseline =
        fs::read_to_string(root.join("LICENSE")).expect("failed to read baseline LICENSE");
    let drifted = baseline.replacen("Version 2.0, January 2004", "Version 1.0, January 2000", 1);
    assert_ne!(
        baseline, drifted,
        "policy fixture mutation must change content"
    );
    write_text(&drifted_policy, &drifted);

    let output = run_checker(&[
        "--workspace-root",
        ".",
        "--expected-license",
        "Apache-2.0",
        "--license-policy-file",
        drifted_policy.to_string_lossy().as_ref(),
        "--lane-profile",
        "ci-smoke",
    ]);
    assert_failure(
        &output,
        "workspace license checker with drifted root policy marker",
    );
    let text = output_text(&output);
    assert!(text.contains("license_policy_marker_mismatch"));
    assert!(text.contains("reason_class=metadata_mismatch"));
}

#[test]
fn integration_workspace_license_policy_ci_tools_command_surface_contains_checker_lane() {
    let script = fs::read_to_string(repo_root().join("scripts/ci/test_ci_tools.sh"))
        .expect("failed to read ci tools script");
    assert!(
        script.contains("bash \"$ROOT_DIR/scripts/ci/test_check_workspace_license_policy.sh\""),
        "ci tools command surface must include workspace license policy checker lane"
    );
}

#[test]
fn regression_workspace_license_policy_checker_rejects_missing_policy_file() {
    let output = run_checker(&[
        "--workspace-root",
        ".",
        "--expected-license",
        "Apache-2.0",
        "--license-policy-file",
        "target/tmp/workspace-license-policy-missing-LICENSE",
        "--lane-profile",
        "ci-smoke",
    ]);
    assert_failure(
        &output,
        "workspace license checker with missing policy file",
    );
    let text = output_text(&output);
    assert!(text.contains("license_policy_file_not_found"));
}

#[test]
fn performance_workspace_license_policy_checker_stays_within_budget() {
    let started = Instant::now();
    let output = run_checker(&[
        "--workspace-root",
        ".",
        "--expected-license",
        "Apache-2.0",
        "--license-policy-file",
        "LICENSE",
        "--lane-profile",
        "ci-smoke",
    ]);
    assert_success(&output, "workspace license checker performance baseline");
    let elapsed_ms = started.elapsed().as_millis();
    assert!(
        elapsed_ms < 5000,
        "workspace license checker exceeded 5s budget: {elapsed_ms}ms"
    );
}
