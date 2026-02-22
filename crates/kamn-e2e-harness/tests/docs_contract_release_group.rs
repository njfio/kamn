// Consolidated release-series docs-contract suites (issue #5690).

mod r52_integration_config_mapping_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c18_r52_integration_config_mapping_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r52-integration-config-mapping-fix.md"),
        )
        .expect("r52 integration-config mapping docs marker artifact should exist");
        assert!(doc.contains("r52_integration_config_mapping_status_before=buggy"));
        assert!(doc.contains("r52_integration_config_mapping_contract=implemented"));
        assert!(doc.contains("r52_integration_config_mapping_status_after=fixed"));
    }

    #[test]
    fn spec_c19_r52_milestone_index_references_active_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md"),
        )
        .expect("r52 milestone index should exist");
        assert!(milestone_index.contains("#5617"));
    }
}

mod r52_preflight_absolute_path_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c16_r52_preflight_absolute_path_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-r52-preflight-absolute-path-diagnostics.md",
            ))
            .expect("r52 preflight absolute-path docs marker artifact should exist");
        assert!(doc.contains("r52_preflight_absolute_path_status_before=partial"));
        assert!(doc.contains("r52_preflight_absolute_path_contract=implemented"));
        assert!(doc.contains("r52_preflight_absolute_path_status_after=implemented"));
    }

    #[test]
    fn spec_c17_r52_milestone_index_references_active_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md"),
        )
        .expect("r52 milestone index should exist");
        assert!(milestone_index.contains("#5615"));
    }
}

mod r52_preflight_executable_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c12_r52_preflight_executable_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r52-preflight-executable-diagnostics.md"),
        )
        .expect("r52 preflight executable docs marker artifact should exist");
        assert!(doc.contains("r52_preflight_executable_status_before=partial"));
        assert!(doc.contains("r52_preflight_executable_contract=implemented"));
        assert!(doc.contains("r52_preflight_executable_status_after=implemented"));
    }

    #[test]
    fn spec_c13_r52_milestone_index_references_active_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md"),
        )
        .expect("r52 milestone index should exist");
        assert!(milestone_index.contains("#5610"));
    }
}

mod r52_preflight_non_file_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c14_r52_preflight_non_file_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r52-preflight-non-file-diagnostics.md"),
        )
        .expect("r52 preflight non-file docs marker artifact should exist");
        assert!(doc.contains("r52_preflight_non_file_status_before=partial"));
        assert!(doc.contains("r52_preflight_non_file_contract=implemented"));
        assert!(doc.contains("r52_preflight_non_file_status_after=implemented"));
    }

    #[test]
    fn spec_c15_r52_milestone_index_references_active_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md"),
        )
        .expect("r52 milestone index should exist");
        assert!(milestone_index.contains("#5613"));
    }
}

mod r53_evidence_contract_status_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r53_evidence_contract_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r53-evidence-contract-status.md"),
        )
        .expect("r53 evidence-contract docs marker artifact should exist");
        assert!(doc.contains("r53_evidence_contract_status_before=implicit"));
        assert!(doc.contains("r53_evidence_contract_contract=implemented"));
        assert!(doc.contains("r53_evidence_contract_status_after=active"));
    }

    #[test]
    fn spec_c02_r53_milestone_index_references_active_issue_5624() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r53-e2e-scenario-execution-activation/index.md"),
        )
        .expect("r53 milestone index should exist");
        assert!(milestone_index.contains("#5624"));
    }
}

mod r53_live_status_alignment_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r53_live_status_alignment_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r53-live-status-alignment.md"),
        )
        .expect("r53 live-status docs marker artifact should exist");
        assert!(doc.contains("r53_live_status_alignment_status_before=static-pass"));
        assert!(doc.contains("r53_live_status_alignment_contract=implemented"));
        assert!(doc.contains("r53_live_status_alignment_status_after=active"));
    }

    #[test]
    fn spec_c02_r53_milestone_index_references_active_issue_5622() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r53-e2e-scenario-execution-activation/index.md"),
        )
        .expect("r53 milestone index should exist");
        assert!(milestone_index.contains("#5622"));
    }
}

