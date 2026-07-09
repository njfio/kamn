use kamn_e2e_harness::mvp_demo::{
    verify_mvp_demo_report_json, CLAIM_LABEL_DEVNET_BACKED, CLAIM_LABEL_DRY_RUN,
    CLAIM_LABEL_LOCAL_ONLY, CLAIM_LABEL_PLACEHOLDER, CLAIM_LABEL_REAL, CLAIM_LABEL_ROADMAP,
    MVP_DEMO_REPORT_SCHEMA_VERSION,
};

#[test]
fn spec_c01_claim_taxonomy_is_stable() {
    assert_eq!(
        MVP_DEMO_REPORT_SCHEMA_VERSION,
        "kamn.mvp.demo.proof-report.v1"
    );
    assert_eq!(CLAIM_LABEL_REAL, "real");
    assert_eq!(CLAIM_LABEL_DEVNET_BACKED, "devnet-backed");
    assert_eq!(CLAIM_LABEL_LOCAL_ONLY, "local-only");
    assert_eq!(CLAIM_LABEL_DRY_RUN, "dry-run");
    assert_eq!(CLAIM_LABEL_PLACEHOLDER, "placeholder");
    assert_eq!(CLAIM_LABEL_ROADMAP, "roadmap");
}

#[test]
fn spec_c02_verifier_accepts_valid_local_only_runtime_report() {
    verify_mvp_demo_report_json(valid_local_only_report())
        .expect("local-only runtime proof without settlement claim should verify");
}

#[test]
fn spec_c03_verifier_accepts_valid_devnet_backed_settlement_report() {
    verify_mvp_demo_report_json(valid_devnet_settlement_report())
        .expect("devnet-backed settlement proof should verify");
}

#[test]
fn spec_c04_verifier_rejects_required_placeholder_claim() {
    let err = verify_mvp_demo_report_json(local_report_with_label(
        "local_runtime_startup",
        "placeholder",
    ))
    .expect_err("placeholder required claim must fail");
    assert!(err.contains("required MVP claim cannot be placeholder"));
}

#[test]
fn spec_c05_verifier_rejects_required_dry_run_claim() {
    let err =
        verify_mvp_demo_report_json(local_report_with_label("durable_state_written", "dry-run"))
            .expect_err("dry-run required claim must fail");
    assert!(err.contains("required MVP claim cannot be dry-run"));
}

#[test]
fn spec_c06_verifier_rejects_settlement_claim_without_devnet_backing() {
    let err = verify_mvp_demo_report_json(settlement_report_with_label("local-only"))
        .expect_err("settlement claims must be devnet-backed");
    assert!(err.contains("value movement claim must be devnet-backed"));
}

#[test]
fn spec_c07_verifier_rejects_unknown_claim_label() {
    let err = verify_mvp_demo_report_json(local_report_with_label(
        "websocket_event_visibility",
        "maybe",
    ))
    .expect_err("unknown claim labels must fail");
    assert!(err.contains("unknown MVP claim label"));
}

#[test]
fn spec_c08_verifier_accepts_devnet_required_no_go_evidence() {
    verify_mvp_demo_report_json(devnet_required_no_go_report())
        .expect("explicit devnet-required NO-GO evidence should verify");
}

#[test]
fn spec_c09_verifier_rejects_malformed_report_json() {
    let malformed = valid_local_only_report().trim_end_matches('}').to_owned();
    let err =
        verify_mvp_demo_report_json(malformed).expect_err("malformed report JSON must be rejected");
    assert!(err.contains("malformed MVP demo report JSON"));
}

#[test]
fn spec_c10_verifier_rejects_missing_localhost_signed_artifact() {
    let report = valid_local_only_report().replace(
        r#""localhost_signed_demo_artifact":".kamn/demo/demo-local/proof/localhost-signed-demo.json","#,
        "",
    );
    let err = verify_mvp_demo_report_json(report)
        .expect_err("missing localhost signed artifact path must fail");
    assert!(err.contains("localhost signed demo artifact"));
}

