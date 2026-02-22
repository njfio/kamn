// Consolidated phase/structure docs-contract suites (issue #5690).

mod phase4b_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c10_phase4b_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4b-gap-analysis.md"),
        )
        .expect("phase-4b docs marker artifact should exist");
        assert!(doc.contains("phase4b_status_before=partial"));
        assert!(doc.contains("phase4b_run_command_contract=implemented"));
        assert!(doc.contains("phase4b_verify_command_contract=implemented"));
        assert!(doc.contains("phase4b_scenario_csv_validation=implemented"));
        assert!(doc.contains("phase4b_verify_output_contract=implemented"));
        assert!(doc.contains("phase4b_status_after=implemented"));
    }

    #[test]
    fn spec_c10_milestone_index_references_active_phase4b_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5566"));
    }
}

mod phase4c_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c07_phase4c_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4c-gap-analysis.md"),
        )
        .expect("phase-4c docs marker artifact should exist");
        assert!(doc.contains("phase4c_status_before=partial"));
        assert!(doc.contains("phase4c_orchestration_phase_model=implemented"));
        assert!(doc.contains("phase4c_phase_progression_markers=implemented"));
        assert!(doc.contains("phase4c_status_after=implemented"));
    }

    #[test]
    fn spec_c08_milestone_index_references_active_phase4c_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5568"));
    }
}

mod phase4d_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c08_phase4d_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4d-gap-analysis.md"),
        )
        .expect("phase-4d docs marker artifact should exist");
        assert!(doc.contains("phase4d_status_before=partial"));
        assert!(doc.contains("phase4d_phase_result_model=implemented"));
        assert!(doc.contains("phase4d_infra_and_agent_placeholders=implemented"));
        assert!(doc.contains("phase4d_status_after=implemented"));
    }

    #[test]
    fn spec_c09_milestone_index_references_active_phase4d_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5570"));
    }
}

mod phase4e_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c07_phase4e_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4e-gap-analysis.md"),
        )
        .expect("phase-4e docs marker artifact should exist");
        assert!(doc.contains("phase4e_status_before=partial"));
        assert!(doc.contains("phase4e_step_record_model=implemented"));
        assert!(doc.contains("phase4e_infra_step_markers=implemented"));
        assert!(doc.contains("phase4e_agent_deploy_step_markers=implemented"));
        assert!(doc.contains("phase4e_status_after=implemented"));
    }

    #[test]
    fn spec_c08_milestone_index_references_active_phase4e_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5572"));
    }
}

mod phase4f_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c08_phase4f_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4f-gap-analysis.md"),
        )
        .expect("phase-4f docs marker artifact should exist");
        assert!(doc.contains("phase4f_status_before=partial"));
        assert!(doc.contains("phase4f_mode_aware_rules=implemented"));
        assert!(doc.contains("phase4f_controlled_fail_path=implemented"));
        assert!(doc.contains("phase4f_status_after=implemented"));
    }

    #[test]
    fn spec_c09_milestone_index_references_active_phase4f_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5574"));
    }
}

mod phase4g_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c07_phase4g_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4g-gap-analysis.md"),
        )
        .expect("phase-4g docs marker artifact should exist");
        assert!(doc.contains("phase4g_status_before=partial"));
        assert!(doc.contains("phase4g_lifecycle_summary=implemented"));
        assert!(doc.contains("phase4g_fail_path_summary=implemented"));
        assert!(doc.contains("phase4g_status_after=implemented"));
    }

    #[test]
    fn spec_c08_milestone_index_references_active_phase4g_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5576"));
    }
}

mod phase4h_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c08_phase4h_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4h-gap-analysis.md"),
        )
        .expect("phase-4h docs marker artifact should exist");
        assert!(doc.contains("phase4h_status_before=partial"));
        assert!(doc.contains("phase4h_runtime_binary_contract=implemented"));
        assert!(doc.contains("phase4h_status_after=implemented"));
    }

    #[test]
    fn spec_c09_milestone_index_references_active_phase4h_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5578"));
    }
}

mod phase4j_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c07_phase4j_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase4j-gap-analysis.md"),
        )
        .expect("phase-4j docs marker artifact should exist");
        assert!(doc.contains("phase4j_status_before=partial"));
        assert!(doc.contains("phase4j_runtime_readiness_contract=implemented"));
        assert!(doc.contains("phase4j_status_after=implemented"));
    }

    #[test]
    fn spec_c08_milestone_index_references_active_phase4j_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5582"));
    }
}

mod phase5a_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c09_phase5a_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase5a-gap-analysis.md"),
        )
        .expect("phase-5a docs marker artifact should exist");
        assert!(doc.contains("phase5a_status_before=partial"));
        assert!(doc.contains("phase5a_process_runtime_contract=implemented"));
        assert!(doc.contains("phase5a_status_after=implemented"));
    }

    #[test]
    fn spec_c10_milestone_index_references_active_phase5a_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5584"));
    }
}

