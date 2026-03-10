use super::*;

#[test]
fn spec_c08_phase_inventory_contains_prd_canonical_order() {
    let phases = all_orchestration_phases();
    let labels: Vec<&str> = phases.iter().map(|phase| phase.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "INFRA_UP",
            "AGENT_DEPLOY",
            "SCENARIO_RUN",
            "EVIDENCE",
            "TEARDOWN"
        ]
    );
}

#[test]
fn spec_c09_run_output_contains_phase_progression_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase_count\":5"));
    assert!(output.contains("\"phases\":[\"INFRA_UP\""));
    assert!(output.contains("\"AGENT_DEPLOY\""));
    assert!(output.contains("\"SCENARIO_RUN\""));
    assert!(output.contains("\"EVIDENCE\""));
    assert!(output.contains("\"TEARDOWN\"]"));
}

#[test]
fn spec_c10_phase_result_status_inventory_is_canonical() {
    let statuses = all_phase_result_statuses();
    let labels: Vec<&str> = statuses.iter().map(|status| status.as_str()).collect();
    assert_eq!(labels, vec!["PASS", "FAIL", "SKIP"]);
}

#[test]
fn spec_c11_run_output_contains_phase_results_required_fields() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase_results\":[{"));
    assert!(output.contains("\"phase\":\"INFRA_UP\""));
    assert!(output.contains("\"status\":\"PASS\""));
    assert!(output.contains("\"started_at\""));
    assert!(output.contains("\"completed_at\""));
    assert!(output.contains("\"details\""));
}

#[test]
fn spec_c12_run_output_contains_infra_and_agent_deploy_placeholders() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase\":\"INFRA_UP\""));
    assert!(output.contains("\"phase\":\"AGENT_DEPLOY\""));
    assert!(output.contains("deterministic placeholder"));
}

#[test]
fn spec_c13_run_output_contains_nested_step_records() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"steps\":[{"));
    assert!(output.contains("\"step\""));
    assert!(output.contains("\"status\""));
    assert!(output.contains("\"detail\""));
}

#[test]
fn spec_c14_infra_up_step_markers_align_with_prd_actions() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("Start PostgreSQL container (docker)"));
    assert!(output.contains("Run Kolme migrations"));
    assert!(output.contains("Verify KAMN Service API health (/healthz)"));
}

#[test]
fn spec_c15_agent_deploy_step_markers_align_with_prd_actions() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("Generate ed25519 key pairs for Alice, Bob, Carol"));
    assert!(output.contains("Register agents via kamn-agent-lib"));
    assert!(output.contains("Record infrastructure evidence"));
}
