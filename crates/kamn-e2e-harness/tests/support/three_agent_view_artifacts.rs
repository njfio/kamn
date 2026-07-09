use std::path::{Path, PathBuf};

use crate::mvp_local_artifacts;

const SIGNATURE: &str = "5nSgnDevnetSignature111111111111111111111111111";

pub(crate) fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!("kamn-7060-{stem}-{}-{millis}", std::process::id()))
}

pub(crate) fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    mvp_local_artifacts::write_file(path.as_path(), report.as_str());
    path
}

pub(crate) fn write_transcript(root: &Path, transcript: String) {
    mvp_local_artifacts::write_file(
        root.join("proof/three-agent-transcript.json").as_path(),
        transcript.as_str(),
    );
}

pub(crate) fn write_view_artifacts(root: &Path, agent_c: Option<String>) {
    mvp_local_artifacts::write_file(
        root.join("proof/agent-a-view.json").as_path(),
        participant_view(
            "agent_a",
            "agent-a-private-digest-7060",
            "agent-a-view-digest-7060",
        )
        .as_str(),
    );
    mvp_local_artifacts::write_file(
        root.join("proof/agent-b-view.json").as_path(),
        participant_view(
            "agent_b",
            "agent-b-private-digest-7060",
            "agent-b-view-digest-7060",
        )
        .as_str(),
    );
    mvp_local_artifacts::write_file(
        root.join("proof/agent-c-verifier-view.json").as_path(),
        agent_c.unwrap_or_else(agent_c_public_view).as_str(),
    );
}

pub(crate) fn replace_agent_a_view(root: &Path, view: String) {
    mvp_local_artifacts::write_file(
        root.join("proof/agent-a-view.json").as_path(),
        view.as_str(),
    );
}

pub(crate) fn replace_agent_b_view(root: &Path, view: String) {
    mvp_local_artifacts::write_file(
        root.join("proof/agent-b-view.json").as_path(),
        view.as_str(),
    );
}

pub(crate) fn report_json(root: &Path, views_root: Option<&Path>) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-three-agent-views","status":"GO","devnet_mode":"required","artifacts":{},"claim_matrix":[{},{},{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(root, views_root),
        local_claims(),
        devnet_settlement_claim(),
        three_agent_claim(root, views_root),
        roadmap_claim()
    )
}

pub(crate) fn transcript(views_root: Option<&Path>) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.three-agent-transcript.v1","proof_label":"local-only","devnet_settlement_linked":true,"transaction_id":"tx-three-agent-7060","escrow_id":"escrow-three-agent-7060","steps":["agent_a_registered","agent_b_registered","agent_a_invoked_transaction","agent_b_accepted_task","escrow_funded","escrow_released","agent_c_verifier_verified"],"views":{{"agent_a":"participant-private","agent_b":"participant-private","agent_c_verifier":"restricted-public"}},"agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"private_payload_redacted":true,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","transcript_digest":"three-agent-transcript-digest-7060"{} }}"#,
        views_root.map(shared_view_fields).unwrap_or_default()
    )
}

pub(crate) fn agent_c_private_view() -> String {
    agent_c_public_view().replace(
        r#""view_scope":"restricted-public""#,
        r#""view_scope":"participant-private","participant_private_view_digest":"leaked""#,
    )
}

pub(crate) fn agent_c_mismatched_signature_view() -> String {
    agent_c_public_view().replace(SIGNATURE, "mismatch")
}

pub(crate) fn agent_c_short_identity_view() -> String {
    agent_c_public_view().replace(r#""agent":"agent_c_verifier""#, r#""agent":"agent_c""#)
}

pub(crate) fn agent_a_mismatched_identity_view() -> String {
    participant_view(
        "agent_b",
        "agent-a-private-digest-7060",
        "agent-a-view-digest-7060",
    )
}

pub(crate) fn agent_b_mismatched_identity_view() -> String {
    participant_view(
        "agent_a",
        "agent-b-private-digest-7060",
        "agent-b-view-digest-7060",
    )
}

fn artifacts_json(root: &Path, views_root: Option<&Path>) -> String {
    let transcript = root.join("proof/three-agent-transcript.json");
    let mut artifacts = mvp_local_artifacts::artifacts_json(root, Some(transcript.as_path()));
    if let Some(view_root) = views_root {
        artifacts.pop();
        artifacts.push_str(
            format!(
                r#","agent_a_view":"{}","agent_b_view":"{}","agent_c_verifier_view":"{}"}}"#,
                view_root.join("proof/agent-a-view.json").display(),
                view_root.join("proof/agent-b-view.json").display(),
                view_root.join("proof/agent-c-verifier-view.json").display()
            )
            .as_str(),
        );
    }
    artifacts
}

fn devnet_settlement_claim() -> &'static str {
    r#"{"id":"devnet_settlement_asset_movement","label":"devnet-backed","required":true,"status":"PASS","summary":"Solana devnet escrow settlement transfer observed","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111"}"#
}

