use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime wiring.
pub struct RuntimeWiring {
    /// Common components.
    pub common_components: Vec<&'static str>,
    /// Role components.
    pub role_components: Vec<&'static str>,
}

impl RuntimeWiring {
    /// Handles all components.
    pub fn all_components(&self) -> Vec<&'static str> {
        let mut components = self.common_components.clone();
        components.extend(self.role_components.iter().copied());
        components
    }
}

/// Runtime transport profile used to select deterministic p2p wiring markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTransportProfile {
    /// In-memory deterministic transport adapter path.
    InMemoryDeterministic,
    /// Live libp2p transport adapter path.
    Libp2pLive,
}

/// Cargo feature flag that enables native libp2p dependency wiring.
pub const LIBP2P_LIVE_TRANSPORT_FEATURE_NAME: &str = "libp2p-live-transport";

/// Compile-time libp2p provider mode selected by cargo features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Libp2pCompileMode {
    /// Deterministic contract-only mode without native libp2p runtime dependency usage.
    ContractOnly,
    /// Native libp2p provider mode backed by optional dependency wiring.
    NativeLibp2p,
}

impl Libp2pCompileMode {
    /// Returns deterministic runtime wiring marker for this compile mode.
    pub fn marker_component(self) -> &'static str {
        match self {
            Self::ContractOnly => "p2p-live-libp2p-provider:contract-only",
            Self::NativeLibp2p => "p2p-live-libp2p-provider:native",
        }
    }
}

/// Returns the cargo feature gate name used for native libp2p transport wiring.
pub fn libp2p_feature_gate_name() -> &'static str {
    LIBP2P_LIVE_TRANSPORT_FEATURE_NAME
}

/// Resolves compile-time libp2p provider mode from cargo feature flags.
pub fn resolve_libp2p_compile_mode() -> Libp2pCompileMode {
    #[cfg(feature = "libp2p-live-transport")]
    {
        Libp2pCompileMode::NativeLibp2p
    }
    #[cfg(not(feature = "libp2p-live-transport"))]
    {
        Libp2pCompileMode::ContractOnly
    }
}

impl RuntimeTransportProfile {
    /// Returns deterministic profile marker used in runtime wiring.
    pub fn marker_component(self) -> &'static str {
        match self {
            Self::InMemoryDeterministic => "p2p-transport-profile:in-memory-deterministic",
            Self::Libp2pLive => "p2p-transport-profile:libp2p-live",
        }
    }

    fn provider_component(self) -> &'static str {
        match self {
            Self::InMemoryDeterministic => "p2p-in-memory-transport-fallback",
            Self::Libp2pLive => "p2p-live-libp2p-provider",
        }
    }
}

/// Builds runtime wiring with explicit deterministic p2p transport profile markers.
pub fn build_runtime_wiring_with_transport_profile(
    config: &NodeConfig,
    transport_profile: RuntimeTransportProfile,
) -> RuntimeWiring {
    RuntimeWiring {
        common_components: build_common_components(config, transport_profile),
        role_components: build_role_components(&config.role),
    }
}

/// Handles build runtime wiring.
pub fn build_runtime_wiring(config: &NodeConfig) -> RuntimeWiring {
    build_runtime_wiring_with_transport_profile(
        config,
        RuntimeTransportProfile::InMemoryDeterministic,
    )
}

fn build_common_components(
    config: &NodeConfig,
    transport_profile: RuntimeTransportProfile,
) -> Vec<&'static str> {
    let mut common_components = vec!["state-store", "message-router", "audit-log", "api-surface"];
    if config.enable_gossip {
        common_components.extend([
            "p2p-discovery",
            "p2p-gossip-transport",
            "p2p-libp2p-swarm-stack",
            "p2p-libp2p-harness-ready",
            transport_profile.marker_component(),
            transport_profile.provider_component(),
        ]);
        if transport_profile == RuntimeTransportProfile::Libp2pLive {
            common_components.push(resolve_libp2p_compile_mode().marker_component());
        }
    } else {
        common_components.push("gossip-transport-disabled");
    }
    common_components
}

fn build_role_components(role: &NodeRole) -> Vec<&'static str> {
    match role {
        NodeRole::Processor => vec![
            "mempool",
            "executor",
            "block-producer",
            "consensus-validator",
        ],
        NodeRole::Listener => vec!["external-listener", "event-normalizer"],
        NodeRole::Approver => vec!["quorum-approver", "outbound-authorizer"],
    }
}
