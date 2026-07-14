use std::path::PathBuf;

const EVIDENCE: &str = "docs/validation/evidence/7123-fresh-checkout-evaluator-rehearsal.md";

#[test]
fn spec_c01_evidence_binds_fresh_clone_and_live_commands() {
    require(&[
        "decision: GO",
        "clone_source: origin/main",
        "clone_state: fresh",
        "inherited_target: false",
        "inherited_kamn_state: false",
        "make demo-agent-transaction",
        "verify-mvp-demo",
        "children_exited_before_verifier: true",
    ]);
}

#[test]
fn spec_c02_evidence_binds_new_independent_devnet_confirmation() {
    require(&[
        "devnet_mode: required",
        "commitment: finalized",
        "amount_lamports: 1000000",
        "transfer_count: 1",
        "retry_duplicate_count: 0",
        "rpc_verification: GO",
        "recipient_balance_delta_lamports: 1000000",
        "https://explorer.solana.com/tx/",
        "?cluster=devnet",
    ]);
}

#[test]
fn spec_c03_evidence_preserves_actor_and_disclosure_boundaries() {
    require(&[
        "actor_driver: pi",
        "distinct_authenticated_actor_count: 3",
        "agent_a_view: participant-private",
        "agent_b_view: participant-private",
        "agent_c_view: restricted-public",
        "agent_c_participant_private_field_count: 0",
        "task_lifecycle: GO",
        "escrow_lifecycle: GO",
        "durable_receipts: GO",
        "relay_projection: GO",
        "websocket_visibility: GO",
        "audit_export: GO",
    ]);
}

#[test]
fn spec_c04_evidence_is_secret_safe_and_claim_bounded() {
    let content = read_evidence();
    for forbidden in ["/Users/", "/private/", "PRIVATE KEY", "secret=", ".kamn/devnet/"] {
        assert!(!content.contains(forbidden), "forbidden marker `{forbidden}`");
    }
    for marker in ["not production", "not mainnet", "not custody", "devnet test tokens"] {
        assert!(content.contains(marker), "missing bounded marker `{marker}`");
    }
}

fn require(markers: &[&str]) {
    let content = read_evidence();
    for marker in markers {
        assert!(content.contains(marker), "{EVIDENCE} is missing `{marker}`");
    }
}

fn read_evidence() -> String {
    std::fs::read_to_string(repo_root().join(EVIDENCE))
        .unwrap_or_else(|error| panic!("{EVIDENCE} should be readable: {error}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
