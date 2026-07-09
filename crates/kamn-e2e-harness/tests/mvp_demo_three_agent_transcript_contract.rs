use kamn_e2e_harness::mvp_demo::verify_mvp_demo_report_json;
use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};
use std::path::{Path, PathBuf};

#[path = "support/mvp_local_artifacts.rs"]
mod mvp_local_artifacts;

#[test]
fn spec_c01_report_rejects_missing_transcript_fields() {
    let root = Path::new("/tmp/kamn-7056-unused");
    let transcript = Path::new("/tmp/unused.json");
    let report = report_with_claim(
        three_agent_claim_without_transcript(transcript),
        root,
        transcript,
    );

    let err = verify_mvp_demo_report_json(report)
        .expect_err("three-agent claim must include transcript fields");

    assert!(err.contains("three_agent_transcript_artifact"));
}

#[test]
fn spec_c02_command_rejects_missing_transcript_artifact() {
    let root = temp_root("missing-artifact");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    let transcript = root.join("proof/three-agent-transcript.json");
    let report = write_report(
        &root,
        report_with_claim(three_agent_claim(&transcript), &root, &transcript),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("missing transcript artifact must fail");

    assert!(err.contains("three-agent transcript artifact"));
}

#[test]
fn spec_c03_command_rejects_mismatched_transcript_settlement() {
    let root = temp_root("mismatched-settlement");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    let transcript = write_transcript(&root, valid_transcript().replace(SIGNATURE, "mismatch"));
    let report = write_report(
        &root,
        report_with_claim(three_agent_claim(&transcript), &root, &transcript),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("mismatched transcript settlement must fail");

    assert!(err.contains("three-agent transcript settlement_tx_signature"));
}

#[test]
fn spec_c04_command_rejects_raw_private_payload_transcript() {
    let root = temp_root("raw-private");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    let leaked = valid_transcript().replace(
        r#""private_payload_redacted":true"#,
        r#""raw_private_payload":"secret","private_payload_redacted":true"#,
    );
    let transcript = write_transcript(&root, leaked);
    let report = write_report(
        &root,
        report_with_claim(three_agent_claim(&transcript), &root, &transcript),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("raw private payload transcript must fail");

    assert!(err.contains("raw private payload"));
}

fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!("kamn-7056-{stem}-{}-{millis}", std::process::id()))
}

fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
    }
}

fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    write_file(path.as_path(), report);
    path
}

fn write_transcript(root: &Path, transcript: String) -> PathBuf {
    let path = root.join("proof/three-agent-transcript.json");
    write_file(path.as_path(), transcript);
    path
}

fn write_file(path: &Path, content: String) {
    std::fs::create_dir_all(path.parent().expect("parent should exist"))
        .expect("fixture directory should be created");
    std::fs::write(path, content).expect("fixture should be written");
}

fn report_with_claim(three_agent_claim: String, root: &Path, transcript_path: &Path) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-three-agent","status":"GO","devnet_mode":"required","artifacts":{},"claim_matrix":[{},{},{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        mvp_local_artifacts::artifacts_json(root, Some(transcript_path)),
        local_claims(),
        devnet_settlement_claim(),
        three_agent_claim,
        roadmap_claim()
    )
}

fn local_claims() -> &'static str {
    r#"{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local runtime"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"agent identities"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"message flow"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket events"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit export"}"#
}

const SIGNATURE: &str = "5nSgnDevnetSignature111111111111111111111111111";

fn devnet_settlement_claim() -> &'static str {
    r#"{"id":"devnet_settlement_asset_movement","label":"devnet-backed","required":true,"status":"PASS","summary":"Solana devnet escrow settlement transfer observed","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111"}"#
}

fn three_agent_claim(path: &Path) -> String {
    transcript_fields(path) + three_agent_claim_without_transcript(path).trim_start_matches('{')
}

fn transcript_fields(path: &Path) -> String {
    format!(
        r#"{{"three_agent_transcript_artifact":"{}","three_agent_transcript_digest":"three-agent-transcript-digest-7045","#,
        path.display()
    )
}

fn three_agent_claim_without_transcript(path: &Path) -> String {
    let base = r#"{"id":"three_agent_escrow_verification","label":"devnet-backed","required":true,"status":"PASS","summary":"Agent C verifies escrow settlement from restricted proof view","transaction_id":"tx-three-agent-7045","terms_digest":"terms-digest-7045","agent_a_terms_digest":"terms-digest-7045","agent_b_terms_digest":"terms-digest-7045","verifier_terms_digest":"terms-digest-7045","escrow_id":"escrow-three-agent-7045","agent_a_escrow_id":"escrow-three-agent-7045","agent_b_escrow_id":"escrow-three-agent-7045","verifier_escrow_id":"escrow-three-agent-7045","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_a_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_b_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","verifier_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","agent_a_settlement_commitment":"finalized","agent_b_settlement_commitment":"finalized","verifier_settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"agent_a_amount_lamports":1,"agent_b_amount_lamports":1,"verifier_amount_lamports":1,"agent_a_private_view_visible":true,"agent_b_private_view_visible":true,"verifier_private_view_visible":false,"agent_a_view_scope":"participant-private","agent_b_view_scope":"participant-private","verifier_view_scope":"restricted-public","agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"agent_a_private_view_digest":"agent-a-private-digest-7045","agent_b_private_view_digest":"agent-b-private-digest-7045","agent_a_public_view_digest":"public-view-digest-7045","agent_b_public_view_digest":"public-view-digest-7045","verifier_public_view_digest":"public-view-digest-7045","private_payload_redacted":true}"#;
    base.replace(
        r#""transaction_id":"tx-three-agent-7045""#,
        format!(
            "{},\"transaction_id\":\"tx-three-agent-7045\"",
            view_fields(path)
        )
        .as_str(),
    )
}

fn view_fields(path: &Path) -> String {
    let proof = path.parent().expect("transcript should live under proof");
    format!(
        r#""agent_a_view_artifact":"{}","agent_b_view_artifact":"{}","agent_c_verifier_view_artifact":"{}","agent_a_view_digest":"agent-a-view-digest-7045","agent_b_view_digest":"agent-b-view-digest-7045","agent_c_verifier_view_digest":"agent-c-view-digest-7045""#,
        proof.join("agent-a-view.json").display(),
        proof.join("agent-b-view.json").display(),
        proof.join("agent-c-verifier-view.json").display()
    )
}

fn valid_transcript() -> String {
    r#"{"schema_version":"kamn.mvp.three-agent-transcript.v1","proof_label":"local-only","devnet_settlement_linked":true,"transaction_id":"tx-three-agent-7045","escrow_id":"escrow-three-agent-7045","steps":["agent_a_registered","agent_b_registered","agent_a_invoked_transaction","agent_b_accepted_task","escrow_funded","escrow_released","agent_c_verified"],"views":{"agent_a":"participant-private","agent_b":"participant-private","agent_c":"restricted-public"},"agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"private_payload_redacted":true,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","transcript_digest":"three-agent-transcript-digest-7045"}"#.to_owned()
}

fn roadmap_claim() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}
