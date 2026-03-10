#![allow(missing_docs)]

#[cfg(feature = "libp2p-live-transport")]
use super::deterministic_config::P2pSwarmDeterministicConfig;
use super::*;

#[cfg(feature = "libp2p-live-transport")]
use libp2p::{gossipsub, identify, noise, swarm::Swarm, tcp, yamux, Multiaddr, SwarmBuilder};

pub fn canonical_libp2p_identify_protocol_id() -> &'static str {
    super::super::LIBP2P_IDENTIFY_PROTOCOL_ID
}

pub fn canonical_libp2p_topic_id(topic: &str) -> Result<String, P2pTransportError> {
    validate_topic(topic)?;
    Ok(format!(
        "{}{}",
        super::super::LIBP2P_TOPIC_NAMESPACE,
        topic.trim()
    ))
}

#[cfg(feature = "libp2p-live-transport")]
pub(crate) fn validate_libp2p_native_runtime_config(
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    config
        .listen_address()
        .parse::<Multiaddr>()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    for bootstrap_peer in config.bootstrap_peers() {
        bootstrap_peer
            .parse::<Multiaddr>()
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    }
    for topic in config.gossip_topics() {
        let topic_id = canonical_libp2p_topic_id(topic.as_str())
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
        let _ = gossipsub::IdentTopic::new(topic_id);
    }
    Ok(())
}

#[cfg(feature = "libp2p-live-transport")]
pub(crate) fn build_libp2p_runtime_swarm(
    config: &P2pSwarmDeterministicConfig,
) -> Result<Swarm<Libp2pDeterministicRuntimeBehaviour>, P2pTransportError> {
    let swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?
        .with_behaviour(|key| {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .validation_mode(gossipsub::ValidationMode::Permissive)
                .build()
                .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            let mut gossipsub_behavior = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            for topic in config.gossip_topics() {
                let topic_id = canonical_libp2p_topic_id(topic.as_str())
                    .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
                gossipsub_behavior
                    .subscribe(&gossipsub::IdentTopic::new(topic_id))
                    .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            }
            Ok(Libp2pDeterministicRuntimeBehaviour {
                gossipsub: gossipsub_behavior,
                identify: identify::Behaviour::new(identify::Config::new(
                    canonical_libp2p_identify_protocol_id().to_owned(),
                    key.public(),
                )),
            })
        })
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?
        .build();
    Ok(swarm)
}

#[cfg(feature = "libp2p-live-transport")]
pub(crate) fn apply_libp2p_runtime_network_config(
    swarm: &mut Swarm<Libp2pDeterministicRuntimeBehaviour>,
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    let listen_multiaddr = config
        .listen_address()
        .parse::<Multiaddr>()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    Swarm::listen_on(swarm, listen_multiaddr)
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    for bootstrap_peer in config.bootstrap_peers() {
        let bootstrap_multiaddr = bootstrap_peer
            .parse::<Multiaddr>()
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
        let _ = Swarm::dial(swarm, bootstrap_multiaddr);
    }
    Ok(())
}

#[cfg(feature = "libp2p-live-transport")]
pub(crate) fn validate_libp2p_runtime_stack_composition(
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    let config = config.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    runtime.block_on(async move {
        let mut swarm = build_libp2p_runtime_swarm(&config)?;
        apply_libp2p_runtime_network_config(&mut swarm, &config)?;
        Ok(())
    })
}

#[cfg(feature = "libp2p-live-transport")]
pub(crate) fn runtime_channel_closed_behavior_failure_class(
    operation: Libp2pRuntimeAdapterOperation,
) -> Libp2pBehaviorFailureClass {
    match operation {
        Libp2pRuntimeAdapterOperation::Connect => {
            Libp2pBehaviorFailureClass::RuntimeConnectChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Discover => {
            Libp2pBehaviorFailureClass::RuntimeDiscoverChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Publish => {
            Libp2pBehaviorFailureClass::RuntimePublishChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Receive => {
            Libp2pBehaviorFailureClass::RuntimeReceiveChannelClosed
        }
        Libp2pRuntimeAdapterOperation::EventDrain => {
            Libp2pBehaviorFailureClass::RuntimeEventDrainChannelClosed
        }
    }
}
