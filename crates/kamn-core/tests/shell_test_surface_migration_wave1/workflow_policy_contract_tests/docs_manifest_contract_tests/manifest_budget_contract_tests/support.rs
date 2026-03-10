use super::super::super::super::support::*;
use super::MANIFEST_RUNNER_COMMAND;

pub fn assert_manifest_inputs(
    legacy_lane_script: &Path,
    manifest_runner: &Path,
    shared_impl: &Path,
    manifest_file: &Path,
) {
    assert!(
        !legacy_lane_script.exists(),
        "superseded Kolme soft-budget wrapper must be deleted"
    );
    assert!(
        manifest_runner.is_file() && shared_impl.is_file() && manifest_file.is_file(),
        "manifest runner, shared impl, and manifest must exist"
    );
}

pub fn run_manifest_contract_lane(
    manifest_runner: &Path,
    manifest_file: &Path,
    report_file: &Path,
) -> CommandOutput {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(manifest_runner)
                .arg("--manifest")
                .arg(manifest_file)
                .arg("--phase")
                .arg("contract")
                .arg("--")
                .arg("--output-json")
                .arg(report_file)
                .arg("--max-runtime-seconds")
                .arg("120");
            command
        },
        "Kolme soft-budget manifest contract lane",
    )
}

pub fn assert_lane_output(output: &CommandOutput) {
    let output_text = output_text(output);
    assert_contains_all(
        &output_text,
        &[
            "kolme_test_harness_loc_soft_budget_contract_status=pass",
            "kolme_test_harness_loc_soft_budget_contract_go_decision=GO",
            "kolme_test_harness_loc_soft_budget_contract_warn_decision=WARN",
            "kolme_test_harness_loc_soft_budget_contract_fail_decision=NO-GO",
        ],
        "Kolme soft-budget manifest contract lane markers",
    );
}

pub fn assert_manifest_docs_and_report(manifest_file: &Path, report_file: &Path) {
    assert_manifest_runner_docs();
    assert_manifest_report(report_file);
    assert_manifest_dispatch_marker(manifest_file);
}

fn assert_manifest_runner_docs() {
    let strategy_doc = read_text("docs/ci/strategy.md");
    let cost_doc = read_text("docs/ci/ci-cost-and-lane-framework.md");
    assert!(
        strategy_doc.contains(MANIFEST_RUNNER_COMMAND),
        "ci strategy doc missing Kolme soft-budget manifest-runner command marker"
    );
    assert!(
        cost_doc.contains(MANIFEST_RUNNER_COMMAND),
        "ci cost doc missing Kolme soft-budget manifest-runner command marker"
    );
}

fn assert_manifest_report(report_file: &Path) {
    let report_text = fs::read_to_string(report_file)
        .expect("failed to read Kolme soft-budget contract report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\": \"kamn.ci.kolme-test-harness-loc-soft-budget-contract-report.v1\"",
            "\"combined_reason_code_contract\": \"pass\"",
            "\"command_surface_fail_reason_contract\": \"pass\"",
        ],
        "Kolme soft-budget contract report markers",
    );
}

fn assert_manifest_dispatch_marker(manifest_file: &Path) {
    let manifest_text =
        fs::read_to_string(manifest_file).expect("failed to read Kolme soft-budget manifest file");
    assert!(
        manifest_text.contains("kolme_test_harness_loc_soft_budget_contract_lane_impl.sh"),
        "Kolme soft-budget manifest must dispatch shared implementation script"
    );
}
