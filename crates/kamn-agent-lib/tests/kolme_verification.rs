use kamn_agent_lib::kolme::{KolmeClient, KolmeProofReceipt};

#[test]
fn spec_c06_kolme_verification_reports_finality_for_verified_receipt() {
    let client = KolmeClient::new("http://localhost:3000").expect("client");
    let receipt = KolmeProofReceipt {
        tx_hash: "sha256:abc".to_owned(),
        block_height: 42,
        finality: "FINAL".to_owned(),
    };

    let result = client
        .verify_proof("msg-123", &receipt)
        .expect("verification result");

    assert_eq!(result.message_id, "msg-123");
    assert_eq!(result.block_height, 42);
    assert_eq!(result.finality, "FINAL");
    assert!(result.verified);
}