mod phase5b_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c09_phase5b_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase5b-gap-analysis.md"),
        )
        .expect("phase-5b docs marker artifact should exist");
        assert!(doc.contains("phase5b_status_before=partial"));
        assert!(doc.contains("phase5b_process_lifecycle_contract=implemented"));
        assert!(doc.contains("phase5b_status_after=implemented"));
    }

    #[test]
    fn spec_c10_milestone_index_references_active_phase5b_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5586"));
    }
}

mod phase5c_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c08_phase5c_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase5c-gap-analysis.md"),
        )
        .expect("phase-5c docs marker artifact should exist");
        assert!(doc.contains("phase5c_status_before=partial"));
        assert!(doc.contains("phase5c_spawn_timeline_contract=implemented"));
        assert!(doc.contains("phase5c_status_after=implemented"));
    }

    #[test]
    fn spec_c09_milestone_index_references_active_phase5c_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5588"));
    }
}

mod phase5d_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c07_phase5d_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase5d-gap-analysis.md"),
        )
        .expect("phase-5d docs marker artifact should exist");
        assert!(doc.contains("phase5d_status_before=partial"));
        assert!(doc.contains("phase5d_live_validation_contract=implemented"));
        assert!(doc.contains("phase5d_status_after=implemented"));
    }

    #[test]
    fn spec_c08_milestone_index_references_active_phase5d_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5590"));
    }
}

mod phase6_runtime_integration_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c11_phase6_runtime_integration_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-phase6-runtime-integration-gap-analysis.md",
            ))
            .expect("phase-6 runtime integration docs marker artifact should exist");
        assert!(doc.contains("phase6_runtime_integration_status_before=partial"));
        assert!(doc.contains("phase6_runtime_integration_guard_contract=implemented"));
        assert!(doc.contains("phase6_runtime_integration_status_after=implemented"));
    }

    #[test]
    fn spec_c12_milestone_index_references_active_phase6_runtime_integration_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5600"));
    }
}

mod phase6_runtime_lifecycle_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c13_phase6_runtime_lifecycle_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-phase6-runtime-lifecycle-gap-analysis.md",
            ))
            .expect("phase-6 runtime lifecycle docs marker artifact should exist");
        assert!(doc.contains("phase6_runtime_lifecycle_status_before=partial"));
        assert!(doc.contains("phase6_runtime_lifecycle_contract=implemented"));
        assert!(doc.contains("phase6_runtime_lifecycle_status_after=implemented"));
    }

    #[test]
    fn spec_c14_milestone_index_references_active_phase6_runtime_lifecycle_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5604"));
    }
}

mod phase6_runtime_orchestration_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c13_phase6_runtime_orchestration_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(root.join(
            "docs/research/e2e-live-testing-prd-phase6-runtime-orchestration-gap-analysis.md",
        ))
        .expect("phase-6 runtime orchestration docs marker artifact should exist");
        assert!(doc.contains("phase6_runtime_orchestration_status_before=partial"));
        assert!(doc.contains("phase6_runtime_orchestration_contract=implemented"));
        assert!(doc.contains("phase6_runtime_orchestration_status_after=implemented"));
    }

    #[test]
    fn spec_c14_milestone_index_references_active_phase6_runtime_orchestration_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5602"));
    }
}

mod phase6_runtime_validation_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c15_phase6_runtime_validation_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-phase6-runtime-validation-gap-analysis.md",
            ))
            .expect("phase-6 runtime validation docs marker artifact should exist");
        assert!(doc.contains("phase6_runtime_validation_status_before=partial"));
        assert!(doc.contains("phase6_runtime_validation_contract=implemented"));
        assert!(doc.contains("phase6_runtime_validation_status_after=implemented"));
    }

    #[test]
    fn spec_c16_milestone_index_references_active_phase6_runtime_validation_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5606"));
        assert!(milestone_index.contains("Active issue(s): None"));
        assert!(milestone_index
            .contains("25. Phase-6 runtime external validation execution. (Completed)"));
    }
}

mod phase6a_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c10_phase6a_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase6a-gap-analysis.md"),
        )
        .expect("phase-6a docs marker artifact should exist");
        assert!(doc.contains("phase6a_status_before=partial"));
        assert!(doc.contains("phase6a_spawn_plan_contract=implemented"));
        assert!(doc.contains("phase6a_status_after=implemented"));
    }

    #[test]
    fn spec_c11_milestone_index_references_active_phase6a_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5592"));
    }
}