fn three_agent_claim(root: &Path, views_root: Option<&Path>) -> String {
    format!(
        r#"{{"id":"three_agent_escrow_verification","label":"devnet-backed","required":true,"status":"PASS","summary":"Agent C verifies escrow settlement from restricted proof view","three_agent_transcript_artifact":"{}","three_agent_transcript_digest":"three-agent-transcript-digest-7060","transaction_id":"tx-three-agent-7060","terms_digest":"terms-digest-7060","agent_a_terms_digest":"terms-digest-7060","agent_b_terms_digest":"terms-digest-7060","verifier_terms_digest":"terms-digest-7060","escrow_id":"escrow-three-agent-7060","agent_a_escrow_id":"escrow-three-agent-7060","agent_b_escrow_id":"escrow-three-agent-7060","verifier_escrow_id":"escrow-three-agent-7060","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_a_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_b_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","verifier_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","agent_a_settlement_commitment":"finalized","agent_b_settlement_commitment":"finalized","verifier_settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"agent_a_amount_lamports":1,"agent_b_amount_lamports":1,"verifier_amount_lamports":1,"agent_a_private_view_visible":true,"agent_b_private_view_visible":true,"verifier_private_view_visible":false,"agent_a_view_scope":"participant-private","agent_b_view_scope":"participant-private","verifier_view_scope":"restricted-public","agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"agent_a_private_view_digest":"agent-a-private-digest-7060","agent_b_private_view_digest":"agent-b-private-digest-7060","agent_a_public_view_digest":"public-view-digest-7060","agent_b_public_view_digest":"public-view-digest-7060","verifier_public_view_digest":"public-view-digest-7060","private_payload_redacted":true{}}}"#,
        root.join("proof/three-agent-transcript.json").display(),
        views_root.map(shared_view_fields).unwrap_or_default()
    )
}

fn shared_view_fields(root: &Path) -> String {
    format!(
        r#","agent_a_view_artifact":"{}","agent_b_view_artifact":"{}","agent_c_verifier_view_artifact":"{}","agent_a_view_digest":"agent-a-view-digest-7060","agent_b_view_digest":"agent-b-view-digest-7060","agent_c_verifier_view_digest":"agent-c-view-digest-7060""#,
        root.join("proof/agent-a-view.json").display(),
        root.join("proof/agent-b-view.json").display(),
        root.join("proof/agent-c-verifier-view.json").display()
    )
}

fn participant_view(agent: &str, private_digest: &str, view_digest: &str) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.three-agent-view.v1","agent":"{}","view_scope":"participant-private","transaction_id":"tx-three-agent-7060","escrow_id":"escrow-three-agent-7060","settlement_tx_signature":"{}","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","private_field_count":3,"participant_private_view_digest":"{}","public_view_digest":"public-view-digest-7060","private_payload_redacted":true,"view_digest":"{}"}}"#,
        agent, SIGNATURE, private_digest, view_digest
    )
}

fn agent_c_public_view() -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.three-agent-view.v1","agent":"agent_c_verifier","view_scope":"restricted-public","transaction_id":"tx-three-agent-7060","escrow_id":"escrow-three-agent-7060","settlement_tx_signature":"{}","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","private_field_count":0,"public_view_digest":"public-view-digest-7060","private_payload_redacted":true,"view_digest":"agent-c-view-digest-7060"}}"#,
        SIGNATURE
    )
}

fn local_claims() -> &'static str {
    r#"{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local runtime"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"agent identities"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"message flow"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket events"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit export"}"#
}

fn roadmap_claim() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}
