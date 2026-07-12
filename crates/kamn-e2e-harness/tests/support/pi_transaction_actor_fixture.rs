use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

#[path = "pi_transaction_receipt_fixture.rs"]
mod receipt_fixture;
pub(crate) use receipt_fixture::sha;
use receipt_fixture::{response_digests, runtime_receipts, ActorInput};

#[derive(Clone)]
pub(crate) struct Overrides {
    pub(crate) agent_c_pid: u64,
    pub(crate) agent_c_did: &'static str,
    pub(crate) agent_c_projection: String,
    pub(crate) agent_c_private: Option<String>,
    pub(crate) agent_b_escrow: &'static str,
    pub(crate) agent_a_handoff_authorized: bool,
    pub(crate) agent_a_handoff_as_string: bool,
    pub(crate) agent_a_include_release: bool,
    pub(crate) agent_a_duplicate_fund: bool,
    pub(crate) agent_a_release_error: bool,
    pub(crate) agent_a_receipt_digest_mismatch: bool,
}

impl Default for Overrides {
    fn default() -> Self {
        Self {
            agent_c_pid: 303,
            agent_c_did: "kamn:did:c",
            agent_c_projection: sha('3'),
            agent_c_private: None,
            agent_b_escrow: "escrow-live-7099",
            agent_a_handoff_authorized: false,
            agent_a_handoff_as_string: false,
            agent_a_include_release: true,
            agent_a_duplicate_fund: false,
            agent_a_release_error: false,
            agent_a_receipt_digest_mismatch: false,
        }
    }
}

pub(crate) struct ActorFixture {
    root: PathBuf,
}

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

impl ActorFixture {
    pub(crate) fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("kamn-pi-actors-{nanos}-{sequence}"));
        std::fs::create_dir_all(&root).expect("fixture directory");
        Self { root }
    }

    pub(crate) fn paths(&self) -> [String; 3] {
        ["agent-a.json", "agent-b.json", "agent-c.json"]
            .map(|name| self.root.join(name).display().to_string())
    }

    pub(crate) fn write_all(&self, overrides: Overrides) {
        self.write_a(&overrides);
        self.write_b(&overrides);
        self.write_c(overrides);
    }

    fn write_a(&self, overrides: &Overrides) {
        let input = ActorInput::new("agent_a", 101, "kamn:did:a", "escrow-live-7099", '1')
            .with_private(sha('e'))
            .with_handoff_authorized(overrides.agent_a_handoff_authorized)
            .with_handoff_as_string(overrides.agent_a_handoff_as_string)
            .with_release(overrides.agent_a_include_release)
            .with_duplicate_fund(overrides.agent_a_duplicate_fund)
            .with_release_error(overrides.agent_a_release_error)
            .with_receipt_digest_mismatch(overrides.agent_a_receipt_digest_mismatch);
        write_actor(&self.root.join("agent-a.json"), input);
    }

    fn write_b(&self, overrides: &Overrides) {
        let input = ActorInput::new("agent_b", 202, "kamn:did:b", overrides.agent_b_escrow, '2')
            .with_private(sha('e'));
        write_actor(&self.root.join("agent-b.json"), input);
    }

    fn write_c(&self, overrides: Overrides) {
        let mut input = ActorInput::new(
            "agent_c",
            overrides.agent_c_pid,
            overrides.agent_c_did,
            "escrow-live-7099",
            '3',
        );
        input.projection = overrides.agent_c_projection;
        input.private = overrides.agent_c_private;
        write_actor(&self.root.join("agent-c.json"), input);
    }
}

fn write_actor(path: &Path, input: ActorInput<'_>) {
    let unsigned = actor_json(&input);
    let digest = format!("sha256:{:x}", Sha256::digest(unsigned.as_bytes()));
    let artifact = format!(
        "{},\"artifact_digest\":\"{digest}\"}}",
        &unsigned[..unsigned.len() - 1]
    );
    std::fs::write(path, artifact).expect("write actor fixture");
}

fn actor_json(input: &ActorInput<'_>) -> String {
    let scope = if input.role == "agent_c" {
        "restricted-public"
    } else {
        "participant-private"
    };
    let private_field = input
        .private
        .as_ref()
        .map(|value| format!(r#","private_receipt_digest":"{value}""#))
        .unwrap_or_default();
    let responses = response_digests(input).join("\",\"");
    let receipts = runtime_receipts(input);
    let participant_role = if input.role == "agent_a" {
        "creator"
    } else {
        "provider"
    };
    let participant_field = if input.role == "agent_c" {
        String::new()
    } else {
        format!(r#","participant_role":"{participant_role}""#)
    };
    let handoff = if input.handoff_as_string {
        format!(r#""{}""#, input.handoff_authorized)
    } else {
        input.handoff_authorized.to_string()
    };
    format!(
        r#"{{"schema_version":"kamn.mvp.pi-transaction-actor.v1","actor":"{}","pi_process_id":{},"did":"{}","mcp_child_process_id":{},"first_request_id":1,"last_request_id":5,"runtime_response_digests":["{responses}"],"runtime_response_receipts":[{receipts}],"runtime_projection_digest":"{}","task_id":"task-live-7099","transaction_id":"transaction-live-7099","escrow_id":"{}","amount_lamports":1000000,"network":"solana-devnet","settlement_tx_signature":"devnet-signature-7099","settlement_commitment":"finalized","public_commitment":"{}","view_scope":"{scope}"{participant_field}{private_field},"source_handoff_digest":"{}","handoff_authorized":{handoff}}}"#,
        input.role,
        input.pid,
        input.did,
        input.pid + 1000,
        input.projection,
        input.escrow,
        sha('d'),
        sha('b'),
    )
}