#[test]
fn spec_c11_verifier_rejects_devnet_success_without_payer_balance_evidence() {
    let report = valid_devnet_settlement_report()
        .replace(r#""payer_balance_before":20,"#, "")
        .replace(r#""payer_balance_after":19,"#, "");
    let err = verify_mvp_demo_report_json(report)
        .expect_err("devnet settlement success must include payer balance evidence");
    assert!(err.contains("devnet-backed settlement evidence"));
}

fn valid_local_only_report() -> &'static str {
    r#"{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-local","status":"GO","devnet_mode":"optional","artifacts":{"report_json":".kamn/demo/latest/proof/report.json","report_md":".kamn/demo/latest/proof/report.md","state_dir":".kamn/demo/demo-local/state","audit_export":".kamn/demo/demo-local/proof/audit-export.json","localhost_signed_demo_artifact":".kamn/demo/demo-local/proof/localhost-signed-demo.json","localhost_signed_demo_output":".kamn/demo/demo-local/proof/localhost-signed-demo-output.txt","service_api_vertical_slice_output":".kamn/demo/demo-local/proof/service-api-vertical-slice-output.txt","service_api_websocket_output":".kamn/demo/demo-local/proof/service-api-websocket-output.txt","devnet_settlement_output":".kamn/demo/demo-local/proof/devnet-settlement-output.txt"},"claim_matrix":[{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local KAMN runtime startup recorded"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"Alice and Bob signed request identities recorded"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"signed task flow recorded"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state written under demo run directory"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection visible"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket event visibility recorded"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit proof export recorded"},{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}],"no_go":{"active":false,"reason":""}}"#
}

fn valid_devnet_settlement_report() -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-devnet","status":"GO","devnet_mode":"required","artifacts":{},"claim_matrix":[{},{},{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        devnet_artifacts_fixture(),
        local_claims_fixture(),
        devnet_settlement_claim_fixture(),
        three_agent_claim_fixture(),
        roadmap_claim_fixture()
    )
}

fn devnet_artifacts_fixture() -> &'static str {
    r#"{"report_json":".kamn/demo/latest/proof/report.json","report_md":".kamn/demo/latest/proof/report.md","state_dir":".kamn/demo/demo-devnet/state","audit_export":".kamn/demo/demo-devnet/proof/audit-export.json","localhost_signed_demo_artifact":".kamn/demo/demo-devnet/proof/localhost-signed-demo.json","localhost_signed_demo_output":".kamn/demo/demo-devnet/proof/localhost-signed-demo-output.txt","service_api_vertical_slice_output":".kamn/demo/demo-devnet/proof/service-api-vertical-slice-output.txt","service_api_websocket_output":".kamn/demo/demo-devnet/proof/service-api-websocket-output.txt","devnet_settlement_output":".kamn/demo/demo-devnet/proof/devnet-settlement-output.txt","three_agent_transcript":".kamn/demo/demo-devnet/proof/three-agent-transcript.json","agent_a_view":".kamn/demo/demo-devnet/proof/agent-a-view.json","agent_b_view":".kamn/demo/demo-devnet/proof/agent-b-view.json","agent_c_verifier_view":".kamn/demo/demo-devnet/proof/agent-c-verifier-view.json"}"#
}

fn local_claims_fixture() -> &'static str {
    r#"{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local KAMN runtime startup recorded"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"Alice and Bob signed request identities recorded"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"signed task flow recorded"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state written under demo run directory"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection visible"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket event visibility recorded"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit proof export recorded"}"#
}

fn devnet_settlement_claim_fixture() -> &'static str {
    r#"{"id":"devnet_settlement_asset_movement","label":"devnet-backed","required":true,"status":"PASS","summary":"Solana devnet escrow settlement transfer observed","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111"}"#
}

