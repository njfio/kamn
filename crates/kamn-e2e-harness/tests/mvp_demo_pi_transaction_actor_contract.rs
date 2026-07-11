use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use kamn_e2e_harness::verify_pi_transaction_actor_paths;
use sha2::{Digest, Sha256};
#[test]
fn spec_c01_rust_verifier_accepts_three_runtime_bound_pi_actors() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides::default());

    let summary = verify_pi_transaction_actor_paths(&fixture.paths())
        .expect("three independent actor artifacts should verify");
    assert!(summary.contains("task-live-7099"));
    assert!(summary.contains("escrow-live-7099"));
    assert!(summary.contains("devnet-signature-7099"));
}
#[test]
fn spec_c02_rust_verifier_rejects_reused_process_and_identity() {
    for overrides in [
        Overrides {
            agent_c_pid: 101,
            ..Overrides::default()
        },
        Overrides {
            agent_c_did: "kamn:did:a",
            ..Overrides::default()
        },
    ] {
        let fixture = ActorFixture::new();
        fixture.write_all(overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("reused actor identity must fail");
        assert!(error.contains("PI_ACTOR_"));
    }
}

#[test]
fn spec_c03_rust_verifier_rejects_runtime_privacy_and_shared_fact_drift() {
    for (overrides, code) in [
        (
            Overrides {
                agent_c_projection: sha('f'),
                ..Overrides::default()
            },
            "PI_RUNTIME_RECEIPT_MISMATCH",
        ),
        (
            Overrides {
                agent_c_private: Some(sha('e')),
                ..Overrides::default()
            },
            "PI_VERIFIER_PRIVATE_LEAK",
        ),
        (
            Overrides {
                agent_b_escrow: "escrow-other",
                ..Overrides::default()
            },
            "PI_TRANSACTION_FACT_MISMATCH",
        ),
        (
            Overrides {
                agent_a_handoff_authorized: true,
                ..Overrides::default()
            },
            "PI_HANDOFF_AUTHORIZATION_FORBIDDEN",
        ),
    ] {
        let fixture = ActorFixture::new();
        fixture.write_all(overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("tampered actor evidence must fail");
        assert!(error.contains(code), "unexpected error: {error}");
    }
}

#[derive(Clone)]
struct Overrides {
    agent_c_pid: u64,
    agent_c_did: &'static str,
    agent_c_projection: String,
    agent_c_private: Option<String>,
    agent_b_escrow: &'static str,
    agent_a_handoff_authorized: bool,
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

struct ActorFixture {
    root: PathBuf,
}

impl ActorFixture {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kamn-pi-actors-{nanos}"));
        std::fs::create_dir_all(&root).expect("fixture directory");
        Self { root }
    }

    fn paths(&self) -> [String; 3] {
        ["agent-a.json", "agent-b.json", "agent-c.json"]
            .map(|name| self.root.join(name).display().to_string())
    }

    fn write_all(&self, overrides: Overrides) {
        write_actor(
            &self.root.join("agent-a.json"),
            "agent_a",
            101,
            "kamn:did:a",
            "escrow-live-7099",
            sha('1'),
            Some(sha('e')),
            overrides.agent_a_handoff_authorized,
        );
        write_actor(
            &self.root.join("agent-b.json"),
            "agent_b",
            202,
            "kamn:did:b",
            overrides.agent_b_escrow,
            sha('2'),
            Some(sha('e')),
            false,
        );
        write_actor(
            &self.root.join("agent-c.json"),
            "agent_c",
            overrides.agent_c_pid,
            overrides.agent_c_did,
            "escrow-live-7099",
            overrides.agent_c_projection,
            overrides.agent_c_private,
            false,
        );
    }
}

fn write_actor(
    path: &Path,
    role: &str,
    pid: u64,
    did: &str,
    escrow: &str,
    projection: String,
    private: Option<String>,
    handoff_authorized: bool,
) {
    let scope = if role == "agent_c" {
        "restricted-public"
    } else {
        "participant-private"
    };
    let private_field = private
        .map(|value| format!(r#",\"private_receipt_digest\":\"{value}\""#))
        .unwrap_or_default();
    let responses = [
        sha('a'),
        sha('b'),
        sha('c'),
        sha('d'),
        if projection == sha('f') {
            sha('3')
        } else {
            projection.clone()
        },
    ]
    .join("\",\"");
    let unsigned = format!(
        r#"{{\"schema_version\":\"kamn.mvp.pi-transaction-actor.v1\",\"actor\":\"{role}\",\"pi_process_id\":{pid},\"did\":\"{did}\",\"mcp_child_process_id\":{},\"first_request_id\":1,\"last_request_id\":5,\"runtime_response_digests\":[\"{responses}\"],\"runtime_projection_digest\":\"{projection}\",\"task_id\":\"task-live-7099\",\"transaction_id\":\"transaction-live-7099\",\"escrow_id\":\"{escrow}\",\"amount_lamports\":1000000,\"network\":\"solana-devnet\",\"settlement_tx_signature\":\"devnet-signature-7099\",\"settlement_commitment\":\"finalized\",\"public_commitment\":\"{}\",\"view_scope\":\"{scope}\"{private_field},\"source_handoff_digest\":\"{}\",\"handoff_authorized\":{handoff_authorized}}}"#,
        pid + 1000,
        sha('d'),
        sha('b')
    );
    let digest = format!("sha256:{:x}", Sha256::digest(unsigned.as_bytes()));
    let artifact = format!(
        "{},\"artifact_digest\":\"{digest}\"}}",
        &unsigned[..unsigned.len() - 1]
    );
    std::fs::write(path, artifact).expect("write actor fixture");
}

fn sha(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}
