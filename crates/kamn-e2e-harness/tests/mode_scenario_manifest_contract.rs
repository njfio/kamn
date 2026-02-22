use kamn_e2e_harness::evidence::{
    EvidenceInfrastructure, EvidenceManifest, EvidenceSummary, ScenarioResult,
    MANIFEST_SCHEMA_VERSION,
};
use kamn_e2e_harness::scenarios::all_scenarios;
use kamn_e2e_harness::verify::{generate_verification_report, generate_verification_report_json};
use kamn_e2e_harness::{all_execution_modes, build_core_run_plan, ExecutionMode};

#[test]
fn spec_c03_execution_mode_registry_contains_required_modes() {
    let modes = all_execution_modes();
    assert_eq!(modes.len(), 4);
    assert!(modes.contains(&ExecutionMode::SdkDirect));
    assert!(modes.contains(&ExecutionMode::CliScripted));
    assert!(modes.contains(&ExecutionMode::McpTau));
    assert!(modes.contains(&ExecutionMode::McpAny));
}

#[test]
fn spec_c04_scenario_registry_contains_full_prd_matrix() {
    let scenarios = all_scenarios();
    let expected = [
        ("S-01", "Agent Discovery & Identity", "P0"),
        ("S-02", "Direct Message Round-Trip", "P0"),
        ("S-03", "Group Channel Messaging", "P0"),
        ("S-04", "Task Lifecycle (Full)", "P0"),
        ("S-05", "Escrow Settlement", "P0"),
        ("S-06", "Kolme Proof Verification", "P0"),
        ("S-07", "Message Replay Protection", "P1"),
        ("S-08", "Node Crash Recovery", "P1"),
        ("S-09", "Transport Failover", "P1"),
        ("S-10", "Multi-Node Topology Coherence", "P1"),
        ("S-11", "Signer Key Rotation", "P2"),
        ("S-12", "Content Retention & Deletion", "P2"),
        ("S-13", "Bridge Message Forwarding", "P2"),
        ("S-14", "Batch Merkle Anchoring", "P2"),
        ("S-15", "Performance Smoke", "P2"),
    ];
    assert_eq!(scenarios.len(), expected.len());
    for ((id, name, priority), actual) in expected.iter().zip(&scenarios) {
        assert_eq!(actual.id, *id);
        assert_eq!(actual.name, *name);
        assert_eq!(actual.priority, *priority);
    }
}

#[test]
fn spec_c05_default_run_plan_schedules_all_prd_scenarios() {
    let plan = build_core_run_plan(ExecutionMode::SdkDirect);
    assert_eq!(plan.scenarios.len(), 15);
    assert_eq!(
        plan.scenarios.first().map(|scenario| scenario.id),
        Some("S-01")
    );
    assert_eq!(
        plan.scenarios.last().map(|scenario| scenario.id),
        Some("S-15")
    );
}

#[test]
fn spec_c06_manifest_schema_version_marker_is_stable() {
    assert_eq!(MANIFEST_SCHEMA_VERSION, "kamn.e2e.evidence-manifest.v3");
}

