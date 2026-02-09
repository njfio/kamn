use kamn_core::{
    BridgeOutboundRequest, BridgePlatform, CrossChainBridgeConfig, CrossChainBridgeEngine,
    CrossChainBridgeError, CrossChainNetwork, DiscordBridgeConfig, DiscordBridgeEngine,
    DiscordBridgeError,
};
use std::collections::{BTreeMap, BTreeSet};

const DISCORD_BRIDGE_DID: &str = "kamn:did:agent:bridge-discord-1";
const DISCORD_ROUTE_CHANNEL: &str = "discord:channel:ops";
const DISCORD_TARGET_DID: &str = "kamn:did:agent:listener-target-discord-1";

const CROSS_CHAIN_BRIDGE_DID: &str = "kamn:did:agent:bridge-crosschain-1";
const SOLANA_ROUTE_CHANNEL: &str = "solana:devnet:program:task-v1";
const SOLANA_TARGET_DID: &str = "kamn:did:agent:listener-target-sol";
const ETHEREUM_ROUTE_CHANNEL: &str = "ethereum:sepolia:contract:escrow-v1";
const ETHEREUM_TARGET_DID: &str = "kamn:did:agent:listener-target-eth";

const APPROVER_1: &str = "kamn:did:agent:approver-1";
const APPROVER_2: &str = "kamn:did:agent:approver-2";
const APPROVER_3: &str = "kamn:did:agent:approver-3";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutboundBridgeCase {
    Discord,
    CrossChainSolana,
}