mod r53_mode_execution_contract_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r53_mode_execution_contract_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r53-mode-execution-contract.md"),
        )
        .expect("r53 mode-execution docs marker artifact should exist");
        assert!(doc.contains("r53_mode_execution_contract_status_before=implicit"));
        assert!(doc.contains("r53_mode_execution_contract_contract=implemented"));
        assert!(doc.contains("r53_mode_execution_contract_status_after=active"));
    }

    #[test]
    fn spec_c02_r53_milestone_index_references_active_issue_5626() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r53-e2e-scenario-execution-activation/index.md"),
        )
        .expect("r53 milestone index should exist");
        assert!(milestone_index.contains("#5626"));
    }

    #[test]
    fn spec_c03_r53_milestone_index_marks_milestone_closed() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r53-e2e-scenario-execution-activation/index.md"),
        )
        .expect("r53 milestone index should exist");
        assert!(milestone_index.contains("Active issue(s): None"));
        assert!(milestone_index.contains("Completed issue(s): #5620, #5622, #5624, #5626"));
        assert!(
            milestone_index.contains(
                "4. Mode execution contract parity across sdk-direct/cli-scripted/mcp-* drivers. (Completed)"
            )
        );
    }
}

mod r53_scenario_run_execution_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r53_scenario_run_execution_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r53-scenario-run-execution.md"),
        )
        .expect("r53 scenario-run docs marker artifact should exist");
        assert!(doc.contains("r53_scenario_run_execution_status_before=scaffold-skip"));
        assert!(doc.contains("r53_scenario_run_execution_contract=implemented"));
        assert!(doc.contains("r53_scenario_run_execution_status_after=active"));
    }

    #[test]
    fn spec_c02_r53_milestone_index_references_active_issue() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r53-e2e-scenario-execution-activation/index.md"),
        )
        .expect("r53 milestone index should exist");
        assert!(milestone_index.contains("#5620"));
    }
}

mod r54_evidence_phase_activation_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r54_evidence_phase_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r54-evidence-phase-activation.md"),
        )
        .expect("r54 evidence-phase docs marker artifact should exist");
        assert!(doc.contains("r54_evidence_phase_status_before=static-skip"));
        assert!(doc.contains("r54_evidence_phase_contract=implemented"));
        assert!(doc.contains("r54_evidence_phase_status_after=active"));
    }

    #[test]
    fn spec_c02_r54_milestone_index_references_active_issue_5629() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r54-e2e-evidence-phase-activation/index.md"),
        )
        .expect("r54 milestone index should exist");
        assert!(milestone_index.contains("#5629"));
    }
}

mod r54_teardown_phase_activation_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r54_teardown_phase_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r54-teardown-phase-activation.md"),
        )
        .expect("r54 teardown-phase docs marker artifact should exist");
        assert!(doc.contains("r54_teardown_phase_status_before=static-skip"));
        assert!(doc.contains("r54_teardown_phase_contract=implemented"));
        assert!(doc.contains("r54_teardown_phase_status_after=active"));
    }

    #[test]
    fn spec_c02_r54_milestone_index_references_issue_5631() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r54-e2e-evidence-phase-activation/index.md"),
        )
        .expect("r54 milestone index should exist");
        assert!(milestone_index.contains("#5631"));
    }
}

mod r55_evidence_step_inventory_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r55_evidence_step_inventory_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r55-evidence-step-inventory.md"),
        )
        .expect("r55 evidence-step docs marker artifact should exist");
        assert!(doc.contains("r55_evidence_step_inventory_status_before=single-step"));
        assert!(doc.contains("r55_evidence_step_inventory_contract=implemented"));
        assert!(doc.contains("r55_evidence_step_inventory_status_after=active"));
    }

    #[test]
    fn spec_c02_r55_milestone_index_references_issue_5634() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r55-e2e-evidence-step-inventory-parity/index.md"),
        )
        .expect("r55 milestone index should exist");
        assert!(milestone_index.contains("#5634"));
    }
}