#[test]
fn spec_c07_manifest_model_includes_prd_section_8_2_markers() {
    let manifest = EvidenceManifest::new(
        "e2e-20260221-143052-a7b3c".to_owned(),
        "2026-02-21T14:30:52Z".to_owned(),
        "2026-02-21T14:35:12Z".to_owned(),
        260,
        ExecutionMode::SdkDirect,
        EvidenceInfrastructure {
            kolme_version: "0.x.y".to_owned(),
            kamn_version: "0.1.0".to_owned(),
            kamn_commit: "49efe252".to_owned(),
            kamn_agent_lib_version: "0.1.0".to_owned(),
            agent_runtime: ExecutionMode::SdkDirect.as_str().to_owned(),
            node_count: 3,
            agent_count: 3,
            storage_backend: "sqlite+postgres".to_owned(),
        },
        vec![ScenarioResult {
            id: "S-01".to_owned(),
            name: "Agent Discovery & Identity".to_owned(),
            status: "PASS".to_owned(),
            duration_seconds: 12,
            evidence_files: vec!["s01-agent-discovery/*.json".to_owned()],
            verifiable_outputs: 4,
        }],
        EvidenceSummary {
            total_scenarios: 15,
            passed: 13,
            failed: 1,
            skipped: 1,
            kolme_blocks_produced: 47,
            messages_exchanged: 128,
            proofs_anchored: 47,
            proofs_verified: 47,
        },
    );

    assert_eq!(manifest.schema_version, "kamn.e2e.evidence-manifest.v3");
    assert_eq!(manifest.run_id, "e2e-20260221-143052-a7b3c");
    assert_eq!(manifest.infrastructure.node_count, 3);
    assert_eq!(manifest.summary.total_scenarios, 15);
    assert_eq!(manifest.scenarios[0].evidence_files.len(), 1);
}

#[test]
fn spec_c08_verifier_report_contains_deterministic_check_markers() {
    let manifest = r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-20260221-143052-a7b3c","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[{"id":"S-01","name":"Agent Discovery & Identity","status":"PASS","duration_seconds":12,"evidence_files":["s01-agent-discovery/*.json"],"verifiable_outputs":4}],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#;

    let report = generate_verification_report(manifest).expect("report should build");
    assert_eq!(report.schema_check.status, "PASS");
    assert_eq!(report.proof_check.status, "PASS");
    assert_eq!(report.chain_check.status, "PASS");
    assert_eq!(report.content_check.status, "PASS");

    let report_json = generate_verification_report_json(manifest).expect("json should render");
    assert!(report_json.contains("\"schema_check\""));
    assert!(report_json.contains("\"proof_check\""));
    assert!(report_json.contains("\"chain_check\""));
    assert!(report_json.contains("\"content_check\""));

    let second = generate_verification_report_json(manifest).expect("json should render");
    assert_eq!(report_json, second);
}

#[test]
fn spec_c09_scenario_registry_exposes_non_empty_contract_fields() {
    let scenarios = all_scenarios();
    for scenario in scenarios {
        assert!(
            !scenario.steps.is_empty(),
            "scenario {} should include steps",
            scenario.id
        );
        assert!(
            !scenario.verifiable_outputs.is_empty(),
            "scenario {} should include verifiable outputs",
            scenario.id
        );
        assert!(
            !scenario.pass_criteria.is_empty(),
            "scenario {} should include pass criteria",
            scenario.id
        );
    }
}

#[test]
fn spec_c10_p0_scenarios_include_prd_contract_markers() {
    let scenarios = all_scenarios();
    let p0 = scenarios
        .into_iter()
        .filter(|scenario| scenario.priority == "P0")
        .collect::<Vec<_>>();
    assert_eq!(p0.len(), 6);

    let s01 = p0
        .iter()
        .find(|scenario| scenario.id == "S-01")
        .expect("S-01 should exist");
    assert!(
        s01.steps
            .iter()
            .any(|step| step.contains("register") || step.contains("Register")),
        "S-01 should include registration step marker"
    );
    assert!(
        s01.verifiable_outputs
            .iter()
            .any(|entry| entry.contains("alice_registration.json")),
        "S-01 should include PRD verifiable output marker"
    );

    let s04 = p0
        .iter()
        .find(|scenario| scenario.id == "S-04")
        .expect("S-04 should exist");
    assert!(
        s04.steps
            .iter()
            .any(|step| step.contains("fund_escrow") || step.contains("fund escrow")),
        "S-04 should include escrow funding step marker"
    );
    assert!(
        s04.pass_criteria
            .iter()
            .any(|entry| entry.contains("Pending") && entry.contains("Completed")),
        "S-04 should include lifecycle pass-criteria marker"
    );
}