#[derive(Debug, Clone, Copy)]
struct OutboundQuorumPassCase {
    case_id: &'static str,
    bridge: OutboundBridgeCase,
    request_id: &'static str,
    destination_channel_id: &'static str,
    approvers: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DispatchSnapshot {
    request_id: String,
    platform: BridgePlatform,
    required_approvals: usize,
    approved_by: Vec<String>,
}

const PASS_MATRIX: [OutboundQuorumPassCase; 2] = [
    OutboundQuorumPassCase {
        case_id: "discord-quorum-pass",
        bridge: OutboundBridgeCase::Discord,
        request_id: "discord-outbound-quorum-pass-1",
        destination_channel_id: DISCORD_ROUTE_CHANNEL,
        approvers: &[APPROVER_1, APPROVER_2],
    },
    OutboundQuorumPassCase {
        case_id: "solana-quorum-pass",
        bridge: OutboundBridgeCase::CrossChainSolana,
        request_id: "solana-outbound-quorum-pass-1",
        destination_channel_id: SOLANA_ROUTE_CHANNEL,
        approvers: &[APPROVER_1, APPROVER_2],
    },
];

const FAIL_CLASS_MATRIX: [&str; 5] = [
    "discord-under-quorum",
    "discord-unauthorized-approver",
    "discord-duplicate-approver",
    "cross-chain-under-quorum",
    "cross-chain-unauthorized-approver",
];

fn discord_config() -> DiscordBridgeConfig {
    let mut channel_routes = BTreeMap::new();
    channel_routes.insert(
        DISCORD_ROUTE_CHANNEL.to_owned(),
        DISCORD_TARGET_DID.to_owned(),
    );

    DiscordBridgeConfig {
        bridge_agent_did: DISCORD_BRIDGE_DID.to_owned(),
        authorized_listener_dids: ["kamn:did:agent:listener-discord-1".to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        authorized_approver_dids: [
            APPROVER_1.to_owned(),
            APPROVER_2.to_owned(),
            APPROVER_3.to_owned(),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>(),
        required_approvals: 2,
        channel_routes,
    }
}

fn cross_chain_config() -> CrossChainBridgeConfig {
    let mut ethereum_routes = BTreeMap::new();
    ethereum_routes.insert(
        ETHEREUM_ROUTE_CHANNEL.to_owned(),
        ETHEREUM_TARGET_DID.to_owned(),
    );

    let mut solana_routes = BTreeMap::new();
    solana_routes.insert(
        SOLANA_ROUTE_CHANNEL.to_owned(),
        SOLANA_TARGET_DID.to_owned(),
    );

    CrossChainBridgeConfig {
        bridge_agent_did: CROSS_CHAIN_BRIDGE_DID.to_owned(),
        authorized_listener_dids: ["kamn:did:agent:listener-crosschain-1".to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        authorized_approver_dids: [
            APPROVER_1.to_owned(),
            APPROVER_2.to_owned(),
            APPROVER_3.to_owned(),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>(),
        required_approvals: 2,
        ethereum_routes,
        solana_routes,
    }
}

fn outbound_request(request_id: &str, destination_channel_id: &str) -> BridgeOutboundRequest {
    BridgeOutboundRequest {
        request_id: request_id.to_owned(),
        from_agent_did: "kamn:did:agent:processor-1".to_owned(),
        destination_channel_id: destination_channel_id.to_owned(),
        body: "status:approved".to_owned(),
        created_at: "2026-02-09T12:00:00Z".to_owned(),
    }
}

fn dispatch_snapshot(case: &OutboundQuorumPassCase) -> DispatchSnapshot {
    let approvers = case
        .approvers
        .iter()
        .map(|approver| (*approver).to_owned())
        .collect::<Vec<_>>();
    match case.bridge {
        OutboundBridgeCase::Discord => {
            let engine = DiscordBridgeEngine::new(discord_config()).expect("engine should build");
            let dispatch = engine
                .process_outbound_with_approvals(
                    &outbound_request(case.request_id, case.destination_channel_id),
                    approvers,
                )
                .expect("discord quorum pass case should dispatch");
            let mut approved_by = dispatch
                .approval
                .approved_by
                .into_iter()
                .collect::<Vec<_>>();
            approved_by.sort();
            DispatchSnapshot {
                request_id: dispatch.envelope.request_id,
                platform: dispatch.envelope.platform,
                required_approvals: dispatch.approval.required_approvals,
                approved_by,
            }
        }
        OutboundBridgeCase::CrossChainSolana => {
            let engine =
                CrossChainBridgeEngine::new(cross_chain_config()).expect("engine should build");
            let dispatch = engine
                .process_outbound_with_approvals(
                    CrossChainNetwork::Solana,
                    &outbound_request(case.request_id, case.destination_channel_id),
                    approvers,
                )
                .expect("cross-chain quorum pass case should dispatch");
            let mut approved_by = dispatch
                .approval
                .approved_by
                .into_iter()
                .collect::<Vec<_>>();
            approved_by.sort();
            DispatchSnapshot {
                request_id: dispatch.envelope.request_id,
                platform: dispatch.envelope.platform,
                required_approvals: dispatch.approval.required_approvals,
                approved_by,
            }
        }
    }
}

#[test]
fn unit_outbound_quorum_matrix_covers_pass_and_fail_classes() {
    assert_eq!(PASS_MATRIX.len(), 2);
    assert_eq!(FAIL_CLASS_MATRIX.len(), 5);
    assert!(PASS_MATRIX
        .iter()
        .any(|case| case.bridge == OutboundBridgeCase::Discord));
    assert!(PASS_MATRIX
        .iter()
        .any(|case| case.bridge == OutboundBridgeCase::CrossChainSolana));
    assert!(FAIL_CLASS_MATRIX.contains(&"discord-under-quorum"));
    assert!(FAIL_CLASS_MATRIX.contains(&"cross-chain-unauthorized-approver"));
}

#[test]
fn functional_outbound_quorum_matrix_dispatches_deterministically() {
    for case in PASS_MATRIX {
        let first = dispatch_snapshot(&case);
        let second = dispatch_snapshot(&case);
        assert_eq!(
            first, second,
            "outbound quorum case {} must be deterministic",
            case.case_id
        );
        assert_eq!(first.request_id, case.request_id.to_owned());
        assert_eq!(first.required_approvals, 2);
        assert_eq!(
            first.approved_by,
            vec![APPROVER_1.to_owned(), APPROVER_2.to_owned()]
        );
    }
}

#[test]
fn integration_outbound_quorum_rejections_are_explicit_and_fail_closed() {
    let discord_engine = DiscordBridgeEngine::new(discord_config()).expect("engine should build");
    assert_eq!(
        discord_engine.process_outbound_with_approvals(
            &outbound_request("discord-under-quorum-1", DISCORD_ROUTE_CHANNEL),
            vec![APPROVER_1.to_owned()],
        ),
        Err(DiscordBridgeError::InsufficientApprovals {
            required: 2,
            provided: 1,
        })
    );
    assert_eq!(
        discord_engine.process_outbound_with_approvals(
            &outbound_request("discord-unauthorized-approver-1", DISCORD_ROUTE_CHANNEL),
            vec![
                APPROVER_1.to_owned(),
                "kamn:did:agent:approver-x".to_owned()
            ],
        ),
        Err(DiscordBridgeError::UnauthorizedApprover(
            "kamn:did:agent:approver-x".to_owned()
        ))
    );
    assert_eq!(
        discord_engine.process_outbound_with_approvals(
            &outbound_request("discord-duplicate-approver-1", DISCORD_ROUTE_CHANNEL),
            vec![APPROVER_1.to_owned(), APPROVER_1.to_owned()],
        ),
        Err(DiscordBridgeError::DuplicateApproval(APPROVER_1.to_owned()))
    );

    let cross_chain_engine =
        CrossChainBridgeEngine::new(cross_chain_config()).expect("engine should build");
    assert_eq!(
        cross_chain_engine.process_outbound_with_approvals(
            CrossChainNetwork::Solana,
            &outbound_request("solana-under-quorum-1", SOLANA_ROUTE_CHANNEL),
            vec![APPROVER_1.to_owned()],
        ),
        Err(CrossChainBridgeError::InsufficientApprovals {
            required: 2,
            provided: 1,
        })
    );
    assert_eq!(
        cross_chain_engine.process_outbound_with_approvals(
            CrossChainNetwork::Solana,
            &outbound_request("solana-unauthorized-approver-1", SOLANA_ROUTE_CHANNEL),
            vec![
                APPROVER_1.to_owned(),
                "kamn:did:agent:approver-x".to_owned()
            ],
        ),
        Err(CrossChainBridgeError::UnauthorizedApprover(
            "kamn:did:agent:approver-x".to_owned()
        ))
    );
}

#[test]
fn regression_rejects_replayed_outbound_quorum_dispatch_requests() {
    // Regression: #851
    let discord_engine = DiscordBridgeEngine::new(discord_config()).expect("engine should build");
    let discord_request = outbound_request("discord-outbound-replay-1", DISCORD_ROUTE_CHANNEL);
    let discord_approvers = vec![APPROVER_1.to_owned(), APPROVER_2.to_owned()];
    discord_engine
        .process_outbound_with_approvals(&discord_request, discord_approvers.clone())
        .expect("first discord dispatch should succeed");
    assert_eq!(
        discord_engine.process_outbound_with_approvals(&discord_request, discord_approvers),
        Err(DiscordBridgeError::Bridge(
            "duplicate outbound request id: discord-outbound-replay-1".to_owned()
        ))
    );

    let cross_chain_engine =
        CrossChainBridgeEngine::new(cross_chain_config()).expect("engine should build");
    let cross_chain_request = outbound_request("solana-outbound-replay-1", SOLANA_ROUTE_CHANNEL);
    let cross_chain_approvers = vec![APPROVER_1.to_owned(), APPROVER_2.to_owned()];
    cross_chain_engine
        .process_outbound_with_approvals(
            CrossChainNetwork::Solana,
            &cross_chain_request,
            cross_chain_approvers.clone(),
        )
        .expect("first cross-chain dispatch should succeed");
    assert_eq!(
        cross_chain_engine.process_outbound_with_approvals(
            CrossChainNetwork::Solana,
            &cross_chain_request,
            cross_chain_approvers,
        ),
        Err(CrossChainBridgeError::Bridge(
            "duplicate outbound request id: solana-outbound-replay-1".to_owned()
        ))
    );
}
