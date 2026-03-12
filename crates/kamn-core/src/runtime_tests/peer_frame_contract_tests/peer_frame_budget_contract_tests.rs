use super::super::*;
use super::support::{peer_authenticator, peer_frame_budget, signed_peer_frame};

#[test]
fn performance_authenticated_peer_frame_validation_stays_within_ci_budget() {
    let budget_millis = peer_frame_budget();
    let mut authenticator = peer_authenticator();
    let started = Instant::now();
    for nonce in 1..=256 {
        let frame = signed_peer_frame(&format!("frame-{nonce}"), nonce, "payload-bounded");
        authenticator
            .validate_inbound(&frame)
            .expect("frame should be accepted");
    }
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis <= budget_millis,
        "authenticated peer frame validation exceeded CI budget: {elapsed_millis}ms (budget={budget_millis}ms)"
    );
}