mod r56_verify_manifest_hardening_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r56_verify_manifest_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(
            root.join("docs/research/e2e-live-testing-prd-r56-verify-manifest-hardening.md"),
        )
        .expect("r56 verify-manifest docs marker artifact should exist");
        assert!(doc.contains("r56_verify_manifest_nested_field_contract_status_before=partial"));
        assert!(doc.contains("r56_verify_manifest_infrastructure_marker_enforcement=implemented"));
        assert!(doc.contains("r56_verify_manifest_summary_marker_enforcement=implemented"));
        assert!(doc.contains("r56_verify_manifest_nested_field_contract_status_after=implemented"));
    }

    #[test]
    fn spec_c02_r56_milestone_index_references_issue_5637() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r56-e2e-verify-manifest-contract-hardening/index.md"),
        )
        .expect("r56 milestone index should exist");
        assert!(milestone_index.contains("#5637"));
    }
}

mod r57_evidence_verification_block_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r57_evidence_verification_block_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(root.join(
            "docs/research/e2e-live-testing-prd-r57-evidence-verification-block-enforcement.md",
        ))
        .expect("r57 evidence verification block docs marker artifact should exist");
        assert!(doc.contains("r57_evidence_verification_block_contract_status_before=missing"));
        assert!(doc.contains("r57_verify_artifact_verification_marker_enforcement=implemented"));
        assert!(doc.contains("r57_evidence_verification_block_contract_status_after=implemented"));
    }

    #[test]
    fn spec_c02_r57_milestone_index_references_issue_5640() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r57-e2e-evidence-verification-block-enforcement/index.md"),
        )
        .expect("r57 milestone index should exist");
        assert!(milestone_index.contains("#5640"));
    }
}

mod r58_chain_dump_verification_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r58_chain_dump_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-r58-chain-dump-verification-hardening.md",
            ))
            .expect("r58 chain dump docs marker artifact should exist");
        assert!(doc.contains("r58_chain_dump_marker_contract_status_before=missing"));
        assert!(doc.contains("r58_verify_chain_dump_marker_enforcement=implemented"));
        assert!(doc.contains("r58_chain_dump_marker_contract_status_after=implemented"));
    }

    #[test]
    fn spec_c02_r58_milestone_index_references_issue_5643() {
        let root = repo_root();
        let milestone_index =
            std::fs::read_to_string(root.join(
                "specs/milestones/r58-e2e-chain-dump-verification-contract-hardening/index.md",
            ))
            .expect("r58 milestone index should exist");
        assert!(milestone_index.contains("#5643"));
    }
}

mod r59_chain_hash_continuity_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r59_chain_hash_continuity_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-r59-chain-hash-continuity-verification.md",
            ))
            .expect("r59 chain hash continuity docs marker artifact should exist");
        assert!(doc.contains("r59_chain_hash_continuity_contract_status_before=missing"));
        assert!(doc.contains("r59_verify_chain_hash_continuity_enforcement=implemented"));
        assert!(doc.contains("r59_chain_hash_continuity_contract_status_after=implemented"));
    }

    #[test]
    fn spec_c02_r59_milestone_index_references_issue_5646() {
        let root = repo_root();
        let milestone_index =
            std::fs::read_to_string(root.join(
                "specs/milestones/r59-e2e-chain-hash-continuity-verification-contract/index.md",
            ))
            .expect("r59 milestone index should exist");
        assert!(milestone_index.contains("#5646"));
    }
}

mod r60_chain_genesis_anchor_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r60_chain_genesis_anchor_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-r60-chain-genesis-anchor-verification.md",
            ))
            .expect("r60 chain genesis anchor docs marker artifact should exist");
        assert!(doc.contains("r60_chain_genesis_anchor_contract_status_before=missing"));
        assert!(doc.contains("r60_verify_chain_genesis_anchor_enforcement=implemented"));
        assert!(doc.contains("r60_chain_genesis_anchor_contract_status_after=implemented"));
    }

    #[test]
    fn spec_c02_r60_milestone_index_references_issue_5649() {
        let root = repo_root();
        let milestone_index =
            std::fs::read_to_string(root.join(
                "specs/milestones/r60-e2e-chain-genesis-anchor-verification-contract/index.md",
            ))
            .expect("r60 milestone index should exist");
        assert!(milestone_index.contains("#5649"));
    }
}

