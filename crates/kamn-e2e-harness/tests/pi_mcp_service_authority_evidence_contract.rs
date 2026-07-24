use std::path::PathBuf;

const EVIDENCE: &str = "docs/validation/evidence/7126-fresh-checkout-pi-mcp-service-authority.md";
const FORBIDDEN: &[&str] = &[
    "/Users/",
    "/private/",
    "PRIVATE KEY",
    "secret=",
    "KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE=",
];

#[test]
fn evidence_binds_fresh_checkout_and_canonical_commands() {
    require(&[
        "source_commit: 732b1d59027724632af16c4073ca4f1cbe27db31",
        "clone_state: fresh",
        "inherited_kamn_state: false",
        "run_id: run-63361-1784859146830",
        "make demo-agent-transaction: GO",
        "standalone_verifier: PASS",
        "children_exited_before_verifier: true",
    ]);
}

#[test]
fn evidence_binds_service_authority_and_actor_views() {
    require(&[
        "authority_schema: kamn.mcp.authority-receipt.v1",
        "receipt_chain_schema: kamn.service.receipt-chain.v1",
        "execution_surface: live-service-persisted-receipt",
        "agent_a_view: participant-private",
        "agent_b_view: participant-private",
        "agent_c_view: restricted-public",
        "agent_c_participant_private_field_count: 0",
    ]);
}

#[test]
fn evidence_binds_finalized_single_transfer() {
    require(&[
        "settlement_commitment: finalized",
        "amount_lamports: 1000000",
        "transfer_count: 1",
        "retry_duplicate_count: 0",
        "rpc_verification: GO",
        "receipt_chain_commitment: sha256:",
        "service_receipt_commitment: sha256:",
    ]);
}

#[test]
fn evidence_is_secret_safe_and_claim_bounded() {
    let content = read_evidence();
    for forbidden in FORBIDDEN {
        assert!(
            !content.contains(forbidden),
            "forbidden marker: {forbidden}"
        );
    }
    assert_contains_all(
        &content,
        &[
            "not production readiness",
            "not mainnet",
            "not custody",
            "not broad bridge finality",
            "not generalized external settlement",
            "devnet test tokens",
        ],
    );
}

fn require(markers: &[&str]) {
    assert_contains_all(&read_evidence(), markers);
}

fn assert_contains_all(content: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            content.contains(marker),
            "{EVIDENCE} missing marker: {marker}"
        );
    }
}

fn read_evidence() -> String {
    std::fs::read_to_string(repo_root().join(EVIDENCE))
        .unwrap_or_else(|error| panic!("{EVIDENCE} should be readable: {error}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
