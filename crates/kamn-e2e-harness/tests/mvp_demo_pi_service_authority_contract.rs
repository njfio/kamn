use kamn_e2e_harness::verify_pi_transaction_actor_paths;

#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{ActorFixture, Overrides};

#[test]
fn spec_c01_accepts_role_scoped_v2_service_authority() {
    let fixture = ActorFixture::new();
    write_valid_service_authority_fixture(&fixture);

    let summary = verify_pi_transaction_actor_paths(&fixture.paths())
        .expect("v2 service authority should verify independently");
    assert!(summary.contains(r#""receipt_chain_commitment":"sha256:"#));
    assert!(summary.contains(r#""public_commitment":"sha256:"#));
}

#[test]
fn spec_c02_rejects_v1_client_local_receipt_authority() {
    let fixture = ActorFixture::new();
    fixture.write_v1_all(Overrides::default());

    let error = verify_pi_transaction_actor_paths(&fixture.paths())
        .expect_err("v1 Pi-local authority must not satisfy the canonical verifier");
    assert_eq!(error, "PI_SERVICE_AUTHORITY_MISMATCH");
}

fn write_valid_service_authority_fixture(fixture: &ActorFixture) {
    fixture.write_v2_all();
}
