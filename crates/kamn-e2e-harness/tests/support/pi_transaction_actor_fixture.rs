use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

#[derive(Clone)]
pub(crate) struct Overrides {
    pub(crate) agent_c_pid: u64,
    pub(crate) agent_c_did: &'static str,
    pub(crate) agent_c_projection: String,
    pub(crate) agent_c_private: Option<String>,
    pub(crate) agent_b_escrow: &'static str,
    pub(crate) agent_a_handoff_authorized: bool,
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
        }
    }
}

pub(crate) struct ActorFixture {
    root: PathBuf,
}

impl ActorFixture {
    pub(crate) fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kamn-pi-actors-{nanos}"));
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
            .with_handoff_authorized(overrides.agent_a_handoff_authorized);
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

struct ActorInput<'a> {
    role: &'a str,
    pid: u64,
    did: &'a str,
    escrow: &'a str,
    projection: String,
    private: Option<String>,
    handoff_authorized: bool,
}

impl<'a> ActorInput<'a> {
    fn new(role: &'a str, pid: u64, did: &'a str, escrow: &'a str, projection: char) -> Self {
        Self {
            role,
            pid,
            did,
            escrow,
            projection: sha(projection),
            private: None,
            handoff_authorized: false,
        }
    }

    fn with_private(mut self, value: String) -> Self {
        self.private = Some(value);
        self
    }

    fn with_handoff_authorized(mut self, value: bool) -> Self {
        self.handoff_authorized = value;
        self
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
    format!(
        r#"{{"schema_version":"kamn.mvp.pi-transaction-actor.v1","actor":"{}","pi_process_id":{},"did":"{}","mcp_child_process_id":{},"first_request_id":1,"last_request_id":5,"runtime_response_digests":["{responses}"],"runtime_projection_digest":"{}","task_id":"task-live-7099","transaction_id":"transaction-live-7099","escrow_id":"{}","amount_lamports":1000000,"network":"solana-devnet","settlement_tx_signature":"devnet-signature-7099","settlement_commitment":"finalized","public_commitment":"{}","view_scope":"{scope}"{private_field},"source_handoff_digest":"{}","handoff_authorized":{}}}"#,
        input.role,
        input.pid,
        input.did,
        input.pid + 1000,
        input.projection,
        input.escrow,
        sha('d'),
        sha('b'),
        input.handoff_authorized,
    )
}

fn response_digests(input: &ActorInput<'_>) -> [String; 5] {
    let projection = if input.projection == sha('f') {
        sha('3')
    } else {
        input.projection.clone()
    };
    [sha('a'), sha('b'), sha('c'), sha('d'), projection]
}

pub(crate) fn sha(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}
