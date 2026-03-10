use super::super::super::support::*;

pub fn assert_coverage_guided_deep_assets(
    fast_lane: &Path,
    deep_lane: &Path,
    deep_impl: &Path,
    deep_manifest: &Path,
    dispatcher: &Path,
) {
    assert!(
        fast_lane.is_file()
            && deep_lane.is_file()
            && deep_impl.is_file()
            && deep_manifest.is_file()
            && dispatcher.is_file(),
        "coverage-guided deep lane assets must exist"
    );
}

pub fn assert_coverage_guided_deep_impl(deep_impl: &Path) {
    let deep_impl_text =
        fs::read_to_string(deep_impl).expect("failed to read coverage-guided deep lane impl");
    assert_contains_all(
        &deep_impl_text,
        &[
            "run_input_mutation_coverage_guided_contract_lane.sh",
            "performance_input_mutation_coverage_guided_deep_lane_stress -- --ignored",
            "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS",
        ],
        "coverage-guided deep lane implementation markers",
    );
}

pub fn assert_dispatcher_manifest_resolution(
    dispatcher: &Path,
    lane_wrapper: &str,
    manifest: &Path,
    label: &str,
) {
    let output = run_manifest_resolution(dispatcher, lane_wrapper, label);
    assert_success(&output, &format!("{label} manifest resolution"));
    assert_eq!(
        output_text(&output).trim(),
        expected_manifest_path(manifest),
        "{label} wrapper must resolve runtime manifest via dispatcher"
    );
}

fn run_manifest_resolution(dispatcher: &Path, lane_wrapper: &str, label: &str) -> CommandOutput {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(dispatcher)
                .arg("--lane-wrapper")
                .arg(lane_wrapper)
                .arg("--resolve-manifest-path");
            command
        },
        &format!("{label} manifest resolution"),
    )
}

fn expected_manifest_path(manifest: &Path) -> String {
    manifest
        .canonicalize()
        .expect("failed to canonicalize expected manifest path")
        .to_string_lossy()
        .into_owned()
}

pub fn assert_deep_manifest_dispatch(deep_manifest: &Path) {
    let deep_manifest_text =
        fs::read_to_string(deep_manifest).expect("failed to read coverage-guided deep manifest");
    assert!(
        deep_manifest_text.contains("run_input_mutation_coverage_guided_deep_lane_impl.sh"),
        "coverage-guided deep manifest must dispatch deep-lane implementation"
    );
}

pub fn assert_deep_lane_success(deep_lane: &Path) {
    let output = run_command(
        {
            let mut command = Command::new("bash");
            command.arg(deep_lane);
            command
        },
        "coverage-guided deep lane execution",
    );
    assert_success(&output, "coverage-guided deep lane execution");
    assert!(
        output_text(&output)
            .contains("runtime input mutation coverage-guided deep lane tests passed."),
        "coverage-guided deep lane must emit deterministic success marker"
    );
}
