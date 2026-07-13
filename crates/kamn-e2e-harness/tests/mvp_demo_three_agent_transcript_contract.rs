use kamn_e2e_harness::mvp_demo::verify_mvp_demo_report_json;
use std::path::Path;

#[path = "support/generated_receipt_fixture.rs"]
mod generated_receipt_fixture;

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[path = "support/mvp_local_artifacts.rs"]
#[allow(dead_code)]
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
    let fixture = generated_receipt_fixture::Fixture::new("missing-transcript");
    fixture.remove_transcript();

    let err = fixture
        .verify()
        .expect_err("missing transcript artifact must fail");

    assert_eq!(err, "PROOF_ARTIFACT_MISSING");
}

#[test]
fn spec_c03_command_rejects_mismatched_transcript_settlement() {
    let fixture = generated_receipt_fixture::Fixture::new("mismatched-transcript-settlement");
    fixture.replace_transcript_field("settlement_tx_signature", "mismatch");

    let err = fixture
        .verify()
        .expect_err("mismatched transcript settlement must fail");

    assert_eq!(err, "TRANSACTION_AGREEMENT_INVALID");
}

#[test]
fn spec_c04_command_rejects_raw_private_payload_transcript() {
    let fixture = generated_receipt_fixture::Fixture::new("raw-private-transcript");
    fixture.replace_transcript_field("raw_private_payload", "secret");

    let err = fixture
        .verify()
        .expect_err("raw private payload transcript must fail");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c05_command_rejects_stale_transcript_digest_after_content_tamper() {
    let fixture = generated_receipt_fixture::Fixture::new("stale-transcript-digest");
    fixture.tamper_transcript();

    let err = fixture
        .verify()
        .expect_err("stale transcript digest must fail");

    assert_eq!(err, "TRANSACTION_AGREEMENT_INVALID");
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

fn devnet_settlement_claim() -> &'static str {
    r#"{"id":"devnet_settlement_asset_movement","label":"devnet-backed","required":true,"status":"PASS","summary":"Solana devnet escrow settlement transfer observed","network":"solana:devnet","execution_surface":"command-override","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"escrow_id":"escrow-command-fixture","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111"}"#
}

fn three_agent_claim_without_transcript(path: &Path) -> String {
    let base = r#"{"id":"three_agent_escrow_verification","label":"devnet-backed","required":true,"status":"PASS","summary":"Agent C verifies escrow settlement from restricted proof view","transaction_id":"tx-three-agent-7045","terms_digest":"terms-digest-7045","agent_a_terms_digest":"terms-digest-7045","agent_b_terms_digest":"terms-digest-7045","verifier_terms_digest":"terms-digest-7045","escrow_id":"escrow-three-agent-7045","agent_a_escrow_id":"escrow-three-agent-7045","agent_b_escrow_id":"escrow-three-agent-7045","verifier_escrow_id":"escrow-three-agent-7045","network":"solana:devnet","execution_surface":"command-override","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"escrow_id":"escrow-command-fixture","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_a_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_b_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","verifier_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","agent_a_settlement_commitment":"finalized","agent_b_settlement_commitment":"finalized","verifier_settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"agent_a_amount_lamports":1,"agent_b_amount_lamports":1,"verifier_amount_lamports":1,"agent_a_private_view_visible":true,"agent_b_private_view_visible":true,"verifier_private_view_visible":false,"agent_a_view_scope":"participant-private","agent_b_view_scope":"participant-private","verifier_view_scope":"restricted-public","agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"agent_a_private_view_digest":"agent-a-private-digest-7045","agent_b_private_view_digest":"agent-b-private-digest-7045","agent_a_public_view_digest":"public-view-digest-7045","agent_b_public_view_digest":"public-view-digest-7045","verifier_public_view_digest":"public-view-digest-7045","private_payload_redacted":true}"#;
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

fn roadmap_claim() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}
