use super::super::support::*;
#[path = "coverage_guided_wrapper_contract_tests/support.rs"]
mod support;
use support::*;

#[test]
fn spec_c16_input_mutation_coverage_guided_contract_lane_wrapper_parity() {
    let (contract_lane, shared_contract, manifest_file) = coverage_guided_contract_paths();
    assert_coverage_guided_assets(&contract_lane, &shared_contract, &manifest_file);
    assert_coverage_guided_shared_contract(&shared_contract);

    let tmp = TempDir::new("coverage-guided-contract-lane");
    let report_file = tmp
        .path()
        .join("input-mutation-coverage-guided-contract-report.json");
    let lane_output = run_coverage_guided_lane(&contract_lane, &report_file, None, "default");
    assert_coverage_guided_lane_output(&lane_output, "default target");
    assert_coverage_guided_report(&report_file);

    let envelope_report = tmp
        .path()
        .join("input-mutation-coverage-guided-envelope-report.json");
    assert_targeted_lane_success(&contract_lane, &envelope_report, "envelope");
    let did_report = tmp
        .path()
        .join("input-mutation-coverage-guided-did-report.json");
    assert_targeted_lane_success(&contract_lane, &did_report, "did");
}

#[test]
fn spec_c17_input_mutation_coverage_guided_deep_lane_wrapper_parity() {
    let (fast_lane, deep_lane, deep_impl, deep_manifest, dispatcher) = coverage_guided_deep_paths();
    assert_coverage_guided_deep_assets(
        &fast_lane,
        &deep_lane,
        &deep_impl,
        &deep_manifest,
        &dispatcher,
    );
    assert_coverage_guided_deep_impl(&deep_impl);
    assert_dispatcher_manifest_resolution(
        &dispatcher,
        "run_input_mutation_coverage_guided_deep_lane.sh",
        &deep_manifest,
        "coverage-guided deep",
    );
    assert_deep_manifest_dispatch(&deep_manifest);
    assert_deep_lane_success(&deep_lane);
}

fn coverage_guided_contract_paths() -> (PathBuf, PathBuf, PathBuf) {
    (
        repo_path("scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh"),
        repo_path("scripts/runtime/input_mutation_coverage_guided_contract_lane_contract.sh"),
        repo_path(
            "scripts/framework/manifests/runtime_input_mutation_coverage_guided_contract_lane.json",
        ),
    )
}

fn coverage_guided_deep_paths() -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    (
        repo_path("scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh"),
        repo_path("scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh"),
        repo_path("scripts/runtime/run_input_mutation_coverage_guided_deep_lane_impl.sh"),
        repo_path(
            "scripts/framework/manifests/runtime_input_mutation_coverage_guided_deep_lane.json",
        ),
        repo_path("scripts/framework/run_non_kolme_contract_lane_dispatch.sh"),
    )
}

fn assert_coverage_guided_assets(
    contract_lane: &Path,
    shared_contract: &Path,
    manifest_file: &Path,
) {
    assert!(
        contract_lane.is_file() && shared_contract.is_file() && manifest_file.is_file(),
        "coverage-guided input mutation lane assets must exist"
    );
}

fn assert_coverage_guided_shared_contract(shared_contract: &Path) {
    let shared_contract_text = fs::read_to_string(shared_contract)
        .expect("failed to read shared coverage-guided contract");
    assert_contains_all(
        &shared_contract_text,
        &[
            "unit_input_mutation_coverage_guided_envelope_seed_corpus_covers_boundary_classes",
            "unit_input_mutation_coverage_guided_did_seed_corpus_covers_boundary_classes",
            "minimal_failing_seed_prefix",
            "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS",
            "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS:-600",
            "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_CASE_MAX_SECONDS:-120",
            "cargo test -p kamn-core --test input_mutation_coverage_guided --no-run",
            "timeout \"$prebuild_timeout_seconds\" cargo test",
            "timeout \"$case_timeout_seconds\" cargo test",
            "runtime input mutation coverage-guided case timed out",
            "run_case \"$case_name\"",
        ],
        "coverage-guided shared contract markers",
    );
}

fn run_coverage_guided_lane(
    contract_lane: &Path,
    report_file: &Path,
    target: Option<&str>,
    label: &str,
) -> std::process::Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command.arg(contract_lane);
            command.env(
                "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS",
                "600",
            );
            if let Some(target) = target {
                command.arg("--target").arg(target);
            }
            command.arg("--output-json").arg(report_file);
            command
        },
        &format!("coverage-guided contract lane {label}"),
    )
}

fn assert_coverage_guided_lane_output(output: &std::process::Output, label: &str) {
    assert_success(output, &format!("coverage-guided contract lane {label}"));
    assert_contains_all(
        &output_text(output),
        &[
            "runtime input mutation coverage-guided contract lane tests passed.",
            "runtime_input_mutation_coverage_guided_contract_report=",
        ],
        "coverage-guided contract lane success markers",
    );
}

fn assert_coverage_guided_report(report_file: &Path) {
    let report_text = fs::read_to_string(report_file)
        .expect("failed to read coverage-guided contract lane report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\":\"kamn.runtime.input-mutation-coverage-guided-contract-report.v1\"",
            "\"status\":\"pass\"",
            "\"target\":\"all\"",
            "\"replay_schema_version\":\"kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1\"",
            "\"replay_artifact_key\":\"input_mutation_coverage_guided_replay:v1\"",
            "\"minimizer\":\"minimal_failing_seed_prefix\"",
        ],
        "coverage-guided contract lane report markers",
    );
}

fn assert_targeted_lane_success(contract_lane: &Path, report_file: &Path, target: &str) {
    let output = run_coverage_guided_lane(contract_lane, report_file, Some(target), target);
    assert_success(
        &output,
        &format!("coverage-guided contract lane {target} target"),
    );
}
