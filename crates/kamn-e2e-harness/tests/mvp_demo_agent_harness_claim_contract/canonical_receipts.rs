use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[path = "../support/artifact_digest.rs"]
mod artifact_digest;

use artifact_digest::{digest_field, with_digest};

use super::three_agent::view_digest_for;

pub(crate) fn valid_receipt_artifacts(root: &Path) -> Vec<(PathBuf, String)> {
    vec![
        (
            root.join("proof/agent-a-observation-receipt.json"),
            participant_receipt(root, "agent_a", "register_and_invoke_transaction"),
        ),
        (
            root.join("proof/agent-b-observation-receipt.json"),
            participant_receipt(root, "agent_b", "register_and_accept_task"),
        ),
        (
            root.join("proof/agent-c-verifier-observation-receipt.json"),
            verifier_receipt(root),
        ),
    ]
}

pub(crate) fn receipt_fields(root: &Path) -> String {
    format!(
        r#","agent_a_observation_receipt_artifact":"{}","agent_b_observation_receipt_artifact":"{}","agent_c_verifier_observation_receipt_artifact":"{}","agent_a_observation_receipt_digest":"{}","agent_b_observation_receipt_digest":"{}","agent_c_verifier_observation_receipt_digest":"{}""#,
        root.join("proof/agent-a-observation-receipt.json")
            .display(),
        root.join("proof/agent-b-observation-receipt.json")
            .display(),
        root.join("proof/agent-c-verifier-observation-receipt.json")
            .display(),
        receipt_digest_for(root, "agent_a"),
        receipt_digest_for(root, "agent_b"),
        receipt_digest_for(root, "agent_c_verifier")
    )
}

pub(crate) fn artifact_entries(root: &Path) -> String {
    format!(
        r#","agent_a_observation_receipt":"{}","agent_b_observation_receipt":"{}","agent_c_verifier_observation_receipt":"{}""#,
        root.join("proof/agent-a-observation-receipt.json")
            .display(),
        root.join("proof/agent-b-observation-receipt.json")
            .display(),
        root.join("proof/agent-c-verifier-observation-receipt.json")
            .display()
    )
}

fn participant_receipt(root: &Path, agent: &str, action: &str) -> String {
    let (file, private_digest) = match agent {
        "agent_a" => ("agent-a-view.json", "agent-a-private-digest-7045"),
        _ => ("agent-b-view.json", "agent-b-private-digest-7045"),
    };
    with_digest(
        format!(
            r#"{{"schema_version":"kamn.mvp.three-agent-observation-receipt.v1","agent":"{}","action":"{}","view_scope":"participant-private","transaction_id":"tx-three-agent-7045","escrow_id":"escrow-three-agent-7045","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","view_artifact":"{}","view_digest":"{}","participant_private_view_digest":"{}","public_view_digest":"public-view-digest-7045","private_payload_redacted":true,"receipt_digest":""}}"#,
            agent,
            action,
            root.join("proof").join(file).display(),
            view_digest_for(agent),
            private_digest
        ),
        "receipt_digest",
    )
}

fn verifier_receipt(root: &Path) -> String {
    with_digest(
        format!(
            r#"{{"schema_version":"kamn.mvp.three-agent-observation-receipt.v1","agent":"agent_c_verifier","action":"verify_three_agent_proof","view_scope":"restricted-public","transaction_id":"tx-three-agent-7045","escrow_id":"escrow-three-agent-7045","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","settlement_commitment":"finalized","view_artifact":"{}","view_digest":"{}","public_view_digest":"public-view-digest-7045","private_payload_redacted":true,"receipt_digest":""}}"#,
            root.join("proof/agent-c-verifier-view.json").display(),
            view_digest_for("agent_c_verifier")
        ),
        "receipt_digest",
    )
}

fn receipt_digest_for(root: &Path, agent: &str) -> String {
    let receipt = match agent {
        "agent_a" => participant_receipt(root, "agent_a", "register_and_invoke_transaction"),
        "agent_b" => participant_receipt(root, "agent_b", "register_and_accept_task"),
        _ => verifier_receipt(root),
    };
    digest_field(receipt.as_str(), "receipt_digest")
}
