mod support;

use kamn_e2e_harness::{
    execute_run_contract, execute_verify_contract, RunCommandConfig, VerifyCommandConfig,
};
use support::command_contract_support::{
    set_executable, temp_path, valid_chain_dump_json, with_external_component_binaries,
    write_failing_stub_binary,
};

#[test]
fn spec_c91_verify_command_rejects_missing_infrastructure_kolme_version_marker() {
    let evidence_dir = temp_path("evidence-missing-infra");
    let output_path = temp_path("report-missing-infra.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_infra.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("manifest missing infrastructure.kolme_version"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c92_verify_command_rejects_missing_summary_proofs_verified_marker() {
    let evidence_dir = temp_path("evidence-missing-summary");
    let output_path = temp_path("report-missing-summary.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_summary.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("manifest missing summary.proofs_verified"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c93_verify_command_rejects_evidence_artifact_missing_verification_block() {
    let evidence_dir = temp_path("evidence-missing-verification-block");
    let output_path = temp_path("report-missing-verification-block.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_verification_block.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("evidence artifact missing _verification block"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c94_verify_command_rejects_evidence_artifact_missing_kolme_anchor_tx_hash() {
    let evidence_dir = temp_path("evidence-missing-verification-tx-hash");
    let output_path = temp_path("report-missing-verification-tx-hash.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_verification_tx_hash.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"block_height":42,"finality":"FINAL"}}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("evidence artifact missing _verification.kolme_anchor.tx_hash"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c95_verify_command_accepts_evidence_artifact_with_complete_verification_block() {
    let evidence_dir = temp_path("evidence-complete-verification-block");
    let output_path = temp_path("report-complete-verification-block.json");
    let chain_dump_path = temp_path("kolme_chain_dump_complete_verification_block.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let report = execute_verify_contract(&config).expect("verify should succeed");
    assert!(report.contains("\"schema_check\""));
    assert!(report.contains("\"proof_check\""));
    assert!(report.contains("\"chain_check\""));
    assert!(report.contains("\"content_check\""));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c96_verify_command_rejects_chain_dump_missing_chain_name_marker() {
    let evidence_dir = temp_path("evidence-missing-chain-name");
    let output_path = temp_path("report-missing-chain-name.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_chain_name.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(&chain_dump_path, r#"{"chain_version":1,"blocks":[]}"#)
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("chain dump missing chain_name marker"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c97_verify_command_rejects_chain_dump_missing_blocks_marker() {
    let evidence_dir = temp_path("evidence-missing-chain-blocks");
    let output_path = temp_path("report-missing-chain-blocks.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_chain_blocks.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        &chain_dump_path,
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1}"#,
    )
    .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("chain dump missing blocks marker"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c98_verify_command_rejects_chain_dump_block_missing_block_hash_marker() {
    let evidence_dir = temp_path("evidence-missing-block-hash-marker");
    let output_path = temp_path("report-missing-block-hash-marker.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_block_hash_marker.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        &chain_dump_path,
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"previous_block_hash":"GENESIS"}]}"#,
    )
    .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("chain dump block missing block_hash marker"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c99_verify_command_rejects_chain_dump_block_missing_previous_block_hash_marker() {
    let evidence_dir = temp_path("evidence-missing-previous-block-hash-marker");
    let output_path = temp_path("report-missing-previous-block-hash-marker.json");
    let chain_dump_path = temp_path("kolme_chain_dump_missing_previous_block_hash_marker.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        &chain_dump_path,
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0"}]}"#,
    )
    .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config).expect_err("verify should fail for missing marker");
    assert!(err.contains("chain dump block missing previous_block_hash marker"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c100_verify_command_rejects_chain_dump_hash_continuity_mismatch() {
    let evidence_dir = temp_path("evidence-chain-hash-continuity-mismatch");
    let output_path = temp_path("report-chain-hash-continuity-mismatch.json");
    let chain_dump_path = temp_path("kolme_chain_dump_chain_hash_continuity_mismatch.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        &chain_dump_path,
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"GENESIS"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:wrong-prior-block"}]}"#,
    )
    .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config)
        .expect_err("verify should fail for chain continuity mismatch");
    assert!(err.contains("chain dump hash continuity mismatch at block index 1"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c101_verify_command_rejects_chain_dump_genesis_anchor_mismatch() {
    let evidence_dir = temp_path("evidence-chain-genesis-anchor-mismatch");
    let output_path = temp_path("report-chain-genesis-anchor-mismatch.json");
    let chain_dump_path = temp_path("kolme_chain_dump_chain_genesis_anchor_mismatch.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        &chain_dump_path,
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"sha256:not-genesis"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:block-0"}]}"#,
    )
    .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config)
        .expect_err("verify should fail for chain genesis anchor mismatch");
    assert!(err.contains("chain dump genesis anchor mismatch at block index 0"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c102_verify_command_rejects_evidence_artifact_non_final_kolme_anchor_finality() {
    let evidence_dir = temp_path("evidence-invalid-finality-value");
    let output_path = temp_path("report-invalid-finality-value.json");
    let chain_dump_path = temp_path("kolme_chain_dump_invalid_finality_value.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"PENDING"}}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config)
        .expect_err("verify should fail for non-final finality value");
    assert!(err.contains("evidence artifact invalid _verification.kolme_anchor.finality value"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c103_verify_command_rejects_evidence_artifact_invalid_evidence_hash_format() {
    let evidence_dir = temp_path("evidence-invalid-evidence-hash-format");
    let output_path = temp_path("report-invalid-evidence-hash-format.json");
    let chain_dump_path = temp_path("kolme_chain_dump_invalid_evidence_hash_format.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err =
        execute_verify_contract(&config).expect_err("verify should fail for invalid evidence hash");
    assert!(err.contains("evidence artifact invalid _verification.evidence_hash format"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c104_verify_command_rejects_evidence_artifact_invalid_anchor_tx_hash_format() {
    let evidence_dir = temp_path("evidence-invalid-anchor-tx-hash-format");
    let output_path = temp_path("report-invalid-anchor-tx-hash-format.json");
    let chain_dump_path = temp_path("kolme_chain_dump_invalid_anchor_tx_hash_format.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"def456","block_height":42,"finality":"FINAL"}}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config)
        .expect_err("verify should fail for invalid anchor tx hash");
    assert!(err.contains("evidence artifact invalid _verification.kolme_anchor.tx_hash format"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c105_verify_command_rejects_evidence_artifact_invalid_anchor_block_height_format() {
    let evidence_dir = temp_path("evidence-invalid-anchor-block-height-format");
    let output_path = temp_path("report-invalid-anchor-block-height-format.json");
    let chain_dump_path = temp_path("kolme_chain_dump_invalid_anchor_block_height_format.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":"forty-two","finality":"FINAL"}}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config)
        .expect_err("verify should fail for invalid anchor block_height");
    assert!(
        err.contains("evidence artifact invalid _verification.kolme_anchor.block_height format")
    );

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c106_verify_command_rejects_evidence_artifact_invalid_captured_at_format() {
    let evidence_dir = temp_path("evidence-invalid-captured-at-format");
    let output_path = temp_path("report-invalid-captured-at-format.json");
    let chain_dump_path = temp_path("kolme_chain_dump_invalid_captured_at_format.json");
    std::fs::create_dir_all(evidence_dir.join("s01-agent-discovery"))
        .expect("evidence scenario dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026/02/21 14:31:05","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
    )
    .expect("evidence artifact should be written");
    std::fs::write(&chain_dump_path, valid_chain_dump_json())
        .expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let err = execute_verify_contract(&config)
        .expect_err("verify should fail for invalid captured_at format");
    assert!(err.contains("evidence artifact invalid _verification.captured_at format"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_file(
        evidence_dir
            .join("s01-agent-discovery")
            .join("alice_registration.json"),
    );
    let _ = std::fs::remove_dir(evidence_dir.join("s01-agent-discovery"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c107_external_execution_probe_failure_marks_runtime_orchestration_fail() {
    let kolme_binary = temp_path("kolme-node-probe-fail");
    write_failing_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);

    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };

    let output = with_external_component_binaries(|| {
        execute_run_contract(&config).expect("run output should render")
    });
    assert!(output.contains("\"runtime_orchestration\":"));
    assert!(output.contains("\"status\":\"FAIL\""));
    assert!(output.contains("probe failed"));

    let _ = std::fs::remove_file(kolme_binary);
}

#[test]
fn spec_c108_external_execution_probe_failure_marks_validation_fail() {
    let kolme_binary = temp_path("kolme-node-runtime-validation-fail");
    write_failing_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);

    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };

    let output = with_external_component_binaries(|| {
        execute_run_contract(&config).expect("run output should render")
    });
    assert!(output.contains("\"runtime_validation_execution\":"));
    assert!(output.contains("\"orchestration_contract\":\"FAIL\""));
    assert!(output.contains("\"lifecycle_contract\":\"FAIL\""));
    assert!(output.contains("\"overall\":\"FAIL\""));

    let _ = std::fs::remove_file(kolme_binary);
}

#[test]
fn spec_c109_run_output_contains_ordered_scenario_contract_projection() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-03".to_owned(), "S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"scenario_contracts\":["));
    assert!(output.contains("\"steps\":["));
    assert!(output.contains("\"verifiable_outputs\":["));
    assert!(output.contains("\"pass_criteria\":["));

    let s03_index = output
        .find("\"id\":\"S-03\"")
        .expect("S-03 contract entry should be present");
    let s01_index = output
        .find("\"id\":\"S-01\"")
        .expect("S-01 contract entry should be present");
    assert!(
        s03_index < s01_index,
        "scenario contracts should preserve selected order"
    );
}

#[test]
fn spec_c110_run_command_persists_manifest_chain_dump_and_scenario_artifact_on_pass() {
    let evidence_dir = temp_path("evidence-persist-pass");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: evidence_dir.display().to_string(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"evidence_contract\":{\"expected_artifacts\":4,\"recorded_artifacts\":4,\"status\":\"PASS\"}"));

    let manifest_path = evidence_dir.join("manifest.json");
    let chain_dump_path = evidence_dir.join("kolme_chain_dump.json");
    let scenario_artifact_path = evidence_dir.join("scenario-s01").join("artifact.json");
    assert!(manifest_path.is_file(), "manifest should be persisted");
    assert!(chain_dump_path.is_file(), "chain dump should be persisted");
    assert!(
        scenario_artifact_path.is_file(),
        "scenario artifact should be persisted"
    );

    let manifest =
        std::fs::read_to_string(&manifest_path).expect("manifest content should be readable");
    assert!(manifest.contains("\"schema_version\":\"kamn.e2e.evidence-manifest.v3\""));
    assert!(manifest.contains("\"id\":\"S-01\""));
    assert!(manifest.contains("scenario-s01/artifact.json"));
    let scenario_artifact = std::fs::read_to_string(&scenario_artifact_path)
        .expect("scenario artifact content should be readable");
    assert!(scenario_artifact.contains("\"_verification\":"));
    assert!(scenario_artifact.contains("\"finality\":\"FINAL\""));

    let _ = std::fs::remove_dir_all(evidence_dir);
}

#[test]
fn spec_c111_run_command_evidence_fail_path_omits_chain_dump_and_scenario_artifacts() {
    let evidence_dir = temp_path("evidence-fail-persist");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    let stale_chain_dump = evidence_dir.join("kolme_chain_dump.json");
    std::fs::write(&stale_chain_dump, valid_chain_dump_json())
        .expect("stale chain dump should be written");

    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: evidence_dir.display().to_string(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"evidence_contract\":{\"expected_artifacts\":4,\"recorded_artifacts\":3,\"status\":\"FAIL\"}"));

    let manifest_path = evidence_dir.join("manifest.json");
    let chain_dump_path = evidence_dir.join("kolme_chain_dump.json");
    let scenario_artifact_path = evidence_dir.join("scenario-s01").join("artifact.json");
    assert!(
        manifest_path.is_file(),
        "manifest should still be persisted"
    );
    assert!(
        !chain_dump_path.exists(),
        "chain dump should be removed on fail path"
    );
    assert!(
        !scenario_artifact_path.exists(),
        "scenario artifacts should not be persisted on fail path"
    );

    let _ = std::fs::remove_dir_all(evidence_dir);
}
