use std::path::{Path, PathBuf};

pub(crate) const NO_THREE_AGENT_BOUNDARY: &str = "";

pub(crate) fn absent_boundary() -> &'static str {
    r#","three_agent_boundary":{"claim_status":"NOT_PRESENT","claim_label":"NOT_PRESENT","claim_present":false}"#
}

pub(crate) fn valid_boundary() -> &'static str {
    r#","three_agent_boundary":{"claim_status":"PASS","claim_label":"devnet-backed","claim_present":true,"agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"private_payload_redacted":true,"verifier_private_view_digest_present":false}"#
}

pub(crate) fn devnet_settlement_claim() -> &'static str {
    r#"{"id":"devnet_settlement_asset_movement","label":"devnet-backed","required":true,"status":"PASS","summary":"Solana devnet escrow settlement transfer observed","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111"}"#
}

pub(crate) fn three_agent_claim(root: &Path, transcript: &Path) -> String {
    format!(
        r#"{{"id":"three_agent_escrow_verification","label":"devnet-backed","required":true,"status":"PASS","summary":"Agent C verifies escrow settlement from restricted proof view","three_agent_transcript_artifact":"{}","three_agent_transcript_digest":"three-agent-transcript-digest-7045","transaction_id":"tx-three-agent-7045","terms_digest":"terms-digest-7045","agent_a_terms_digest":"terms-digest-7045","agent_b_terms_digest":"terms-digest-7045","verifier_terms_digest":"terms-digest-7045","escrow_id":"escrow-three-agent-7045","agent_a_escrow_id":"escrow-three-agent-7045","agent_b_escrow_id":"escrow-three-agent-7045","verifier_escrow_id":"escrow-three-agent-7045","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_a_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_b_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","verifier_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","agent_a_settlement_commitment":"finalized","agent_b_settlement_commitment":"finalized","verifier_settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"agent_a_amount_lamports":1,"agent_b_amount_lamports":1,"verifier_amount_lamports":1,"agent_a_private_view_visible":true,"agent_b_private_view_visible":true,"verifier_private_view_visible":false,"agent_a_view_scope":"participant-private","agent_b_view_scope":"participant-private","verifier_view_scope":"restricted-public","agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"agent_a_private_view_digest":"agent-a-private-digest-7045","agent_b_private_view_digest":"agent-b-private-digest-7045","agent_a_public_view_digest":"public-view-digest-7045","agent_b_public_view_digest":"public-view-digest-7045","verifier_public_view_digest":"public-view-digest-7045","private_payload_redacted":true{}}}"#,
        transcript.display(),
        view_fields(root)
    )
}

pub(crate) fn valid_transcript(root: &Path) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.three-agent-transcript.v1","proof_label":"local-only","devnet_settlement_linked":true,"transaction_id":"tx-three-agent-7045","escrow_id":"escrow-three-agent-7045","steps":["agent_a_registered","agent_b_registered","agent_a_invoked_transaction","agent_b_accepted_task","escrow_funded","escrow_released","agent_c_verified"],"views":{{"agent_a":"participant-private","agent_b":"participant-private","agent_c":"restricted-public"}},"agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"private_payload_redacted":true,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","transcript_digest":"three-agent-transcript-digest-7045"{} }}"#,
        view_fields(root)
    )
}

pub(crate) fn valid_view_artifacts(root: &Path) -> Vec<(PathBuf, String)> {
    vec![
        (
            root.join("proof/agent-a-view.json"),
            participant_view(
                "agent_a",
                "agent-a-private-digest-7045",
                "agent-a-view-digest-7045",
            ),
        ),
        (
            root.join("proof/agent-b-view.json"),
            participant_view(
                "agent_b",
                "agent-b-private-digest-7045",
                "agent-b-view-digest-7045",
            ),
        ),
        (
            root.join("proof/agent-c-verifier-view.json"),
            verifier_view(),
        ),
    ]
}

fn view_fields(root: &Path) -> String {
    format!(
        r#","agent_a_view_artifact":"{}","agent_b_view_artifact":"{}","agent_c_verifier_view_artifact":"{}","agent_a_view_digest":"agent-a-view-digest-7045","agent_b_view_digest":"agent-b-view-digest-7045","agent_c_verifier_view_digest":"agent-c-view-digest-7045""#,
        root.join("proof/agent-a-view.json").display(),
        root.join("proof/agent-b-view.json").display(),
        root.join("proof/agent-c-verifier-view.json").display()
    )
}

fn participant_view(agent: &str, private_digest: &str, view_digest: &str) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.three-agent-view.v1","agent":"{}","view_scope":"participant-private","transaction_id":"tx-three-agent-7045","escrow_id":"escrow-three-agent-7045","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","private_field_count":3,"participant_private_view_digest":"{}","public_view_digest":"public-view-digest-7045","private_payload_redacted":true,"view_digest":"{}"}}"#,
        agent, private_digest, view_digest
    )
}

fn verifier_view() -> String {
    r#"{"schema_version":"kamn.mvp.three-agent-view.v1","agent":"agent_c","view_scope":"restricted-public","transaction_id":"tx-three-agent-7045","escrow_id":"escrow-three-agent-7045","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","private_field_count":0,"public_view_digest":"public-view-digest-7045","private_payload_redacted":true,"view_digest":"agent-c-view-digest-7045"}"#.to_owned()
}
