use std::path::Path;

use kamn_e2e_harness::MvpDemoCommandConfig;

#[path = "live_task_evidence.rs"]
pub(crate) mod live_task_evidence;

pub(crate) fn local_demo_config(temp: &Path) -> MvpDemoCommandConfig {
    MvpDemoCommandConfig {
        output_root: temp.display().to_string(),
        devnet_mode: "optional".to_owned(),
        solana_rpc_url: None,
        devnet_settlement_command: None,
        localhost_signed_demo_command: Some(stub_localhost_signed_demo_command()),
        service_api_vertical_slice_command: Some(stub_service_api_command(
            "integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence",
        )),
        service_api_websocket_command: Some(stub_service_api_command(
            "integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event",
        )),
        agent_harness_evidence_path: None,
        live_task_evidence: None,
        pi_transaction_actor_paths: None,
    }
}

pub(crate) fn devnet_required_demo_config(temp: &Path) -> MvpDemoCommandConfig {
    let mut config = local_demo_config(temp);
    config.devnet_mode = "required".to_owned();
    config.solana_rpc_url = Some("https://api.devnet.solana.com".to_owned());
    config.devnet_settlement_command = Some(stub_devnet_settlement_command());
    config.live_task_evidence = Some(live_task_evidence::write(temp));
    config
}

#[allow(dead_code)]
pub(crate) fn devnet_required_without_task_binding(temp: &Path) -> MvpDemoCommandConfig {
    let mut config = devnet_required_demo_config(temp);
    config.live_task_evidence = None;
    config
}

fn stub_localhost_signed_demo_command() -> Vec<String> {
    vec![
        "sh".to_owned(),
        "-c".to_owned(),
        r#"cat > "$2" <<'JSON'
{"schema_version":"kamn.sdk.localhost-signed.demo-receipt-artifact.v1","status": "pass","signed_exchange":{"from":"kamn:did:agent:alice","to":"kamn:did:agent:bob","verified": true},"signed_flow":"task"}
JSON
echo "receipt_reconciliation=GO"
echo "localhost signed message demo completed."
"#
        .to_owned(),
        "kamn-mvp-stub".to_owned(),
    ]
}

fn stub_service_api_command(test_name: &str) -> Vec<String> {
    vec![
        "sh".to_owned(),
        "-c".to_owned(),
        format!(r#"echo "test {test_name} ... ok"; echo "test result: ok""#),
    ]
}

fn stub_devnet_settlement_command() -> Vec<String> {
    vec![
        "sh".to_owned(),
        "-c".to_owned(),
        r#"cat <<'JSON'
{"network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T","recipient_pubkey":"FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe","lamports":1000000,"escrow_id":"escrow-local-bound-7086","settlement_tx_signature":"devnet-signature-111","settlement_commitment":"finalized","payer_balance_before":2500000000,"payer_balance_after":2498995000,"recipient_balance_before":2500000000,"recipient_balance_after":2501000000,"persisted_settlement_tx_signature":"devnet-signature-111"}
JSON
"#
        .to_owned(),
    ]
}
