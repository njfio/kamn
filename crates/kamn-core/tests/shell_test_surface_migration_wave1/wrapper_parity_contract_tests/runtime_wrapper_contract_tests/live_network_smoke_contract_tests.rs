use super::super::super::support::*;

#[test]
fn spec_c18_live_network_smoke_contract_lane_wrapper_parity() {
    let (contract_lane, shared_contract, manifest_file, dispatcher, smoke_runner) =
        live_network_smoke_paths();
    assert_live_network_smoke_assets(
        &contract_lane,
        &shared_contract,
        &manifest_file,
        &dispatcher,
        &smoke_runner,
    );
    assert_live_network_shared_contract(&shared_contract);
    assert_live_network_lane_success(&contract_lane);
    assert_dispatcher_manifest_resolution(
        &dispatcher,
        "run_live_network_smoke_contract_lane.sh",
        &manifest_file,
    );
}

fn live_network_smoke_paths() -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    (
        repo_path("scripts/runtime/run_live_network_smoke_contract_lane.sh"),
        repo_path("scripts/runtime/live_network_smoke_contract_lane_contract.sh"),
        repo_path("scripts/framework/manifests/runtime_live_network_smoke_contract_lane.json"),
        repo_path("scripts/framework/run_non_kolme_contract_lane_dispatch.sh"),
        repo_path("scripts/runtime/run_live_network_smoke_lane.sh"),
    )
}

fn assert_live_network_smoke_assets(
    contract_lane: &Path,
    shared_contract: &Path,
    manifest_file: &Path,
    dispatcher: &Path,
    smoke_runner: &Path,
) {
    assert!(
        contract_lane.is_file()
            && shared_contract.is_file()
            && manifest_file.is_file()
            && dispatcher.is_file()
            && smoke_runner.is_file(),
        "live-network smoke lane assets must exist"
    );
}

fn assert_live_network_shared_contract(shared_contract: &Path) {
    let shared_contract_text =
        fs::read_to_string(shared_contract).expect("failed to read live-network shared contract");
    assert!(
        shared_contract_text.contains("run_live_network_smoke_lane.sh"),
        "live-network shared contract must execute smoke runner"
    );
}

fn assert_live_network_lane_success(contract_lane: &Path) {
    let output = run_command(
        {
            let mut command = Command::new("bash");
            command.arg(contract_lane);
            command
        },
        "live-network smoke contract lane execution",
    );
    assert_success(&output, "live-network smoke contract lane execution");
    assert!(
        output_text(&output).contains("live-network smoke contract lane tests passed."),
        "live-network smoke contract lane must emit deterministic success marker"
    );
}

fn assert_dispatcher_manifest_resolution(dispatcher: &Path, lane_wrapper: &str, manifest: &Path) {
    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(dispatcher)
                .arg("--lane-wrapper")
                .arg(lane_wrapper)
                .arg("--resolve-manifest-path");
            command
        },
        "live-network smoke manifest resolution",
    );
    assert_success(&output, "live-network smoke manifest resolution");
    assert_eq!(
        output_text(&output).trim(),
        manifest
            .canonicalize()
            .expect("failed to canonicalize expected live-network manifest path")
            .to_string_lossy(),
        "live-network smoke wrapper must resolve runtime manifest via dispatcher"
    );
}
