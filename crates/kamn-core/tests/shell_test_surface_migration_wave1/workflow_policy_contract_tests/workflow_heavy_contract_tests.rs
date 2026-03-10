use super::super::support::*;

#[test]
fn spec_c04_workflow_kolme_heavy_exclusion_checker_contract() {
    let fixtures = workflow_heavy_fixtures();
    assert_required_files(&fixtures);
    let tmp = TempDir::new("workflow-kolme-heavy");
    assert_safe_fixture(&fixtures, tmp.path());
    assert_fast_workflow(&fixtures, tmp.path());
    assert_reason_cases(&fixtures);
}

struct WorkflowHeavyFixtures {
    checker: PathBuf,
    selector: PathBuf,
    ci_tools: PathBuf,
    safe_fixture: PathBuf,
    missing_input_fixture: PathBuf,
    forced_true_fixture: PathBuf,
    missing_local_heavy_fixture: PathBuf,
}

fn workflow_heavy_fixtures() -> WorkflowHeavyFixtures {
    WorkflowHeavyFixtures {
        checker: repo_path("scripts/ci/check_workflow_kolme_heavy_exclusion_policy.py"),
        selector: repo_path("scripts/ci/select_targets.sh"),
        ci_tools: repo_path(CI_TOOLS_SCRIPT),
        safe_fixture: repo_path("fixtures/ci/workflow_kolme_heavy_policy_safe.yml"),
        missing_input_fixture: repo_path(
            "fixtures/ci/workflow_kolme_heavy_policy_unsafe_missing_input.yml",
        ),
        forced_true_fixture: repo_path(
            "fixtures/ci/workflow_kolme_heavy_policy_unsafe_forced_true.yml",
        ),
        missing_local_heavy_fixture: repo_path(
            "fixtures/ci/workflow_kolme_heavy_policy_unsafe_missing_local_heavy_command.yml",
        ),
    }
}

fn assert_required_files(fixtures: &WorkflowHeavyFixtures) {
    assert!(
        fixtures.checker.is_file()
            && fixtures.selector.is_file()
            && fixtures.ci_tools.is_file()
            && fixtures.safe_fixture.is_file()
            && fixtures.missing_input_fixture.is_file()
            && fixtures.forced_true_fixture.is_file()
            && fixtures.missing_local_heavy_fixture.is_file(),
        "workflow kolmen heavy exclusion checker fixtures must exist"
    );
}

fn build_checker_command(
    checker: &Path,
    workflow_file: &Path,
    selector: Option<&Path>,
    ci_tools: Option<&Path>,
    output_json: Option<&Path>,
) -> Command {
    let mut command = Command::new("python3");
    command
        .arg(checker)
        .arg("--workflow-file")
        .arg(workflow_file);
    append_optional_arg(&mut command, "--selector-file", selector);
    append_optional_arg(&mut command, "--ci-tools-file", ci_tools);
    append_optional_arg(&mut command, "--output-json", output_json);
    command
}

fn append_optional_arg(command: &mut Command, flag: &str, value: Option<&Path>) {
    if let Some(value) = value {
        command.arg(flag).arg(value);
    }
}

fn run_checker(
    checker: &Path,
    workflow_file: &Path,
    selector: Option<&Path>,
    ci_tools: Option<&Path>,
    output_json: Option<&Path>,
    label: &str,
) -> CommandOutput {
    let command = build_checker_command(checker, workflow_file, selector, ci_tools, output_json);
    run_command(command, label)
}

fn assert_safe_fixture(fixtures: &WorkflowHeavyFixtures, tmp_root: &Path) {
    let report = tmp_root.join("safe-report.json");
    let output = run_checker(
        &fixtures.checker,
        &fixtures.safe_fixture,
        Some(&fixtures.selector),
        Some(&fixtures.ci_tools),
        Some(&report),
        "workflow heavy checker safe fixture",
    );
    assert_success(&output, "workflow heavy checker safe fixture");
    assert_safe_fixture_output(&output);
    assert_safe_fixture_report(&report);
}

fn assert_safe_fixture_output(output: &CommandOutput) {
    assert_contains_all(
        &output_text(output),
        &["status=pass", "reason_codes=none"],
        "workflow heavy checker safe fixture markers",
    );
}

fn assert_safe_fixture_report(report: &Path) {
    let report_text =
        fs::read_to_string(report).expect("failed to read safe workflow checker report");
    assert!(
        report_text.contains("\"final_decision\": \"GO\""),
        "safe workflow checker report must contain GO decision"
    );
}

fn assert_fast_workflow(fixtures: &WorkflowHeavyFixtures, tmp_root: &Path) {
    let report = tmp_root.join("fast-report.json");
    let output = run_checker(
        &fixtures.checker,
        &repo_path(FAST_WORKFLOW),
        Some(&fixtures.selector),
        Some(&fixtures.ci_tools),
        Some(&report),
        "workflow heavy checker ci-fast-gate workflow",
    );
    assert_success(&output, "workflow heavy checker ci-fast-gate workflow");
    assert_contains_all(
        &output_text(&output),
        &["status=pass", "reason_codes=none"],
        "workflow heavy checker ci-fast-gate markers",
    );
}

fn assert_reason_cases(fixtures: &WorkflowHeavyFixtures) {
    assert_reason_case(
        &fixtures.checker,
        &fixtures.missing_input_fixture,
        "workflow_dispatch_input_missing",
        "workflow heavy checker missing-input fixture",
    );
    assert_reason_case(
        &fixtures.checker,
        &fixtures.forced_true_fixture,
        "selector_opt_in_env_forced_true_literal",
        "workflow heavy checker forced-true fixture",
    );
    assert_reason_case(
        &fixtures.checker,
        &fixtures.missing_local_heavy_fixture,
        "local_heavy_lane_commands_missing",
        "workflow heavy checker missing-local-heavy fixture",
    );
}

fn assert_reason_case(checker: &Path, fixture: &Path, reason_code: &str, label: &str) {
    let output = run_checker(checker, fixture, None, None, None, label);
    assert_failure(&output, label);
    assert!(
        output_text(&output).contains(reason_code),
        "{label} must emit deterministic reason code"
    );
}