mod r61_verification_finality_value_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r61_verification_finality_value_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(root.join(
            "docs/research/e2e-live-testing-prd-r61-verification-finality-value-contract.md",
        ))
        .expect("r61 finality value docs marker artifact should exist");
        assert!(doc.contains("r61_verification_finality_value_contract_status_before=missing"));
        assert!(doc.contains("r61_verify_artifact_finality_value_enforcement=implemented"));
        assert!(doc.contains("r61_verification_finality_value_contract_status_after=implemented"));
    }

    #[test]
    fn spec_c02_r61_milestone_index_references_issue_5652() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r61-e2e-verification-finality-value-contract/index.md"),
        )
        .expect("r61 milestone index should exist");
        assert!(milestone_index.contains("#5652"));
    }
}

mod r62_verification_hash_format_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r62_verification_hash_format_docs_markers_present() {
        let root = repo_root();
        let doc =
            std::fs::read_to_string(root.join(
                "docs/research/e2e-live-testing-prd-r62-verification-hash-format-contract.md",
            ))
            .expect("r62 hash-format docs marker artifact should exist");
        assert!(doc.contains("r62_verification_hash_format_contract_status_before=missing"));
        assert!(doc.contains("r62_verify_artifact_hash_format_enforcement=implemented"));
        assert!(doc.contains("r62_verification_hash_format_contract_status_after=implemented"));
    }

    #[test]
    fn spec_c02_r62_milestone_index_references_issue_5655() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r62-e2e-verification-hash-format-contract/index.md"),
        )
        .expect("r62 milestone index should exist");
        assert!(milestone_index.contains("#5655"));
    }
}

mod r63_verification_anchor_height_format_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r63_verification_anchor_height_format_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(root.join(
            "docs/research/e2e-live-testing-prd-r63-verification-anchor-height-format-contract.md",
        ))
        .expect("r63 anchor-height format docs marker artifact should exist");
        assert!(
            doc.contains("r63_verification_anchor_height_format_contract_status_before=missing")
        );
        assert!(doc.contains("r63_verify_anchor_block_height_format_enforcement=implemented"));
        assert!(
            doc.contains("r63_verification_anchor_height_format_contract_status_after=implemented")
        );
    }

    #[test]
    fn spec_c02_r63_milestone_index_references_issue_5658() {
        let root = repo_root();
        let milestone_index =
            std::fs::read_to_string(root.join(
                "specs/milestones/r63-e2e-verification-anchor-height-format-contract/index.md",
            ))
            .expect("r63 milestone index should exist");
        assert!(milestone_index.contains("#5658"));
    }
}

mod r64_verification_captured_at_format_docs_contract {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .to_path_buf()
    }

    #[test]
    fn spec_c01_r64_verification_captured_at_format_docs_markers_present() {
        let root = repo_root();
        let doc = std::fs::read_to_string(root.join(
            "docs/research/e2e-live-testing-prd-r64-verification-captured-at-format-contract.md",
        ))
        .expect("r64 captured-at format docs marker artifact should exist");
        assert!(doc.contains("r64_verification_captured_at_format_contract_status_before=missing"));
        assert!(doc.contains("r64_verify_captured_at_format_enforcement=implemented"));
        assert!(
            doc.contains("r64_verification_captured_at_format_contract_status_after=implemented")
        );
    }

    #[test]
    fn spec_c02_r64_milestone_index_references_issue_5661() {
        let root = repo_root();
        let milestone_index = std::fs::read_to_string(
            root.join("specs/milestones/r64-e2e-verification-captured-at-format-contract/index.md"),
        )
        .expect("r64 milestone index should exist");
        assert!(milestone_index.contains("#5661"));
    }
}