fn three_agent_claim_fixture() -> &'static str {
    r#"{"id":"three_agent_escrow_verification","label":"devnet-backed","required":true,"status":"PASS","summary":"Agent C verifies escrow settlement from restricted proof view","three_agent_transcript_artifact":".kamn/demo/demo-devnet/proof/three-agent-transcript.json","three_agent_transcript_digest":"three-agent-transcript-digest-7045","transaction_id":"tx-three-agent-7045","terms_digest":"terms-digest-7045","agent_a_terms_digest":"terms-digest-7045","agent_b_terms_digest":"terms-digest-7045","verifier_terms_digest":"terms-digest-7045","escrow_id":"escrow-three-agent-7045","agent_a_escrow_id":"escrow-three-agent-7045","agent_b_escrow_id":"escrow-three-agent-7045","verifier_escrow_id":"escrow-three-agent-7045","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"payer111111111111111111111111111111111111111","recipient_pubkey":"recipient11111111111111111111111111111111111","lamports":1,"settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_a_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","agent_b_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","verifier_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","settlement_commitment":"finalized","agent_a_settlement_commitment":"finalized","agent_b_settlement_commitment":"finalized","verifier_settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"5nSgnDevnetSignature111111111111111111111111111","amount_lamports":1,"agent_a_amount_lamports":1,"agent_b_amount_lamports":1,"verifier_amount_lamports":1,"agent_a_private_view_visible":true,"agent_b_private_view_visible":true,"verifier_private_view_visible":false,"agent_a_view_scope":"participant-private","agent_b_view_scope":"participant-private","verifier_view_scope":"restricted-public","agent_a_private_field_count":3,"agent_b_private_field_count":3,"verifier_private_field_count":0,"agent_a_private_view_digest":"agent-a-private-digest-7045","agent_b_private_view_digest":"agent-b-private-digest-7045","agent_a_public_view_digest":"public-view-digest-7045","agent_b_public_view_digest":"public-view-digest-7045","verifier_public_view_digest":"public-view-digest-7045","private_payload_redacted":true,"agent_a_view_artifact":".kamn/demo/demo-devnet/proof/agent-a-view.json","agent_b_view_artifact":".kamn/demo/demo-devnet/proof/agent-b-view.json","agent_c_verifier_view_artifact":".kamn/demo/demo-devnet/proof/agent-c-verifier-view.json","agent_a_view_digest":"agent-a-view-digest-7045","agent_b_view_digest":"agent-b-view-digest-7045","agent_c_verifier_view_digest":"agent-c-view-digest-7045"}"#
}

fn roadmap_claim_fixture() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}

fn devnet_required_no_go_report() -> &'static str {
    r#"{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-no-go","status":"NO-GO","devnet_mode":"required","artifacts":{"report_json":".kamn/demo/latest/proof/report.json","report_md":".kamn/demo/latest/proof/report.md","state_dir":".kamn/demo/demo-no-go/state","audit_export":".kamn/demo/demo-no-go/proof/audit-export.json","localhost_signed_demo_artifact":".kamn/demo/demo-no-go/proof/localhost-signed-demo.json","localhost_signed_demo_output":".kamn/demo/demo-no-go/proof/localhost-signed-demo-output.txt","service_api_vertical_slice_output":".kamn/demo/demo-no-go/proof/service-api-vertical-slice-output.txt","service_api_websocket_output":".kamn/demo/demo-no-go/proof/service-api-websocket-output.txt","devnet_settlement_output":".kamn/demo/demo-no-go/proof/devnet-settlement-output.txt"},"claim_matrix":[{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local KAMN runtime startup recorded"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"Alice and Bob signed request identities recorded"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"signed task flow recorded"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state written under demo run directory"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection visible"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket event visibility recorded"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit proof export recorded"},{"id":"devnet_settlement_no_go","label":"devnet-backed","required":true,"status":"NO-GO","summary":"Solana devnet escrow settlement evidence unavailable","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","no_go_reason":"devnet_keypair_not_configured"}],"no_go":{"active":true,"reason":"devnet_keypair_not_configured"}}"#
}

fn local_report_with_label(claim_id: &str, label: &str) -> String {
    for current in ["real", "local-only", "devnet-backed", "roadmap"] {
        let target = format!(r#""id":"{claim_id}","label":"{current}""#);
        if valid_local_only_report().contains(target.as_str()) {
            return valid_local_only_report().replace(
                target.as_str(),
                &format!(r#""id":"{claim_id}","label":"{label}""#),
            );
        }
    }
    valid_local_only_report().to_owned()
}

fn settlement_report_with_label(label: &str) -> String {
    valid_devnet_settlement_report().replace(
        r#""id":"devnet_settlement_asset_movement","label":"devnet-backed""#,
        &format!(r#""id":"devnet_settlement_asset_movement","label":"{label}""#),
    )
}
