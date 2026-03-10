use super::super::super::support::*;
#[path = "manifest_budget_contract_tests/support.rs"]
mod support;
use support::*;

const MANIFEST_RUNNER_COMMAND: &str = "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_kolme_test_harness_loc_soft_budget_contract_lane.json --phase contract --output-json /tmp/kolme-test-harness-loc-soft-budget-contract-report.json";

#[test]
fn spec_c06_generate_test_harness_loc_report_contract() {
    let generator = repo_path("scripts/ci/generate_test_harness_loc_report.sh");
    assert!(
        generator.is_file(),
        "test-harness LOC report generator script must exist"
    );

    let tmp = TempDir::new("test-harness-loc-report");
    let scripts_root = tmp.path().join("scripts");
    write_harness_scripts(&scripts_root);
    let report_file = tmp.path().join("test-harness-loc-report.json");

    let output = run_loc_report(&generator, &scripts_root, &report_file);
    assert_success(&output, "test-harness LOC report generation");
    assert_loc_report_output(&output);
    assert_loc_report_json(&report_file);
}

#[test]
fn spec_c09_kolme_test_harness_loc_soft_budget_manifest_contract_lane() {
    let legacy_lane_script =
        repo_path("scripts/ci/run_kolme_test_harness_loc_soft_budget_contract_lane.sh");
    let manifest_runner = repo_path("scripts/framework/run_manifest_lane.sh");
    let shared_impl =
        repo_path("scripts/ci/kolme_test_harness_loc_soft_budget_contract_lane_impl.sh");
    let manifest_file = repo_path(
        "scripts/framework/manifests/ci_kolme_test_harness_loc_soft_budget_contract_lane.json",
    );

    assert_manifest_inputs(
        &legacy_lane_script,
        &manifest_runner,
        &shared_impl,
        &manifest_file,
    );
    let tmp = TempDir::new("kolme-soft-budget-contract");
    let report_file = tmp
        .path()
        .join("kolme-test-harness-soft-budget-contract-report.json");
    let output = run_manifest_contract_lane(&manifest_runner, &manifest_file, &report_file);
    assert_success(&output, "Kolme soft-budget manifest contract lane");
    assert_lane_output(&output);
    assert_manifest_docs_and_report(&manifest_file, &report_file);
}

fn write_harness_scripts(scripts_root: &Path) {
    fs::create_dir_all(scripts_root.join("ci")).expect("failed to create ci script fixture root");
    fs::create_dir_all(scripts_root.join("sdk")).expect("failed to create sdk script fixture root");
    fs::write(
        scripts_root.join("ci/test_alpha.sh"),
        "#!/usr/bin/env bash\necho \"alpha\"\n",
    )
    .expect("failed to write ci/test_alpha.sh");
    fs::write(
        scripts_root.join("sdk/test_beta.sh"),
        "#!/usr/bin/env bash\necho \"beta\"\n",
    )
    .expect("failed to write sdk/test_beta.sh");
    fs::write(
        scripts_root.join("sdk/run_non_harness.sh"),
        "#!/usr/bin/env bash\necho \"ignore\"\n",
    )
    .expect("failed to write sdk/run_non_harness.sh");
}

fn run_loc_report(generator: &Path, scripts_root: &Path, report_file: &Path) -> CommandOutput {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(generator)
                .arg("--scripts-root")
                .arg(scripts_root)
                .arg("--output-json")
                .arg(report_file);
            command
        },
        "test-harness LOC report generation",
    )
}

fn assert_loc_report_output(output: &CommandOutput) {
    let output_text = output_text(output);
    assert_contains_all(
        &output_text,
        &[
            "status=ok",
            "harness_script_count=2",
            "harness_shell_line_total=4",
            "report_file=",
        ],
        "test-harness LOC report output markers",
    );
}

fn assert_loc_report_json(report_file: &Path) {
    let report_text =
        fs::read_to_string(report_file).expect("failed to read generated LOC report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\": \"kamn.ci.test-harness-loc-report.v1\"",
            "\"harness_script_count\": 2",
            "\"harness_shell_line_total\": 4",
            "\"domains\"",
            "\"harness_scripts\"",
        ],
        "test-harness LOC report JSON markers",
    );
}
