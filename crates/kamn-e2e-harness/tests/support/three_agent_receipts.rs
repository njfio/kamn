use std::path::{Path, PathBuf};

use crate::{artifact_digest, mvp_local_artifacts, three_agent_view_artifacts};

pub(crate) fn base_fixture(stem: &str) -> PathBuf {
    let root = temp_root(stem);
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_view_artifacts(&root, None);
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(Some(&root)),
    );
    root
}

pub(crate) fn report_with_receipts(root: &Path, paths: &ReceiptPaths) -> String {
    three_agent_view_artifacts::report_json(root, Some(root)).replace(
        r#""private_payload_redacted":true"#,
        format!(
            r#""private_payload_redacted":true,{}"#,
            observation_receipt_fields(paths)
        )
        .as_str(),
    )
}

pub(crate) fn write_receipts(root: &Path, overrides: ReceiptOverrides) -> ReceiptPaths {
    let paths = ReceiptPaths::new(root);
    write_receipt(root, paths.agent_a.as_path(), agent_a_receipt, &overrides);
    write_receipt(root, paths.agent_b.as_path(), agent_b_receipt, &overrides);
    write_receipt(root, paths.agent_c.as_path(), agent_c_receipt, &overrides);
    paths
}

pub(crate) fn tamper_json_file(path: &Path, marker: &str) {
    let raw = std::fs::read_to_string(path).expect("receipt should be readable");
    let tampered = raw
        .strip_suffix('}')
        .map(|prefix| format!("{prefix},\"tamper_marker\":\"{marker}\"}}"))
        .expect("receipt should be a JSON object");
    mvp_local_artifacts::write_file(path, tampered.as_str());
}

pub(crate) fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    mvp_local_artifacts::write_file(path.as_path(), report.as_str());
    path
}

pub(crate) fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!(
        "kamn-7070-{stem}-{}-{millis}",
        std::process::id()
    ))
}

fn observation_receipt_fields(paths: &ReceiptPaths) -> String {
    format!(
        r#""agent_a_observation_receipt_artifact":"{}","agent_b_observation_receipt_artifact":"{}","agent_c_verifier_observation_receipt_artifact":"{}","agent_a_observation_receipt_digest":"{}","agent_b_observation_receipt_digest":"{}","agent_c_verifier_observation_receipt_digest":"{}""#,
        paths.agent_a.display(),
        paths.agent_b.display(),
        paths.agent_c.display(),
        receipt_digest(paths.agent_a.as_path()),
        receipt_digest(paths.agent_b.as_path()),
        receipt_digest(paths.agent_c.as_path())
    )
}

fn write_receipt(
    root: &Path,
    path: &Path,
    receipt: fn(&Path, &ReceiptOverrides) -> String,
    overrides: &ReceiptOverrides,
) {
    let raw = artifact_digest::with_digest(receipt(root, overrides), "receipt_digest");
    mvp_local_artifacts::write_file(path, raw.as_str());
}

fn agent_a_receipt(root: &Path, overrides: &ReceiptOverrides) -> String {
    participant_receipt(
        root,
        "agent_a",
        "register_and_invoke_transaction",
        "agent-a-view.json",
        "agent-a-private-digest-7060",
        overrides.agent_a_view_digest.as_deref(),
    )
}

fn agent_b_receipt(root: &Path, _overrides: &ReceiptOverrides) -> String {
    participant_receipt(
        root,
        "agent_b",
        "register_and_accept_task",
        "agent-b-view.json",
        "agent-b-private-digest-7060",
        None,
    )
}

fn agent_c_receipt(root: &Path, overrides: &ReceiptOverrides) -> String {
    let private = if overrides.agent_c_private_digest {
        r#","participant_private_view_digest":"leak""#
    } else {
        ""
    };
    format!(
        r#"{{"schema_version":"kamn.mvp.three-agent-observation-receipt.v1","agent":"agent_c_verifier","action":"verify_three_agent_proof","view_scope":"restricted-public","transaction_id":"tx-three-agent-7060","escrow_id":"escrow-three-agent-7060","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"view_artifact":"{}","view_digest":"{}","public_view_digest":"public-view-digest-7060","private_payload_redacted":true{},"receipt_digest":""}}"#,
        root.join("proof/agent-c-verifier-view.json").display(),
        view_digest(root, "agent-c-verifier-view.json"),
        private
    )
}

fn participant_receipt(
    root: &Path,
    agent: &str,
    action: &str,
    view_file: &str,
    private_digest: &str,
    view_digest_override: Option<&str>,
) -> String {
    let digest = view_digest_override
        .map(str::to_owned)
        .unwrap_or_else(|| view_digest(root, view_file));
    format!(
        r#"{{"schema_version":"kamn.mvp.three-agent-observation-receipt.v1","agent":"{}","action":"{}","view_scope":"participant-private","transaction_id":"tx-three-agent-7060","escrow_id":"escrow-three-agent-7060","settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"view_artifact":"{}","view_digest":"{}","participant_private_view_digest":"{}","public_view_digest":"public-view-digest-7060","private_payload_redacted":true,"receipt_digest":""}}"#,
        agent,
        action,
        root.join("proof").join(view_file).display(),
        digest,
        private_digest
    )
}

fn view_digest(root: &Path, file: &str) -> String {
    let view = std::fs::read_to_string(root.join("proof").join(file))
        .expect("view artifact should be readable");
    artifact_digest::digest_field(view.as_str(), "view_digest")
}

fn receipt_digest(path: &Path) -> String {
    let receipt = std::fs::read_to_string(path).expect("receipt should be readable");
    artifact_digest::digest_field(receipt.as_str(), "receipt_digest")
}

pub(crate) struct ReceiptPaths {
    pub(crate) agent_a: PathBuf,
    pub(crate) agent_b: PathBuf,
    pub(crate) agent_c: PathBuf,
}

impl ReceiptPaths {
    fn new(root: &Path) -> Self {
        Self {
            agent_a: root.join("proof/agent-a-observation-receipt.json"),
            agent_b: root.join("proof/agent-b-observation-receipt.json"),
            agent_c: root.join("proof/agent-c-verifier-observation-receipt.json"),
        }
    }
}

#[derive(Default)]
pub(crate) struct ReceiptOverrides {
    pub(crate) agent_a_view_digest: Option<String>,
    pub(crate) agent_c_private_digest: bool,
}