mod phase6b_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c12_phase6b_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase6b-gap-analysis.md"),
        )
        .expect("phase-6b docs marker artifact should exist");
        assert!(doc.contains("phase6b_status_before=partial"));
        assert!(doc.contains("phase6b_spawn_execution_contract=implemented"));
        assert!(doc.contains("phase6b_status_after=implemented"));
    }

    #[test]
    fn spec_c13_milestone_index_references_active_phase6b_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5594"));
    }
}

mod phase6c_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c14_phase6c_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase6c-gap-analysis.md"),
        )
        .expect("phase-6c docs marker artifact should exist");
        assert!(doc.contains("phase6c_status_before=partial"));
        assert!(doc.contains("phase6c_live_process_execution_contract=implemented"));
        assert!(doc.contains("phase6c_status_after=implemented"));
    }

    #[test]
    fn spec_c15_milestone_index_references_active_phase6c_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5596"));
    }
}

mod phase6d_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c16_phase6d_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-phase6d-gap-analysis.md"),
        )
        .expect("phase-6d docs marker artifact should exist");
        assert!(doc.contains("phase6d_status_before=partial"));
        assert!(doc.contains("phase6d_live_execution_contract=implemented"));
        assert!(doc.contains("phase6d_status_after=implemented"));
    }

    #[test]
    fn spec_c17_milestone_index_references_active_phase6d_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
        )
        .expect("milestone index should exist");
        assert!(milestone_index.contains("#5598"));
    }
}

mod structure_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_harness_required_paths_exist() {
        let root = repo_root();
        let required_paths = [
            "crates/kamn-e2e-harness/Cargo.toml",
            "crates/kamn-e2e-harness/src/main.rs",
            "crates/kamn-e2e-harness/src/infrastructure.rs",
            "crates/kamn-e2e-harness/src/kolme_devnet.rs",
            "crates/kamn-e2e-harness/src/identity.rs",
            "crates/kamn-e2e-harness/src/drivers/mod.rs",
            "crates/kamn-e2e-harness/src/drivers/sdk_direct.rs",
            "crates/kamn-e2e-harness/src/drivers/cli_scripted.rs",
            "crates/kamn-e2e-harness/src/drivers/mcp_agent.rs",
            "crates/kamn-e2e-harness/src/scenarios/mod.rs",
            "crates/kamn-e2e-harness/src/scenarios/s01_discovery.rs",
            "crates/kamn-e2e-harness/src/scenarios/s02_message.rs",
            "crates/kamn-e2e-harness/src/scenarios/s03_group.rs",
            "crates/kamn-e2e-harness/src/scenarios/s04_task.rs",
            "crates/kamn-e2e-harness/src/scenarios/s05_escrow.rs",
            "crates/kamn-e2e-harness/src/scenarios/s06_kolme_verify.rs",
            "crates/kamn-e2e-harness/src/scenarios/s07_replay_protection.rs",
            "crates/kamn-e2e-harness/src/scenarios/s08_crash_recovery.rs",
            "crates/kamn-e2e-harness/src/scenarios/s09_transport_failover.rs",
            "crates/kamn-e2e-harness/src/scenarios/s10_topology_coherence.rs",
            "crates/kamn-e2e-harness/src/scenarios/s11_signer_rotation.rs",
            "crates/kamn-e2e-harness/src/scenarios/s12_retention_deletion.rs",
            "crates/kamn-e2e-harness/src/scenarios/s13_bridge_forwarding.rs",
            "crates/kamn-e2e-harness/src/scenarios/s14_batch_merkle.rs",
            "crates/kamn-e2e-harness/src/scenarios/s15_performance_smoke.rs",
            "crates/kamn-e2e-harness/src/evidence.rs",
            "crates/kamn-e2e-harness/src/verify.rs",
        ];

        for path in required_paths {
            assert!(root.join(path).is_file(), "required path missing: {path}");
        }
    }

    #[test]
    fn spec_c12_phase4a_docs_markers_present() {
        let root = repo_root();
        let doc_path = root.join("docs/research/e2e-live-testing-prd-phase4a-gap-analysis.md");
        let doc = std::fs::read_to_string(&doc_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", doc_path.display()));

        assert!(doc.contains("phase4a_required_paths_total=28"));
        assert!(doc.contains("phase4a_required_paths_present_before=20"));
        assert!(doc.contains("phase4a_required_paths_missing_before=8"));
        assert!(doc.contains("phase4a_required_paths_present_after=28"));
        assert!(doc.contains("phase4a_required_paths_missing_after=0"));
        assert!(doc.contains("phase4a_scenario_inventory_count=15"));
        assert!(doc.contains("phase4a_manifest_schema_version=kamn.e2e.evidence-manifest.v3"));
        assert!(doc.contains("phase4a_verifier_report_markers=schema,proof,chain,content"));
        assert!(doc.contains("phase4a_status_after=implemented"));
    }
}
