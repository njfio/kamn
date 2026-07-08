use kamn_e2e_harness::mvp_demo::verify_mvp_demo_report_json;

#[test]
fn spec_c01_verifier_rejects_devnet_report_without_three_agent_claim() {
    let err = verify_mvp_demo_report_json(devnet_report_without_three_agent_claim())
        .expect_err("devnet settlement report must include three-agent verification");
    assert!(err.contains("missing three-agent escrow verification claim"));
}

#[test]
fn spec_c02_verifier_accepts_matching_three_agent_views() {
    verify_mvp_demo_report_json(valid_three_agent_report())
        .expect("matching participant and verifier commitments should pass");
}

#[test]
fn spec_c03_verifier_rejects_verifier_private_context_leak() {
    let report = valid_three_agent_report().replace(
        r#""verifier_private_view_visible":false"#,
        r#""verifier_private_view_visible":true"#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("third-party verifier view must not expose private context");
    assert!(err.contains("verifier view must not expose private fields"));
}

#[test]
fn spec_c04_verifier_rejects_mismatched_shared_commitment() {
    let report = valid_three_agent_report().replace(
        r#""verifier_terms_digest":"terms-digest-7045""#,
        r#""verifier_terms_digest":"terms-digest-mismatch""#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("third-party verifier commitment mismatch must fail");
    assert!(err.contains("three-agent shared commitment mismatch"));
}

#[test]
fn spec_c05_verifier_rejects_local_only_three_agent_settlement_claim() {
    let report = valid_three_agent_report().replace(
        r#""id":"three_agent_escrow_verification","label":"devnet-backed""#,
        r#""id":"three_agent_escrow_verification","label":"local-only""#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("three-agent settlement claim must stay devnet-backed");
    assert!(err.contains("value movement claim must be devnet-backed"));
}

#[test]
fn spec_c06_verifier_rejects_missing_view_scope() {
    let report =
        valid_three_agent_report().replace(r#""agent_a_view_scope":"participant-private","#, "");
    let err = verify_mvp_demo_report_json(report)
        .expect_err("three-agent claim must expose participant view scopes");
    assert!(err.contains("missing claim field: agent_a_view_scope"));
}

#[test]
fn spec_c07_verifier_rejects_participant_without_private_evidence() {
    let report = valid_three_agent_report().replace(
        r#""agent_a_private_field_count":3"#,
        r#""agent_a_private_field_count":0"#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("participant view must be richer than verifier view");
    assert!(err.contains("participant views must include private evidence"));
}

#[test]
fn spec_c08_verifier_rejects_verifier_private_evidence() {
    let report = valid_three_agent_report().replace(
        r#""verifier_private_field_count":0"#,
        r#""verifier_private_field_count":1"#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("third-party verifier must not receive private evidence");
    assert!(err.contains("verifier view must not expose private fields"));
}

#[test]
fn spec_c09_verifier_rejects_verifier_private_digest_leak() {
    let report = valid_three_agent_report().replace(
        r#""private_payload_redacted":true"#,
        r#""verifier_private_view_digest":"verifier-private-leak","private_payload_redacted":true"#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("third-party verifier must not receive a private digest");
    assert!(err.contains("verifier view must not expose private digest"));
}

#[test]
fn spec_c10_verifier_rejects_unredacted_private_payloads() {
    let report = valid_three_agent_report().replace(
        r#""private_payload_redacted":true"#,
        r#""private_payload_redacted":false"#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("three-agent claim must explicitly redact private payloads");
    assert!(err.contains("private payloads must be redacted"));
}

#[test]
fn spec_c11_verifier_rejects_mismatched_public_view_digest() {
    let report = valid_three_agent_report().replace(
        r#""verifier_public_view_digest":"public-view-digest-7045""#,
        r#""verifier_public_view_digest":"public-view-digest-mismatch""#,
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("verifier must validate the same public commitment");
    assert!(err.contains("three-agent public view digest mismatch"));
}

fn valid_three_agent_report() -> String {
    report_with_claims(&[
        local_claims(),
        devnet_settlement_claim(),
        three_agent_claim(),
        roadmap_claim(),
    ])
}

fn devnet_report_without_three_agent_claim() -> String {
    report_with_claims(&[local_claims(), devnet_settlement_claim(), roadmap_claim()])
}

fn report_with_claims(claims: &[&str]) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-three-agent","status":"GO","devnet_mode":"required","artifacts":{},"claim_matrix":[{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(),
        claims.join(",")
    )
}

fn artifacts_json() -> &'static str {
    r#"{"report_json":".kamn/demo/latest/proof/report.json","report_md":".kamn/demo/latest/proof/report.md","state_dir":".kamn/demo/demo-three-agent/state","audit_export":".kamn/demo/demo-three-agent/proof/audit-export.json","localhost_signed_demo_artifact":".kamn/demo/demo-three-agent/proof/localhost-signed-demo.json","localhost_signed_demo_output":".kamn/demo/demo-three-agent/proof/localhost-signed-demo-output.txt","service_api_vertical_slice_output":".kamn/demo/demo-three-agent/proof/service-api-vertical-slice-output.txt","service_api_websocket_output":".kamn/demo/demo-three-agent/proof/service-api-websocket-output.txt","devnet_settlement_output":".kamn/demo/demo-three-agent/proof/devnet-settlement-output.txt"}"#
}

fn local_claims() -> &'static str {
    r#"{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local KAMN runtime startup recorded"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"Alice and Bob signed request identities recorded"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"signed task flow recorded"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state written under demo run directory"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection visible"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket event visibility recorded"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit proof export recorded"}"#
}

fn devnet_settlement_claim() -> &'static str {
    r#"{"id":"devnet_settlement_asset_movement","label":"devnet-backed","required":true,"status":"PASS","summary":"Solana devnet escrow settlement transfer observed","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111"}"#
}

fn three_agent_claim() -> &'static str {
    r#"{"id":"three_agent_escrow_verification","label":"devnet-backed","required":true,"status":"PASS","summary":"Agent C verifies escrow settlement from restricted proof view","transaction_id":"tx-three-agent-7045","terms_digest":"terms-digest-7045","agent_a_terms_digest":"terms-digest-7045","agent_b_terms_digest":"terms-digest-7045","verifier_terms_digest":"terms-digest-7045","escrow_id":"escrow-three-agent-7045","agent_a_escrow_id":"escrow-three-agent-7045","agent_b_escrow_id":"escrow-three-agent-7045","verifier_escrow_id":"escrow-three-agent-7045","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_a_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_b_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","verifier_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","agent_a_settlement_commitment":"finalized","agent_b_settlement_commitment":"finalized","verifier_settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"agent_a_amount_lamports":1,"agent_b_amount_lamports":1,"verifier_amount_lamports":1,"agent_a_private_view_visible":true,"agent_b_private_view_visible":true,"verifier_private_view_visible":false,"agent_a_view_scope":"participant-private","agent_b_view_scope":"participant-private","verifier_view_scope":"restricted-public","agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"agent_a_private_view_digest":"agent-a-private-digest-7045","agent_b_private_view_digest":"agent-b-private-digest-7045","agent_a_public_view_digest":"public-view-digest-7045","agent_b_public_view_digest":"public-view-digest-7045","verifier_public_view_digest":"public-view-digest-7045","private_payload_redacted":true}"#
}

fn roadmap_claim() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}
