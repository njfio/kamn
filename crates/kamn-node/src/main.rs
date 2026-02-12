use std::env;
use std::process::ExitCode;

use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};
use kamn_core::{
    bootstrap, BootstrapPlan, ConfigError, DeterministicProposalPlanner, KolmeApiBroadcastRequest,
    KolmeApiNextNonceRequest, KolmeCommitReceiptFinality, KolmeRuntimeCommitFinalityChecker,
    KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitProvider,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderOutcome,
    KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitRequest, NodeConfig, NodeRole,
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate, RecoveryRejoinGuard,
    RecoveryStatus, RejoinAttempt, SyncMode,
};

const KOLME_LIVE_PROVIDER_CONTRACT: &str = "KolmeRuntimeCommitLiveProvider";
const KOLME_LIVE_SIGNING_PROFILE: &str = "kolme-fork-secp256k1-v1";
const KOLME_IN_MEMORY_PROVIDER_MARKER: &str = "InMemoryKolmeRuntimeCommitClient";
const KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS: u64 = 30;
const KOLME_LIVE_FINALITY_STATUS_PATH: &str = "/runtime-commit/status";
const KOLME_LIVE_FINALITY_MAX_ATTEMPTS: u32 = 2;
const KOLME_LIVE_NONCE_PATH: &str = "/get-next-nonce";
const KOLME_LIVE_SIGNER_PROFILE_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_PROFILE";
const KOLME_LIVE_SIGNER_PROFILE_PRIMARY: &str = "ops-primary";
const KOLME_LIVE_SIGNER_PROFILE_SECONDARY: &str = "ops-secondary";
const KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL: &str = "env-local";
const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX";
const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY";
const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK";
const KOLME_LIVE_NATIVE_CREATED_AT: &str = "2026-02-12T00:00:00Z";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutputMode {
    kind: OutputModeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputModeKind {
    Text,
    Json,
}

impl OutputMode {
    fn text() -> Self {
        Self {
            kind: OutputModeKind::Text,
        }
    }

    fn json() -> Self {
        Self {
            kind: OutputModeKind::Json,
        }
    }

    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "text" => Ok(Self::text()),
            "json" => Ok(Self::json()),
            other => Err(ConfigError::InvalidOutputMode(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeMode {
    kind: RuntimeModeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeModeKind {
    Bootstrap,
    Planning,
    RecoveryCheck,
    Daemon,
    KolmeLive,
}

impl RuntimeMode {
    fn bootstrap() -> Self {
        Self {
            kind: RuntimeModeKind::Bootstrap,
        }
    }

    fn planning() -> Self {
        Self {
            kind: RuntimeModeKind::Planning,
        }
    }

    fn recovery_check() -> Self {
        Self {
            kind: RuntimeModeKind::RecoveryCheck,
        }
    }

    fn daemon() -> Self {
        Self {
            kind: RuntimeModeKind::Daemon,
        }
    }

    fn kolme_live() -> Self {
        Self {
            kind: RuntimeModeKind::KolmeLive,
        }
    }

    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "bootstrap" => Ok(Self::bootstrap()),
            "planning" => Ok(Self::planning()),
            "recovery-check" => Ok(Self::recovery_check()),
            "daemon" => Ok(Self::daemon()),
            "kolme-live" => Ok(Self::kolme_live()),
            other => Err(ConfigError::InvalidRuntimeMode(other.to_owned())),
        }
    }

    fn as_str(self) -> &'static str {
        match self.kind {
            RuntimeModeKind::Bootstrap => "bootstrap",
            RuntimeModeKind::Planning => "planning",
            RuntimeModeKind::RecoveryCheck => "recovery-check",
            RuntimeModeKind::Daemon => "daemon",
            RuntimeModeKind::KolmeLive => "kolme-live",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticsMode {
    Basic,
    Snapshot,
}

impl DiagnosticsMode {
    fn basic() -> Self {
        Self::Basic
    }

    fn snapshot() -> Self {
        Self::Snapshot
    }

    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "basic" => Ok(Self::basic()),
            "snapshot" => Ok(Self::snapshot()),
            other => Err(ConfigError::InvalidDiagnosticsMode(other.to_owned())),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Snapshot => "snapshot",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalProfile {
    Processor,
    Listener,
    Approver,
}

impl LocalProfile {
    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "local-processor" => Ok(Self::Processor),
            "local-listener" => Ok(Self::Listener),
            "local-approver" => Ok(Self::Approver),
            other => Err(ConfigError::InvalidNodeProfile(other.to_owned())),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Processor => "local-processor",
            Self::Listener => "local-listener",
            Self::Approver => "local-approver",
        }
    }

    fn default_role(self) -> NodeRole {
        match self {
            Self::Processor => NodeRole::Processor,
            Self::Listener => NodeRole::Listener,
            Self::Approver => NodeRole::Approver,
        }
    }

    fn default_storage_dir(self) -> &'static str {
        match self {
            Self::Processor => "./data/processor",
            Self::Listener => "./data/listener",
            Self::Approver => "./data/approver",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeCli {
    profile: Option<LocalProfile>,
    role: NodeRole,
    chain_id: String,
    chain_version: String,
    storage_dir: String,
    enable_gossip: bool,
    sync_mode: SyncMode,
    runtime_mode: RuntimeMode,
    expected_state_version: Option<u64>,
    expected_state_hash: Option<String>,
    proposals: Vec<ProposalCandidate>,
    rejoin_attempts: Vec<RejoinAttempt>,
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
    daemon_peer_id: Option<String>,
    daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
    kolme_live_base_url: Option<String>,
    kolme_live_provider_hint: Option<String>,
    kolme_live_signing_profile: Option<String>,
    kolme_live_strict_signer_contracts: bool,
    kolme_live_signer_profile: Option<String>,
    kolme_live_signer_key_source: Option<String>,
    output_mode: OutputMode,
    diagnostics_mode: DiagnosticsMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanningExecution {
    expected_state_hash: String,
    candidate_count: usize,
    scheduled_candidate_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecoveryExecution {
    expected_state_version: u64,
    expected_state_hash: String,
    attempt_count: usize,
    decisions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DaemonExecution {
    max_ticks: u64,
    tick_interval_ms: u64,
    executed_ticks: u64,
    completion_reason: String,
    peer_id: Option<String>,
    peer_lifecycle_final_state: Option<String>,
    peer_lifecycle_applied_events: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KolmeLiveExecution {
    provider_client_contract: String,
    base_url: String,
    provider_hint: String,
    signing_profile: String,
    signer_profile_selector_env: String,
    signer_profile: String,
    signer_key_source: String,
    signer_private_key_env: String,
    execution_status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KolmeLiveSignerSelection {
    profile: &'static str,
    key_source: &'static str,
    private_key_env: &'static str,
}

#[derive(Debug, Clone)]
struct KolmeForkSecp256k1SignerAdapter {
    signing_key: SigningKey,
    private_key_env: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct RuntimeExecutionBundle {
    planning: Option<PlanningExecution>,
    recovery: Option<RecoveryExecution>,
    daemon: Option<DaemonExecution>,
    kolme_live: Option<KolmeLiveExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeBootstrapReport {
    runtime_mode: String,
    diagnostics_mode: String,
    component_count: usize,
    planning_expected_state_hash: Option<String>,
    planning_candidate_count: Option<usize>,
    planning_scheduled_candidate_ids: Option<Vec<String>>,
    recovery_expected_state_version: Option<u64>,
    recovery_expected_state_hash: Option<String>,
    recovery_attempt_count: Option<usize>,
    recovery_decisions: Option<Vec<String>>,
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
    daemon_executed_ticks: Option<u64>,
    daemon_completion_reason: Option<String>,
    daemon_peer_id: Option<String>,
    daemon_peer_lifecycle_final_state: Option<String>,
    daemon_peer_lifecycle_applied_events: Option<Vec<String>>,
    kolme_live_provider_client_contract: Option<String>,
    kolme_live_base_url: Option<String>,
    kolme_live_provider_hint: Option<String>,
    kolme_live_signing_profile: Option<String>,
    kolme_live_signer_profile_selector_env: Option<String>,
    kolme_live_signer_profile: Option<String>,
    kolme_live_signer_key_source: Option<String>,
    kolme_live_signer_private_key_env: Option<String>,
    kolme_live_execution_status: Option<String>,
    profile: Option<String>,
    role: String,
    chain_id: String,
    chain_version: String,
    storage_dir: String,
    gossip_enabled: bool,
    sync_mode: String,
    sync_startup: String,
    sync_recovery: String,
    state_version: u32,
    pending_migrations: usize,
    components: Vec<String>,
}

fn parse_args<I>(args: I) -> Result<NodeCli, ConfigError>
where
    I: IntoIterator<Item = String>,
{
    let mut role: Option<NodeRole> = None;
    let mut profile: Option<LocalProfile> = None;
    let mut chain_id = String::from("kamn-devnet");
    let mut chain_version = String::from("v0.1.0");
    let mut storage_dir = String::from("./data");
    let mut enable_gossip = true;
    let mut sync_mode = SyncMode::Fast;
    let mut runtime_mode = RuntimeMode::bootstrap();
    let mut expected_state_version: Option<u64> = None;
    let mut expected_state_hash: Option<String> = None;
    let mut proposals: Vec<ProposalCandidate> = Vec::new();
    let mut rejoin_attempts: Vec<RejoinAttempt> = Vec::new();
    let mut daemon_max_ticks: Option<u64> = None;
    let mut daemon_tick_interval_ms: Option<u64> = None;
    let mut daemon_peer_id: Option<String> = None;
    let mut daemon_lifecycle_events: Vec<PeerLifecycleEvent> = Vec::new();
    let mut kolme_live_base_url: Option<String> = None;
    let mut kolme_live_provider_hint: Option<String> = None;
    let mut kolme_live_signing_profile: Option<String> = None;
    let mut kolme_live_strict_signer_contracts = false;
    let mut kolme_live_signer_profile: Option<String> = None;
    let mut kolme_live_signer_key_source: Option<String> = None;
    let mut output_mode = OutputMode::text();
    let mut diagnostics_mode = DiagnosticsMode::basic();
    let mut role_overridden = false;
    let mut chain_id_overridden = false;
    let mut chain_version_overridden = false;
    let mut storage_dir_overridden = false;
    let mut gossip_overridden = false;
    let mut sync_mode_overridden = false;

    let mut iter = args.into_iter();
    let _bin = iter.next();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--role" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--role"))?;
                role = Some(value.parse::<NodeRole>()?);
                role_overridden = true;
            }
            "--profile" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--profile"))?;
                profile = Some(LocalProfile::parse(&value)?);
            }
            "--chain-id" => {
                chain_id = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--chain-id"))?;
                chain_id_overridden = true;
            }
            "--chain-version" => {
                chain_version = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--chain-version"))?;
                chain_version_overridden = true;
            }
            "--storage-dir" => {
                storage_dir = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--storage-dir"))?;
                storage_dir_overridden = true;
            }
            "--disable-gossip" => {
                enable_gossip = false;
                gossip_overridden = true;
            }
            "--sync-mode" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--sync-mode"))?;
                sync_mode = value.parse::<SyncMode>()?;
                sync_mode_overridden = true;
            }
            "--runtime-mode" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--runtime-mode"))?;
                runtime_mode = RuntimeMode::parse(&value)?;
            }
            "--expected-state-version" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--expected-state-version",
                ))?;
                expected_state_version = Some(parse_state_version_arg(&value)?);
            }
            "--expected-state-hash" => {
                expected_state_hash = Some(
                    iter.next()
                        .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?,
                );
            }
            "--proposal" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--proposal"))?;
                proposals.push(parse_proposal_candidate(&value)?);
            }
            "--rejoin-attempt" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--rejoin-attempt"))?;
                rejoin_attempts.push(parse_rejoin_attempt(&value)?);
            }
            "--daemon-max-ticks" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
                daemon_max_ticks = Some(parse_daemon_control_arg(&value)?);
            }
            "--daemon-tick-interval-ms" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--daemon-tick-interval-ms",
                ))?;
                daemon_tick_interval_ms = Some(parse_daemon_control_arg(&value)?);
            }
            "--daemon-peer-id" => {
                daemon_peer_id = Some(
                    iter.next()
                        .ok_or(ConfigError::MissingArgumentValue("--daemon-peer-id"))?,
                );
            }
            "--daemon-lifecycle-event" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--daemon-lifecycle-event",
                ))?;
                daemon_lifecycle_events.push(parse_daemon_lifecycle_event(&value)?);
            }
            "--kolme-live-base-url" => {
                kolme_live_base_url = Some(
                    iter.next()
                        .ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?,
                );
            }
            "--kolme-live-provider-hint" => {
                kolme_live_provider_hint = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-provider-hint"),
                )?);
            }
            "--kolme-live-signing-profile" => {
                kolme_live_signing_profile = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-signing-profile"),
                )?);
            }
            "--kolme-live-strict-signer-contracts" => {
                kolme_live_strict_signer_contracts = true;
            }
            "--kolme-live-signer-profile" => {
                kolme_live_signer_profile = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-signer-profile"),
                )?);
            }
            "--kolme-live-signer-key-source" => {
                kolme_live_signer_key_source = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-signer-key-source"),
                )?);
            }
            "--output" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--output"))?;
                output_mode = OutputMode::parse(&value)?;
            }
            "--diagnostics" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--diagnostics"))?;
                diagnostics_mode = DiagnosticsMode::parse(&value)?;
            }
            unknown => {
                return Err(ConfigError::UnknownArgument(unknown.to_owned()));
            }
        }
    }

    if let Some(selected_profile) = profile {
        if !role_overridden {
            role = Some(selected_profile.default_role());
        }
        if !chain_id_overridden {
            chain_id = "kamn-localnet".to_owned();
        }
        if !chain_version_overridden {
            chain_version = "v0.1.0".to_owned();
        }
        if !storage_dir_overridden {
            storage_dir = selected_profile.default_storage_dir().to_owned();
        }
        if !gossip_overridden {
            enable_gossip = true;
        }
        if !sync_mode_overridden {
            sync_mode = SyncMode::Fast;
        }
    }

    let role = role.ok_or(ConfigError::MissingArgumentValue("--role"))?;

    if runtime_mode.kind == RuntimeModeKind::Planning {
        if expected_state_hash.is_none() {
            return Err(ConfigError::MissingArgumentValue("--expected-state-hash"));
        }
        if proposals.is_empty() {
            return Err(ConfigError::MissingArgumentValue("--proposal"));
        }
    }
    if runtime_mode.kind == RuntimeModeKind::RecoveryCheck {
        if expected_state_version.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--expected-state-version",
            ));
        }
        if expected_state_hash.is_none() {
            return Err(ConfigError::MissingArgumentValue("--expected-state-hash"));
        }
        if rejoin_attempts.is_empty() {
            return Err(ConfigError::MissingArgumentValue("--rejoin-attempt"));
        }
    }
    if runtime_mode.kind == RuntimeModeKind::Daemon {
        if daemon_max_ticks.is_none() {
            return Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"));
        }
        if daemon_tick_interval_ms.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--daemon-tick-interval-ms",
            ));
        }
        if !daemon_lifecycle_events.is_empty() && daemon_peer_id.is_none() {
            return Err(ConfigError::MissingArgumentValue("--daemon-peer-id"));
        }
    }
    if runtime_mode.kind == RuntimeModeKind::KolmeLive {
        if kolme_live_base_url.is_none() {
            return Err(ConfigError::MissingArgumentValue("--kolme-live-base-url"));
        }
        if kolme_live_provider_hint.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--kolme-live-provider-hint",
            ));
        }
        if kolme_live_signing_profile.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--kolme-live-signing-profile",
            ));
        }
        let provider_hint = kolme_live_provider_hint
            .as_ref()
            .expect("provider hint is required for kolme-live mode");
        if provider_hint.contains(KOLME_IN_MEMORY_PROVIDER_MARKER) {
            return Err(ConfigError::InvalidKolmeLiveProviderHint(
                provider_hint.to_owned(),
            ));
        }
        let signing_profile = kolme_live_signing_profile
            .as_ref()
            .expect("signing profile is required for kolme-live mode");
        if signing_profile != KOLME_LIVE_SIGNING_PROFILE {
            return Err(ConfigError::InvalidKolmeLiveSigningProfile(
                signing_profile.to_owned(),
            ));
        }
        if kolme_live_strict_signer_contracts {
            let signer_profile =
                kolme_live_signer_profile
                    .as_deref()
                    .ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-profile",
                    ))?;
            normalize_kolme_live_signer_profile_selector(signer_profile)?;
            let key_source = kolme_live_signer_key_source.as_deref().ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-signer-key-source"),
            )?;
            normalize_kolme_live_signer_key_source(key_source)?;
        }
    }

    Ok(NodeCli {
        profile,
        role,
        chain_id,
        chain_version,
        storage_dir,
        enable_gossip,
        sync_mode,
        runtime_mode,
        expected_state_version,
        expected_state_hash,
        proposals,
        rejoin_attempts,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_peer_id,
        daemon_lifecycle_events,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_strict_signer_contracts,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        output_mode,
        diagnostics_mode,
    })
}

fn normalize_kolme_live_signer_profile_selector(value: &str) -> Result<&'static str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-profile must not be empty".to_owned(),
        ));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(KOLME_LIVE_SIGNER_PROFILE_PRIMARY),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(KOLME_LIVE_SIGNER_PROFILE_SECONDARY),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-profile has unsupported profile: {trimmed}"
        ))),
    }
}

fn normalize_kolme_live_signer_key_source(value: &str) -> Result<&'static str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-key-source must not be empty".to_owned(),
        ));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL => Ok(KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-key-source has unsupported key source: {trimmed}"
        ))),
    }
}

fn parse_state_version_arg(value: &str) -> Result<u64, ConfigError> {
    let state_version = value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidExpectedStateVersion(value.to_owned()))?;
    if state_version == 0 {
        return Err(ConfigError::InvalidExpectedStateVersion(value.to_owned()));
    }
    Ok(state_version)
}

fn parse_proposal_candidate(value: &str) -> Result<ProposalCandidate, ConfigError> {
    let parts = value.split('|').collect::<Vec<&str>>();
    if parts.len() != 4 {
        return Err(ConfigError::InvalidProposalArgument(value.to_owned()));
    }
    let nonce = parts[2]
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidProposalArgument(value.to_owned()))?;
    ProposalCandidate::new(parts[0], parts[1], nonce, parts[3])
        .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))
}

fn parse_rejoin_attempt(value: &str) -> Result<RejoinAttempt, ConfigError> {
    let parts = value.split('|').collect::<Vec<&str>>();
    if parts.len() != 4 {
        return Err(ConfigError::InvalidRejoinAttemptArgument(value.to_owned()));
    }
    let state_version = parts[1]
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidRejoinAttemptArgument(value.to_owned()))?;
    RejoinAttempt::new(parts[0], state_version, parts[2], parts[3])
        .map_err(|_| ConfigError::InvalidRejoinAttemptArgument(value.to_owned()))
}

fn parse_daemon_control_arg(value: &str) -> Result<u64, ConfigError> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidDaemonControlArgument(value.to_owned()))?;
    if parsed == 0 {
        return Err(ConfigError::InvalidDaemonControlArgument(value.to_owned()));
    }
    Ok(parsed)
}

fn parse_daemon_lifecycle_event(value: &str) -> Result<PeerLifecycleEvent, ConfigError> {
    match value {
        "start-connect" => Ok(PeerLifecycleEvent::StartConnect),
        "handshake-succeeded" => Ok(PeerLifecycleEvent::HandshakeSucceeded),
        "heartbeat-missed" => Ok(PeerLifecycleEvent::HeartbeatMissed),
        "heartbeat-restored" => Ok(PeerLifecycleEvent::HeartbeatRestored),
        "disconnect" => Ok(PeerLifecycleEvent::Disconnect),
        "rejoin" => Ok(PeerLifecycleEvent::Rejoin),
        _ => Err(ConfigError::InvalidDaemonLifecycleEvent(value.to_owned())),
    }
}

fn daemon_lifecycle_event_as_str(event: PeerLifecycleEvent) -> &'static str {
    match event {
        PeerLifecycleEvent::StartConnect => "start-connect",
        PeerLifecycleEvent::HandshakeSucceeded => "handshake-succeeded",
        PeerLifecycleEvent::HeartbeatMissed => "heartbeat-missed",
        PeerLifecycleEvent::HeartbeatRestored => "heartbeat-restored",
        PeerLifecycleEvent::Disconnect => "disconnect",
        PeerLifecycleEvent::Rejoin => "rejoin",
    }
}

fn peer_lifecycle_state_as_str(state: PeerLifecycleState) -> &'static str {
    match state {
        PeerLifecycleState::Disconnected => "disconnected",
        PeerLifecycleState::Connecting => "connecting",
        PeerLifecycleState::Active => "active",
        PeerLifecycleState::Degraded => "degraded",
    }
}

fn run() -> Result<(), ConfigError> {
    let cli = parse_args(env::args())?;
    let output_mode = cli.output_mode;
    let report = execute(cli)?;
    println!("{}", render_bootstrap_report(&report, output_mode));

    Ok(())
}

fn build_kolme_live_request(
    plan: &BootstrapPlan,
) -> Result<KolmeRuntimeCommitRequest, ConfigError> {
    let role_label = plan.config.role.as_str();
    let operation_id = format!("runtime-commit:{}:{role_label}", plan.config.chain_id);
    let state_root = format!(
        "state:{}:{}",
        plan.config.chain_version, plan.state_schema.version.0
    );
    let actor_did = format!("kamn:did:agent:node-runtime-{role_label}");
    let payload_hash = format!("payload:{}:{role_label}", plan.config.chain_version);
    KolmeRuntimeCommitRequest::deterministic(
        operation_id.as_str(),
        state_root.as_str(),
        actor_did.as_str(),
        1,
        payload_hash.as_str(),
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))
}

fn decode_kolme_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

fn decode_kolme_hex_bytes(value: &str, env_name: &str) -> Result<Vec<u8>, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must not be empty"
        )));
    }
    if !trimmed.len().is_multiple_of(2) {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must have an even number of hex characters"
        )));
    }
    let mut decoded = Vec::with_capacity(trimmed.len() / 2);
    for pair in trimmed.as_bytes().chunks_exact(2) {
        let high = decode_kolme_hex_nibble(pair[0]).ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "{env_name} contains invalid hex character '{}'",
                pair[0] as char
            ))
        })?;
        let low = decode_kolme_hex_nibble(pair[1]).ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "{env_name} contains invalid hex character '{}'",
                pair[1] as char
            ))
        })?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn encode_kolme_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn resolve_kolme_live_signer_profile_selector_from_env() -> Result<Option<&'static str>, ConfigError>
{
    let profile_value = match env::var(KOLME_LIVE_SIGNER_PROFILE_ENV) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PROFILE_ENV} must be valid utf-8"
            )))
        }
    };
    let trimmed = profile_value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PROFILE_ENV} must not be empty"
        )));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(Some(KOLME_LIVE_SIGNER_PROFILE_PRIMARY)),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(Some(KOLME_LIVE_SIGNER_PROFILE_SECONDARY)),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PROFILE_ENV} has unsupported profile: {trimmed}"
        ))),
    }
}

fn resolve_kolme_live_signer_private_key_env_name(
    strict_signer_profile: Option<&str>,
) -> Result<(&'static str, &'static str), ConfigError> {
    let profile_from_env = resolve_kolme_live_signer_profile_selector_from_env()?;
    let profile_value = if let Some(profile) = strict_signer_profile {
        let strict_profile = normalize_kolme_live_signer_profile_selector(profile)?;
        if let Some(env_profile) = profile_from_env {
            if env_profile != strict_profile {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "strict signer profile mismatch: --kolme-live-signer-profile={strict_profile} conflicts with {KOLME_LIVE_SIGNER_PROFILE_ENV}={env_profile}"
                )));
            }
        }
        strict_profile
    } else {
        profile_from_env.unwrap_or(KOLME_LIVE_SIGNER_PROFILE_PRIMARY)
    };
    match profile_value {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok((
            KOLME_LIVE_SIGNER_PROFILE_PRIMARY,
            KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV,
        )),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok((
            KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
            KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV,
        )),
        _ => unreachable!("profile value is normalized before key env mapping"),
    }
}

fn resolve_kolme_live_signer_selection(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<KolmeLiveSignerSelection, ConfigError> {
    let key_source = if let Some(key_source) = strict_signer_key_source {
        normalize_kolme_live_signer_key_source(key_source)?
    } else if strict_signer_profile.is_some() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-key-source must be declared for strict signer contracts"
                .to_owned(),
        ));
    } else {
        KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL
    };
    let (profile, private_key_env) =
        resolve_kolme_live_signer_private_key_env_name(strict_signer_profile)?;
    Ok(KolmeLiveSignerSelection {
        profile,
        key_source,
        private_key_env,
    })
}

trait KolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError>;

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError>;
}

struct EnvKolmeLiveSignerSecretProvider;

impl KolmeLiveSignerSecretProvider for EnvKolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError> {
        match env::var(KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV) {
            Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must remain unset (fallback_signer_secret_present_violation)"
            ))),
            Err(env::VarError::NotPresent) => Ok(()),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must be valid utf-8 when present (fallback_signer_secret_present_violation)"
            ))),
        }
    }

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError> {
        match env::var(selection.private_key_env) {
            Ok(private_key_hex) => Ok(private_key_hex),
            Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be set for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be valid utf-8 for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
        }
    }
}

fn read_kolme_live_signer_private_key_hex_with_provider<P: KolmeLiveSignerSecretProvider>(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
    provider: &P,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    provider.ensure_no_fallback_private_key_path()?;
    let private_key_hex = provider.read_private_key_hex(&selection)?;
    Ok((private_key_hex, selection))
}

fn read_kolme_live_signer_private_key_hex(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let provider = EnvKolmeLiveSignerSecretProvider;
    read_kolme_live_signer_private_key_hex_with_provider(
        strict_signer_profile,
        strict_signer_key_source,
        &provider,
    )
}

fn escape_kolme_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

impl KolmeForkSecp256k1SignerAdapter {
    fn from_private_key_hex(
        private_key_hex: &str,
        private_key_env: &'static str,
    ) -> Result<Self, ConfigError> {
        let private_key_bytes = decode_kolme_hex_bytes(private_key_hex, private_key_env)?;
        let signing_key =
            SigningKey::from_slice(private_key_bytes.as_slice()).map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "{private_key_env} is not a valid secp256k1 private key: {error}",
                ))
            })?;
        Ok(Self {
            signing_key,
            private_key_env,
        })
    }

    fn public_key_compressed_hex(&self) -> String {
        encode_kolme_hex_lower(
            self.signing_key
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        )
    }

    fn verify_message(
        &self,
        message: &str,
        signature_hex: &str,
        recovery_id: u8,
    ) -> Result<(), ConfigError> {
        let signature_bytes =
            decode_kolme_hex_bytes(signature_hex, "runtime_commit_signature_hex")?;
        if signature_bytes.len() != 64 {
            return Err(ConfigError::RuntimeKolmeLive(
                "runtime commit signature hex must decode to exactly 64 bytes".to_owned(),
            ));
        }
        let signature = Signature::from_slice(signature_bytes.as_slice()).map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "runtime commit signature bytes are invalid secp256k1 material: {error}",
            ))
        })?;
        let recovery = RecoveryId::from_byte(recovery_id).ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "runtime commit recovery id must be within secp256k1 range [0,3], found {recovery_id}",
            ))
        })?;
        let recovered = VerifyingKey::recover_from_msg(message.as_bytes(), &signature, recovery)
            .map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "failed to recover secp256k1 public key from runtime commit signature: {error}",
                ))
            })?;
        if recovered != *self.signing_key.verifying_key() {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "runtime commit signature recovered public key does not match signer selection {}",
                self.private_key_env,
            )));
        }
        Ok(())
    }

    fn sign_message(&self, message: &str) -> Result<(String, u8), ConfigError> {
        let (signature, recovery_id) = self
            .signing_key
            .sign_recoverable(message.as_bytes())
            .map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "failed to sign live runtime commit payload: {error}",
                ))
            })?;
        let signature_hex = encode_kolme_hex_lower(signature.to_bytes().as_ref());
        let recovery_id = recovery_id.to_byte();
        self.verify_message(message, signature_hex.as_str(), recovery_id)?;
        Ok((signature_hex, recovery_id))
    }
}

fn build_kolme_live_signer_adapter(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(KolmeForkSecp256k1SignerAdapter, KolmeLiveSignerSelection), ConfigError> {
    let (private_key_hex, selection) =
        read_kolme_live_signer_private_key_hex(strict_signer_profile, strict_signer_key_source)?;
    let signer_adapter = KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        private_key_hex.as_str(),
        selection.private_key_env,
    )?;
    Ok((signer_adapter, selection))
}

fn resolve_kolme_live_nonce(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    pubkey: &str,
) -> Result<u64, ConfigError> {
    let request = KolmeApiNextNonceRequest::new(pubkey)
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let response = transport
        .fetch_next_nonce(base_url, KOLME_LIVE_NONCE_PATH, &request)
        .map_err(|error| match error {
            KolmeRuntimeCommitProviderError::Timeout => {
                ConfigError::RuntimeKolmeLive("nonce request timed out".to_owned())
            }
            KolmeRuntimeCommitProviderError::Unavailable { reason } => {
                ConfigError::RuntimeKolmeLive(format!("nonce request unavailable: {reason}"))
            }
            KolmeRuntimeCommitProviderError::MalformedResponse { reason } => {
                ConfigError::RuntimeKolmeLive(format!("nonce response malformed: {reason}"))
            }
        })?;
    Ok(response.next_nonce)
}

fn render_kolme_live_native_direct_message(
    request: &KolmeRuntimeCommitRequest,
    pubkey: &str,
    nonce: u64,
) -> Result<String, ConfigError> {
    if nonce == 0 {
        return Err(ConfigError::RuntimeKolmeLive(
            "native direct-signed message nonce must be positive".to_owned(),
        ));
    }
    let pubkey = KolmeApiNextNonceRequest::new(pubkey)
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?
        .pubkey;
    request
        .validate()
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let metadata_message = format!(
        "{{\"type\":\"kamn-runtime-commit\",\"operation_id\":\"{}\",\"state_root\":\"{}\",\"actor_did\":\"{}\",\"payload_hash\":\"{}\",\"idempotency_key\":\"{}\",\"wire_payload\":\"{}\"}}",
        escape_kolme_json_string(request.operation_id.as_str()),
        escape_kolme_json_string(request.state_root.as_str()),
        escape_kolme_json_string(request.actor_did.as_str()),
        escape_kolme_json_string(request.payload_hash.as_str()),
        escape_kolme_json_string(request.idempotency_key()),
        escape_kolme_json_string(request.to_wire_payload().as_str()),
    );
    Ok(format!(
        "{{\"pubkey\":\"{}\",\"nonce\":{},\"created\":\"{}\",\"messages\":[{}],\"max_height\":null}}",
        escape_kolme_json_string(pubkey.as_str()),
        nonce,
        KOLME_LIVE_NATIVE_CREATED_AT,
        metadata_message
    ))
}

fn build_kolme_live_direct_signed_wire_payload(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    request: &KolmeRuntimeCommitRequest,
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let (signer_adapter, signer_selection) =
        build_kolme_live_signer_adapter(strict_signer_profile, strict_signer_key_source)?;
    let pubkey = signer_adapter.public_key_compressed_hex();
    let nonce = resolve_kolme_live_nonce(base_url, transport, pubkey.as_str())?;
    let canonical_message =
        render_kolme_live_native_direct_message(request, pubkey.as_str(), nonce)?;
    let (signature_hex, recovery_id) = signer_adapter.sign_message(canonical_message.as_str())?;
    let request = KolmeApiBroadcastRequest::new(
        canonical_message.as_str(),
        signature_hex.as_str(),
        recovery_id,
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    Ok((request.to_json_payload(), signer_selection))
}

fn ensure_kolme_live_provider_marker(expected: &str, observed: &str) -> Result<(), ConfigError> {
    if expected == observed {
        return Ok(());
    }
    Err(ConfigError::RuntimeKolmeLive(format!(
        "provider marker drift: expected '{expected}', observed '{observed}'"
    )))
}

fn kolme_live_finality_label(finality: KolmeCommitReceiptFinality) -> &'static str {
    match finality {
        KolmeCommitReceiptFinality::Pending => "pending",
        KolmeCommitReceiptFinality::Final => "final",
        KolmeCommitReceiptFinality::Failed => "failed",
    }
}

fn map_kolme_live_submit_outcome(
    outcome: KolmeRuntimeCommitProviderOutcome,
) -> Result<(&'static str, KolmeRuntimeCommitProviderReceipt), ConfigError> {
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => Ok(("submitted", receipt)),
        KolmeRuntimeCommitProviderOutcome::Duplicate(receipt) => Ok(("duplicate", receipt)),
        KolmeRuntimeCommitProviderOutcome::Rejected { reason } => {
            Err(ConfigError::RuntimeKolmeLive(format!(
                "provider rejected runtime commit submission: {reason}"
            )))
        }
    }
}

fn execute(cli: NodeCli) -> Result<NodeBootstrapReport, ConfigError> {
    let NodeCli {
        profile,
        role,
        chain_id,
        chain_version,
        storage_dir,
        enable_gossip,
        sync_mode,
        runtime_mode,
        expected_state_version,
        expected_state_hash,
        proposals,
        rejoin_attempts,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_peer_id,
        daemon_lifecycle_events,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_strict_signer_contracts,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        output_mode: _,
        diagnostics_mode,
    } = cli;
    let config = NodeConfig {
        chain_id: chain_id.clone(),
        chain_version: chain_version.clone(),
        role,
        storage_dir: storage_dir.clone(),
        enable_gossip,
        sync_mode,
    };

    let plan = bootstrap(config)?;
    let runtime_execution = match runtime_mode.kind {
        RuntimeModeKind::Bootstrap => RuntimeExecutionBundle::default(),
        RuntimeModeKind::Planning => {
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let planner = DeterministicProposalPlanner::new(&expected_state_hash);
            let proposal_plan = planner
                .plan(proposals)
                .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))?;
            RuntimeExecutionBundle {
                planning: Some(PlanningExecution {
                    expected_state_hash,
                    candidate_count: proposal_plan.ordered_candidates().len(),
                    scheduled_candidate_ids: proposal_plan.ordered_candidate_ids(),
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::RecoveryCheck => {
            let expected_state_version = expected_state_version.ok_or(
                ConfigError::MissingArgumentValue("--expected-state-version"),
            )?;
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let mut guard = RecoveryRejoinGuard::new(expected_state_version, &expected_state_hash)
                .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?;
            let mut decisions = Vec::with_capacity(rejoin_attempts.len());
            for attempt in rejoin_attempts {
                let decision = match guard
                    .evaluate(attempt)
                    .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?
                {
                    RecoveryStatus::RejoinAccepted => "rejoin-accepted".to_owned(),
                    RecoveryStatus::CatchUpRequired {
                        from_version,
                        to_version,
                    } => {
                        format!("catch-up-required:{from_version}->{to_version}")
                    }
                };
                decisions.push(decision);
            }
            RuntimeExecutionBundle {
                recovery: Some(RecoveryExecution {
                    expected_state_version,
                    expected_state_hash,
                    attempt_count: decisions.len(),
                    decisions,
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::Daemon => {
            let max_ticks =
                daemon_max_ticks.ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
            let tick_interval_ms = daemon_tick_interval_ms.ok_or(
                ConfigError::MissingArgumentValue("--daemon-tick-interval-ms"),
            )?;
            let (peer_id, peer_lifecycle_final_state, peer_lifecycle_applied_events) =
                match daemon_peer_id {
                    Some(peer_id) => {
                        let mut lifecycle = PeerLifecycle::new(&peer_id).map_err(|error| {
                            ConfigError::RuntimeDaemonLifecycle(error.to_string())
                        })?;
                        let mut applied_events = Vec::with_capacity(daemon_lifecycle_events.len());
                        for event in daemon_lifecycle_events {
                            lifecycle.transition(event).map_err(|error| {
                                ConfigError::RuntimeDaemonLifecycle(error.to_string())
                            })?;
                            applied_events.push(daemon_lifecycle_event_as_str(event).to_owned());
                        }
                        (
                            Some(peer_id),
                            Some(peer_lifecycle_state_as_str(lifecycle.state()).to_owned()),
                            Some(applied_events),
                        )
                    }
                    None => (None, None, None),
                };
            RuntimeExecutionBundle {
                daemon: Some(DaemonExecution {
                    max_ticks,
                    tick_interval_ms,
                    executed_ticks: max_ticks,
                    completion_reason: "tick-budget-exhausted".to_owned(),
                    peer_id,
                    peer_lifecycle_final_state,
                    peer_lifecycle_applied_events,
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::KolmeLive => {
            let base_url = kolme_live_base_url
                .ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?;
            let provider_hint = kolme_live_provider_hint.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-provider-hint"),
            )?;
            let signing_profile = kolme_live_signing_profile.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-signing-profile"),
            )?;
            if provider_hint.contains(KOLME_IN_MEMORY_PROVIDER_MARKER) {
                return Err(ConfigError::InvalidKolmeLiveProviderHint(provider_hint));
            }
            if signing_profile != KOLME_LIVE_SIGNING_PROFILE {
                return Err(ConfigError::InvalidKolmeLiveSigningProfile(signing_profile));
            }
            let strict_signer_profile = if kolme_live_strict_signer_contracts {
                Some(normalize_kolme_live_signer_profile_selector(
                    kolme_live_signer_profile.as_deref().ok_or(
                        ConfigError::MissingArgumentValue("--kolme-live-signer-profile"),
                    )?,
                )?)
            } else {
                None
            };
            let strict_signer_key_source = if kolme_live_strict_signer_contracts {
                Some(normalize_kolme_live_signer_key_source(
                    kolme_live_signer_key_source.as_deref().ok_or(
                        ConfigError::MissingArgumentValue("--kolme-live-signer-key-source"),
                    )?,
                )?)
            } else {
                None
            };
            let mut transport =
                KolmeRuntimeCommitHttpTransport::new(KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS)
                    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
            let request = build_kolme_live_request(&plan)?;
            let (signed_wire_payload, signer_selection) =
                build_kolme_live_direct_signed_wire_payload(
                    base_url.as_str(),
                    &mut transport,
                    &request,
                    strict_signer_profile,
                    strict_signer_key_source,
                )?;
            let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
                base_url.as_str(),
                provider_hint.as_str(),
                transport.clone(),
            )
            .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
            let submit_outcome = provider
                .submit_runtime_commit(signed_wire_payload.as_str(), request.idempotency_key())
                .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
            let (submit_status, mut receipt) = map_kolme_live_submit_outcome(submit_outcome)?;
            ensure_kolme_live_provider_marker(provider_hint.as_str(), receipt.provider.as_str())?;
            let mut resolution = "submit-receipt".to_owned();
            if matches!(receipt.finality, KolmeCommitReceiptFinality::Pending) {
                let mut checker = KolmeRuntimeCommitFinalityChecker::new(
                    base_url.as_str(),
                    KOLME_LIVE_FINALITY_STATUS_PATH,
                    transport,
                )
                .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
                match checker
                    .poll_finality(receipt.commit_id.as_str(), KOLME_LIVE_FINALITY_MAX_ATTEMPTS)
                {
                    Ok(polled_receipt) => {
                        ensure_kolme_live_provider_marker(
                            provider_hint.as_str(),
                            polled_receipt.provider.as_str(),
                        )?;
                        receipt = polled_receipt;
                        resolution = "finality-polled".to_owned();
                    }
                    Err(KolmeRuntimeCommitProviderError::Timeout) => {
                        resolution = "finality-timeout".to_owned();
                    }
                    Err(KolmeRuntimeCommitProviderError::Unavailable { .. }) => {
                        resolution = "finality-unavailable".to_owned();
                    }
                    Err(KolmeRuntimeCommitProviderError::MalformedResponse { reason }) => {
                        return Err(ConfigError::RuntimeKolmeLive(format!(
                            "finality response malformed: {reason}"
                        )));
                    }
                }
            }
            let finality = kolme_live_finality_label(receipt.finality);
            RuntimeExecutionBundle {
                kolme_live: Some(KolmeLiveExecution {
                    provider_client_contract: KOLME_LIVE_PROVIDER_CONTRACT.to_owned(),
                    base_url,
                    provider_hint,
                    signing_profile,
                    signer_profile_selector_env: KOLME_LIVE_SIGNER_PROFILE_ENV.to_owned(),
                    signer_profile: signer_selection.profile.to_owned(),
                    signer_key_source: signer_selection.key_source.to_owned(),
                    signer_private_key_env: signer_selection.private_key_env.to_owned(),
                    execution_status: format!(
                        "{submit_status};commit_id={};finality={finality};resolution={resolution}",
                        receipt.commit_id
                    ),
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
    };
    let report = build_bootstrap_report(
        &plan,
        profile,
        diagnostics_mode,
        runtime_mode,
        runtime_execution,
    );

    Ok(report)
}

fn build_bootstrap_report(
    plan: &BootstrapPlan,
    profile: Option<LocalProfile>,
    diagnostics_mode: DiagnosticsMode,
    runtime_mode: RuntimeMode,
    runtime_execution: RuntimeExecutionBundle,
) -> NodeBootstrapReport {
    let RuntimeExecutionBundle {
        planning,
        recovery,
        daemon,
        kolme_live,
    } = runtime_execution;
    let operational_profile = plan.config.operational_profile();
    let components = plan
        .wiring
        .all_components()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<String>>();
    let planning_expected_state_hash = planning
        .as_ref()
        .map(|planning| planning.expected_state_hash.clone());
    let planning_candidate_count = planning.as_ref().map(|planning| planning.candidate_count);
    let planning_scheduled_candidate_ids = planning
        .as_ref()
        .map(|planning| planning.scheduled_candidate_ids.clone());
    let recovery_expected_state_version = recovery
        .as_ref()
        .map(|recovery| recovery.expected_state_version);
    let recovery_expected_state_hash = recovery
        .as_ref()
        .map(|recovery| recovery.expected_state_hash.clone());
    let recovery_attempt_count = recovery.as_ref().map(|recovery| recovery.attempt_count);
    let recovery_decisions = recovery.as_ref().map(|recovery| recovery.decisions.clone());
    let daemon_max_ticks = daemon.as_ref().map(|daemon| daemon.max_ticks);
    let daemon_tick_interval_ms = daemon.as_ref().map(|daemon| daemon.tick_interval_ms);
    let daemon_executed_ticks = daemon.as_ref().map(|daemon| daemon.executed_ticks);
    let daemon_completion_reason = daemon
        .as_ref()
        .map(|daemon| daemon.completion_reason.clone());
    let daemon_peer_id = daemon.as_ref().and_then(|daemon| daemon.peer_id.clone());
    let daemon_peer_lifecycle_final_state = daemon
        .as_ref()
        .and_then(|daemon| daemon.peer_lifecycle_final_state.clone());
    let daemon_peer_lifecycle_applied_events = daemon
        .as_ref()
        .and_then(|daemon| daemon.peer_lifecycle_applied_events.clone());
    let kolme_live_provider_client_contract = kolme_live
        .as_ref()
        .map(|execution| execution.provider_client_contract.clone());
    let kolme_live_base_url = kolme_live
        .as_ref()
        .map(|execution| execution.base_url.clone());
    let kolme_live_provider_hint = kolme_live
        .as_ref()
        .map(|execution| execution.provider_hint.clone());
    let kolme_live_signing_profile = kolme_live
        .as_ref()
        .map(|execution| execution.signing_profile.clone());
    let kolme_live_signer_profile_selector_env = kolme_live
        .as_ref()
        .map(|execution| execution.signer_profile_selector_env.clone());
    let kolme_live_signer_profile = kolme_live
        .as_ref()
        .map(|execution| execution.signer_profile.clone());
    let kolme_live_signer_key_source = kolme_live
        .as_ref()
        .map(|execution| execution.signer_key_source.clone());
    let kolme_live_signer_private_key_env = kolme_live
        .as_ref()
        .map(|execution| execution.signer_private_key_env.clone());
    let kolme_live_execution_status = kolme_live
        .as_ref()
        .map(|execution| execution.execution_status.clone());
    NodeBootstrapReport {
        runtime_mode: runtime_mode.as_str().to_owned(),
        diagnostics_mode: diagnostics_mode.as_str().to_owned(),
        component_count: components.len(),
        planning_expected_state_hash,
        planning_candidate_count,
        planning_scheduled_candidate_ids,
        recovery_expected_state_version,
        recovery_expected_state_hash,
        recovery_attempt_count,
        recovery_decisions,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_executed_ticks,
        daemon_completion_reason,
        daemon_peer_id,
        daemon_peer_lifecycle_final_state,
        daemon_peer_lifecycle_applied_events,
        kolme_live_provider_client_contract,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_signer_profile_selector_env,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        kolme_live_signer_private_key_env,
        kolme_live_execution_status,
        profile: profile.map(LocalProfile::as_str).map(str::to_owned),
        role: plan.config.role.as_str().to_owned(),
        chain_id: plan.config.chain_id.clone(),
        chain_version: plan.config.chain_version.clone(),
        storage_dir: plan.config.storage_dir.clone(),
        gossip_enabled: plan.config.enable_gossip,
        sync_mode: plan.config.sync_mode.as_str().to_owned(),
        sync_startup: format!("{:?}", operational_profile.startup_strategy),
        sync_recovery: format!("{:?}", operational_profile.recovery_strategy),
        state_version: plan.state_schema.version.0,
        pending_migrations: plan.migration_plan.steps.len(),
        components,
    }
}

fn render_bootstrap_report(report: &NodeBootstrapReport, mode: OutputMode) -> String {
    match mode.kind {
        OutputModeKind::Text => render_text_report(report),
        OutputModeKind::Json => render_json_report(report),
    }
}

fn render_text_report(report: &NodeBootstrapReport) -> String {
    let profile = report.profile.as_deref().unwrap_or("none");
    let planning_expected_state_hash = report
        .planning_expected_state_hash
        .as_deref()
        .unwrap_or("none");
    let planning_candidate_count = report
        .planning_candidate_count
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let planning_scheduled_candidate_ids = report
        .planning_scheduled_candidate_ids
        .as_ref()
        .map(|value| value.join(", "))
        .unwrap_or_else(|| "none".to_owned());
    let recovery_expected_state_version = report
        .recovery_expected_state_version
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let recovery_expected_state_hash = report
        .recovery_expected_state_hash
        .as_deref()
        .unwrap_or("none");
    let recovery_attempt_count = report
        .recovery_attempt_count
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let recovery_decisions = report
        .recovery_decisions
        .as_ref()
        .map(|value| value.join(", "))
        .unwrap_or_else(|| "none".to_owned());
    let daemon_max_ticks = report
        .daemon_max_ticks
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_tick_interval_ms = report
        .daemon_tick_interval_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_executed_ticks = report
        .daemon_executed_ticks
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_completion_reason = report.daemon_completion_reason.as_deref().unwrap_or("none");
    let daemon_peer_id = report.daemon_peer_id.as_deref().unwrap_or("none");
    let daemon_peer_lifecycle_final_state = report
        .daemon_peer_lifecycle_final_state
        .as_deref()
        .unwrap_or("none");
    let daemon_peer_lifecycle_applied_events = report
        .daemon_peer_lifecycle_applied_events
        .as_ref()
        .map(|value| value.join(", "))
        .unwrap_or_else(|| "none".to_owned());
    let kolme_live_provider_client_contract = report
        .kolme_live_provider_client_contract
        .as_deref()
        .unwrap_or("none");
    let kolme_live_base_url = report.kolme_live_base_url.as_deref().unwrap_or("none");
    let kolme_live_provider_hint = report.kolme_live_provider_hint.as_deref().unwrap_or("none");
    let kolme_live_signing_profile = report
        .kolme_live_signing_profile
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_profile_selector_env = report
        .kolme_live_signer_profile_selector_env
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_profile = report
        .kolme_live_signer_profile
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_key_source = report
        .kolme_live_signer_key_source
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_private_key_env = report
        .kolme_live_signer_private_key_env
        .as_deref()
        .unwrap_or("none");
    let kolme_live_execution_status = report
        .kolme_live_execution_status
        .as_deref()
        .unwrap_or("none");
    format!(
        "KAMN node bootstrap\n  runtime_mode: {}\n  diagnostics_mode: {}\n  profile: {}\n  role: {}\n  chain: {} ({})\n  storage: {}\n  gossip: {}\n  sync_mode: {}\n  sync_startup: {}\n  sync_recovery: {}\n  state_version: {}\n  pending_migrations: {}\n  component_count: {}\n  planning_expected_state_hash: {}\n  planning_candidate_count: {}\n  planning_scheduled_candidate_ids: {}\n  recovery_expected_state_version: {}\n  recovery_expected_state_hash: {}\n  recovery_attempt_count: {}\n  recovery_decisions: {}\n  daemon_max_ticks: {}\n  daemon_tick_interval_ms: {}\n  daemon_executed_ticks: {}\n  daemon_completion_reason: {}\n  daemon_peer_id: {}\n  daemon_peer_lifecycle_final_state: {}\n  daemon_peer_lifecycle_applied_events: {}\n  kolme_live_provider_client_contract: {}\n  kolme_live_base_url: {}\n  kolme_live_provider_hint: {}\n  kolme_live_signing_profile: {}\n  kolme_live_signer_profile_selector_env: {}\n  kolme_live_signer_profile: {}\n  kolme_live_signer_key_source: {}\n  kolme_live_signer_private_key_env: {}\n  kolme_live_execution_status: {}\n  components: {}",
        report.runtime_mode,
        report.diagnostics_mode,
        profile,
        report.role,
        report.chain_id,
        report.chain_version,
        report.storage_dir,
        if report.gossip_enabled {
            "enabled"
        } else {
            "disabled"
        },
        report.sync_mode,
        report.sync_startup,
        report.sync_recovery,
        report.state_version,
        report.pending_migrations,
        report.component_count,
        planning_expected_state_hash,
        planning_candidate_count,
        planning_scheduled_candidate_ids,
        recovery_expected_state_version,
        recovery_expected_state_hash,
        recovery_attempt_count,
        recovery_decisions,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_executed_ticks,
        daemon_completion_reason,
        daemon_peer_id,
        daemon_peer_lifecycle_final_state,
        daemon_peer_lifecycle_applied_events,
        kolme_live_provider_client_contract,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_signer_profile_selector_env,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        kolme_live_signer_private_key_env,
        kolme_live_execution_status,
        report.components.join(", "),
    )
}

fn render_json_report(report: &NodeBootstrapReport) -> String {
    let profile = match &report.profile {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let planning_expected_state_hash = match &report.planning_expected_state_hash {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let planning_candidate_count = match report.planning_candidate_count {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let planning_scheduled_candidate_ids = match &report.planning_scheduled_candidate_ids {
        Some(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{}\"", json_escape(value)))
                .collect::<Vec<String>>()
                .join(",")
        ),
        None => "null".to_owned(),
    };
    let recovery_expected_state_version = match report.recovery_expected_state_version {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let recovery_expected_state_hash = match &report.recovery_expected_state_hash {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let recovery_attempt_count = match report.recovery_attempt_count {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let recovery_decisions = match &report.recovery_decisions {
        Some(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{}\"", json_escape(value)))
                .collect::<Vec<String>>()
                .join(",")
        ),
        None => "null".to_owned(),
    };
    let daemon_max_ticks = match report.daemon_max_ticks {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_tick_interval_ms = match report.daemon_tick_interval_ms {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_executed_ticks = match report.daemon_executed_ticks {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_completion_reason = match &report.daemon_completion_reason {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let daemon_peer_id = match &report.daemon_peer_id {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let daemon_peer_lifecycle_final_state = match &report.daemon_peer_lifecycle_final_state {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let daemon_peer_lifecycle_applied_events = match &report.daemon_peer_lifecycle_applied_events {
        Some(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{}\"", json_escape(value)))
                .collect::<Vec<String>>()
                .join(",")
        ),
        None => "null".to_owned(),
    };
    let kolme_live_provider_client_contract = match &report.kolme_live_provider_client_contract {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_base_url = match &report.kolme_live_base_url {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_provider_hint = match &report.kolme_live_provider_hint {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signing_profile = match &report.kolme_live_signing_profile {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signer_profile_selector_env =
        match &report.kolme_live_signer_profile_selector_env {
            Some(value) => format!("\"{}\"", json_escape(value)),
            None => "null".to_owned(),
        };
    let kolme_live_signer_profile = match &report.kolme_live_signer_profile {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signer_key_source = match &report.kolme_live_signer_key_source {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signer_private_key_env = match &report.kolme_live_signer_private_key_env {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_execution_status = match &report.kolme_live_execution_status {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let components = report
        .components
        .iter()
        .map(|component| format!("\"{}\"", json_escape(component)))
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\"runtime_mode\":\"{}\",\"diagnostics_mode\":\"{}\",\"profile\":{},\"role\":\"{}\",\"chain_id\":\"{}\",\"chain_version\":\"{}\",\"storage_dir\":\"{}\",\"gossip_enabled\":{},\"sync_mode\":\"{}\",\"sync_startup\":\"{}\",\"sync_recovery\":\"{}\",\"state_version\":{},\"pending_migrations\":{},\"component_count\":{},\"planning_expected_state_hash\":{},\"planning_candidate_count\":{},\"planning_scheduled_candidate_ids\":{},\"recovery_expected_state_version\":{},\"recovery_expected_state_hash\":{},\"recovery_attempt_count\":{},\"recovery_decisions\":{},\"daemon_max_ticks\":{},\"daemon_tick_interval_ms\":{},\"daemon_executed_ticks\":{},\"daemon_completion_reason\":{},\"daemon_peer_id\":{},\"daemon_peer_lifecycle_final_state\":{},\"daemon_peer_lifecycle_applied_events\":{},\"kolme_live_provider_client_contract\":{},\"kolme_live_base_url\":{},\"kolme_live_provider_hint\":{},\"kolme_live_signing_profile\":{},\"kolme_live_signer_profile_selector_env\":{},\"kolme_live_signer_profile\":{},\"kolme_live_signer_key_source\":{},\"kolme_live_signer_private_key_env\":{},\"kolme_live_execution_status\":{},\"components\":[{}]}}",
        json_escape(&report.runtime_mode),
        json_escape(&report.diagnostics_mode),
        profile,
        json_escape(&report.role),
        json_escape(&report.chain_id),
        json_escape(&report.chain_version),
        json_escape(&report.storage_dir),
        report.gossip_enabled,
        json_escape(&report.sync_mode),
        json_escape(&report.sync_startup),
        json_escape(&report.sync_recovery),
        report.state_version,
        report.pending_migrations,
        report.component_count,
        planning_expected_state_hash,
        planning_candidate_count,
        planning_scheduled_candidate_ids,
        recovery_expected_state_version,
        recovery_expected_state_hash,
        recovery_attempt_count,
        recovery_decisions,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_executed_ticks,
        daemon_completion_reason,
        daemon_peer_id,
        daemon_peer_lifecycle_final_state,
        daemon_peer_lifecycle_applied_events,
        kolme_live_provider_client_contract,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_signer_profile_selector_env,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        kolme_live_signer_private_key_env,
        kolme_live_execution_status,
        components,
    )
}

fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_bootstrap_report, build_kolme_live_direct_signed_wire_payload,
        build_kolme_live_signer_adapter, execute, parse_args, render_bootstrap_report,
        render_kolme_live_native_direct_message, resolve_kolme_live_nonce,
        resolve_kolme_live_signer_private_key_env_name, DiagnosticsMode, LocalProfile,
        NodeBootstrapReport, OutputMode, RuntimeExecutionBundle, RuntimeMode,
    };
    use kamn_core::{
        bootstrap, ConfigError, KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitRequest,
        NodeConfig, NodeRole, SyncMode,
    };
    use std::io::{ErrorKind, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};
    use std::{env, sync::OnceLock};

    const TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX: &str =
        "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
    const TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY: &str =
        "838c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

    fn signer_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = env::var(key).ok();
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_deref() {
                env::set_var(self.key, previous);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[derive(Clone)]
    struct MockHttpReply {
        status_line: &'static str,
        body: String,
    }

    impl MockHttpReply {
        fn ok(body: &str) -> Self {
            Self {
                status_line: "HTTP/1.1 200 OK",
                body: body.to_owned(),
            }
        }
    }

    fn read_http_request(stream: &mut std::net::TcpStream) -> String {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        let mut header_end = None;
        let mut expected_total = None;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("read timeout should be set");

        loop {
            let read_count = match stream.read(&mut chunk) {
                Ok(read_count) => read_count,
                Err(error)
                    if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    break;
                }
                Err(error) => panic!("request bytes should be readable: {error}"),
            };
            if read_count == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read_count]);

            if header_end.is_none() {
                header_end = buffer
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|pos| pos + 4);
                if let Some(end) = header_end {
                    let headers = String::from_utf8_lossy(&buffer[..end]);
                    let content_length = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            if name.eq_ignore_ascii_case("Content-Length") {
                                return value.trim().parse::<usize>().ok();
                            }
                            None
                        })
                        .unwrap_or(0);
                    expected_total = Some(end + content_length);
                }
            }
            if let Some(total) = expected_total {
                if buffer.len() >= total {
                    break;
                }
            }
        }

        String::from_utf8(buffer).expect("request should be valid utf-8")
    }

    fn request_body(raw_request: &str) -> &str {
        raw_request
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .unwrap_or("")
    }

    fn extract_json_string_field(body: &str, field: &str) -> Option<String> {
        let marker = format!("\"{field}\":\"");
        let start = body.find(marker.as_str())?;
        let remainder = &body[start + marker.len()..];
        let end = remainder.find('"')?;
        Some(remainder[..end].to_owned())
    }

    fn spawn_kolme_live_mock_server(
        replies: Vec<MockHttpReply>,
    ) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        listener
            .set_nonblocking(true)
            .expect("listener should allow nonblocking accepts");
        let addr = listener.local_addr().expect("local addr should resolve");
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let recorded_requests_ref = Arc::clone(&recorded_requests);
        thread::spawn(move || {
            for reply in replies {
                let accept_deadline = Instant::now() + Duration::from_secs(2);
                loop {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let request = read_http_request(&mut stream);
                            recorded_requests_ref
                                .lock()
                                .expect("request mutex should lock")
                                .push(request);
                            let response = format!(
                                "{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                reply.status_line,
                                reply.body.len(),
                                reply.body
                            );
                            stream
                                .write_all(response.as_bytes())
                                .expect("response should write");
                            break;
                        }
                        Err(error) if error.kind() == ErrorKind::WouldBlock => {
                            if Instant::now() >= accept_deadline {
                                return;
                            }
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("accept should succeed: {error}"),
                    }
                }
            }
        });
        (format!("http://{addr}"), recorded_requests)
    }

    #[test]
    fn parses_required_role_and_defaults() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.profile, None);
        assert_eq!(parsed.role, NodeRole::Processor);
        assert_eq!(parsed.chain_id, "kamn-devnet");
        assert_eq!(parsed.chain_version, "v0.1.0");
        assert_eq!(parsed.storage_dir, "./data");
        assert!(parsed.enable_gossip);
        assert_eq!(parsed.sync_mode, SyncMode::Fast);
        assert_eq!(parsed.runtime_mode, RuntimeMode::bootstrap());
        assert_eq!(parsed.expected_state_hash, None);
        assert_eq!(parsed.expected_state_version, None);
        assert!(parsed.proposals.is_empty());
        assert!(parsed.rejoin_attempts.is_empty());
        assert_eq!(parsed.daemon_max_ticks, None);
        assert_eq!(parsed.daemon_tick_interval_ms, None);
        assert_eq!(parsed.daemon_peer_id, None);
        assert!(parsed.daemon_lifecycle_events.is_empty());
        assert_eq!(parsed.kolme_live_base_url, None);
        assert_eq!(parsed.kolme_live_provider_hint, None);
        assert_eq!(parsed.kolme_live_signing_profile, None);
        assert_eq!(parsed.output_mode, OutputMode::text());
        assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::basic());
    }

    #[test]
    fn parses_disable_gossip_flag() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "listener".to_owned(),
            "--disable-gossip".to_owned(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.role, NodeRole::Listener);
        assert!(!parsed.enable_gossip);
        assert_eq!(parsed.sync_mode, SyncMode::Fast);
    }

    #[test]
    fn parses_sync_mode_flag() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--sync-mode".to_owned(),
            "archive".to_owned(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.sync_mode, SyncMode::Archive);
    }

    #[test]
    fn parses_output_mode_json_flag() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.output_mode, OutputMode::json());
    }

    #[test]
    fn parses_diagnostics_snapshot_flag() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--diagnostics".to_owned(),
            "snapshot".to_owned(),
        ];

        let parsed = parse_args(args).expect("diagnostics args should parse");
        assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::snapshot());
    }

    #[test]
    fn parses_runtime_mode_planning_with_proposals() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "planning".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-1".to_owned(),
            "--proposal".to_owned(),
            "tx-2|did:kamn:agent:bbb|2|state-1".to_owned(),
            "--proposal".to_owned(),
            "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
        ];

        let parsed = parse_args(args).expect("planning args should parse");
        assert_eq!(parsed.runtime_mode, RuntimeMode::planning());
        assert_eq!(parsed.expected_state_hash, Some("state-1".to_owned()));
        assert_eq!(parsed.proposals.len(), 2);
    }

    #[test]
    fn parses_runtime_mode_recovery_check_with_rejoin_attempt() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-42|resume-1".to_owned(),
        ];

        let parsed = parse_args(args).expect("recovery-check args should parse");
        assert_eq!(parsed.runtime_mode, RuntimeMode::recovery_check());
        assert_eq!(parsed.expected_state_version, Some(42));
        assert_eq!(parsed.expected_state_hash, Some("state-42".to_owned()));
        assert_eq!(parsed.rejoin_attempts.len(), 1);
    }

    #[test]
    fn parses_runtime_mode_daemon_with_bounded_controls() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "3".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "25".to_owned(),
            "--daemon-peer-id".to_owned(),
            "peer-alpha".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "start-connect".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "handshake-succeeded".to_owned(),
        ];

        let parsed = parse_args(args).expect("daemon args should parse");
        assert_eq!(parsed.runtime_mode, RuntimeMode::daemon());
        assert_eq!(parsed.daemon_max_ticks, Some(3));
        assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
        assert_eq!(parsed.daemon_peer_id, Some("peer-alpha".to_owned()));
        assert_eq!(parsed.daemon_lifecycle_events.len(), 2);
    }

    #[test]
    fn parses_runtime_mode_kolme_live_with_required_flags() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
        ];

        let parsed = parse_args(args).expect("kolme-live args should parse");
        assert_eq!(parsed.runtime_mode.as_str(), "kolme-live");
        assert_eq!(
            parsed.kolme_live_base_url,
            Some("http://127.0.0.1:3000".to_owned())
        );
        assert_eq!(
            parsed.kolme_live_provider_hint,
            Some("kolme-fork-local".to_owned())
        );
        assert_eq!(
            parsed.kolme_live_signing_profile,
            Some("kolme-fork-secp256k1-v1".to_owned())
        );
        assert!(!parsed.kolme_live_strict_signer_contracts);
        assert_eq!(parsed.kolme_live_signer_profile, None);
        assert_eq!(parsed.kolme_live_signer_key_source, None);
    }

    #[test]
    fn parses_local_listener_profile_defaults() {
        let args = vec![
            "kamn-node".to_owned(),
            "--profile".to_owned(),
            "local-listener".to_owned(),
        ];

        let parsed = parse_args(args).expect("profile args should parse");
        assert_eq!(parsed.profile, Some(LocalProfile::Listener));
        assert_eq!(parsed.role, NodeRole::Listener);
        assert_eq!(parsed.chain_id, "kamn-localnet");
        assert_eq!(parsed.storage_dir, "./data/listener");
        assert_eq!(parsed.sync_mode, SyncMode::Fast);
        assert!(parsed.enable_gossip);
    }

    #[test]
    fn profile_defaults_can_be_overridden_by_explicit_flags() {
        let args = vec![
            "kamn-node".to_owned(),
            "--profile".to_owned(),
            "local-listener".to_owned(),
            "--chain-id".to_owned(),
            "kamn-custom".to_owned(),
            "--storage-dir".to_owned(),
            "./tmp/custom-listener".to_owned(),
            "--disable-gossip".to_owned(),
            "--sync-mode".to_owned(),
            "archive".to_owned(),
        ];

        let parsed = parse_args(args).expect("profile args with overrides should parse");
        assert_eq!(parsed.profile, Some(LocalProfile::Listener));
        assert_eq!(parsed.role, NodeRole::Listener);
        assert_eq!(parsed.chain_id, "kamn-custom");
        assert_eq!(parsed.storage_dir, "./tmp/custom-listener");
        assert_eq!(parsed.sync_mode, SyncMode::Archive);
        assert!(!parsed.enable_gossip);
    }

    #[test]
    fn functional_json_render_is_deterministic() {
        let report = NodeBootstrapReport {
            runtime_mode: "bootstrap".to_owned(),
            diagnostics_mode: "basic".to_owned(),
            component_count: 2,
            planning_expected_state_hash: None,
            planning_candidate_count: None,
            planning_scheduled_candidate_ids: None,
            recovery_expected_state_version: None,
            recovery_expected_state_hash: None,
            recovery_attempt_count: None,
            recovery_decisions: None,
            daemon_max_ticks: None,
            daemon_tick_interval_ms: None,
            daemon_executed_ticks: None,
            daemon_completion_reason: None,
            daemon_peer_id: None,
            daemon_peer_lifecycle_final_state: None,
            daemon_peer_lifecycle_applied_events: None,
            kolme_live_provider_client_contract: None,
            kolme_live_base_url: None,
            kolme_live_provider_hint: None,
            kolme_live_signing_profile: None,
            kolme_live_signer_profile_selector_env: None,
            kolme_live_signer_profile: None,
            kolme_live_signer_key_source: None,
            kolme_live_signer_private_key_env: None,
            kolme_live_execution_status: None,
            profile: None,
            role: "processor".to_owned(),
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            storage_dir: "./data".to_owned(),
            gossip_enabled: true,
            sync_mode: "fast".to_owned(),
            sync_startup: "StateSyncToLatest".to_owned(),
            sync_recovery: "ResumeRecentState".to_owned(),
            state_version: 1,
            pending_migrations: 0,
            components: vec!["processor".to_owned(), "listener".to_owned()],
        };

        let first = render_bootstrap_report(&report, OutputMode::json());
        let second = render_bootstrap_report(&report, OutputMode::json());
        assert_eq!(first, second, "json output should be deterministic");
        assert!(first.contains("\"role\":\"processor\""));
        assert!(first.contains("\"components\":[\"processor\",\"listener\"]"));
    }

    #[test]
    fn integration_parse_bootstrap_and_render_json() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let parsed = parse_args(args).expect("args should parse");
        let config = NodeConfig {
            chain_id: parsed.chain_id,
            chain_version: parsed.chain_version,
            role: parsed.role,
            storage_dir: parsed.storage_dir,
            enable_gossip: parsed.enable_gossip,
            sync_mode: parsed.sync_mode,
        };
        let plan = bootstrap(config).expect("bootstrap should succeed");
        let report = build_bootstrap_report(
            &plan,
            parsed.profile,
            parsed.diagnostics_mode,
            RuntimeMode::bootstrap(),
            RuntimeExecutionBundle::default(),
        );
        let rendered = render_bootstrap_report(&report, parsed.output_mode);

        assert!(rendered.contains("\"diagnostics_mode\":\"basic\""));
        assert!(rendered.contains("\"profile\":null"));
        assert!(rendered.contains("\"role\":\"processor\""));
        assert!(rendered.contains("\"chain_id\":\"kamn-devnet\""));
        assert!(rendered.contains("\"sync_mode\":\"fast\""));
        assert!(rendered.contains("\"components\":["));
    }

    #[test]
    fn integration_profile_bootstrap_and_render_json() {
        let args = vec![
            "kamn-node".to_owned(),
            "--profile".to_owned(),
            "local-listener".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let parsed = parse_args(args).expect("profile args should parse");
        let config = NodeConfig {
            chain_id: parsed.chain_id,
            chain_version: parsed.chain_version,
            role: parsed.role,
            storage_dir: parsed.storage_dir,
            enable_gossip: parsed.enable_gossip,
            sync_mode: parsed.sync_mode,
        };
        let plan = bootstrap(config).expect("bootstrap should succeed");
        let report = build_bootstrap_report(
            &plan,
            parsed.profile,
            parsed.diagnostics_mode,
            RuntimeMode::bootstrap(),
            RuntimeExecutionBundle::default(),
        );
        let rendered = render_bootstrap_report(&report, parsed.output_mode);

        assert!(rendered.contains("\"diagnostics_mode\":\"basic\""));
        assert!(rendered.contains("\"profile\":\"local-listener\""));
        assert!(rendered.contains("\"role\":\"listener\""));
        assert!(rendered.contains("\"chain_id\":\"kamn-localnet\""));
        assert!(rendered.contains("\"storage_dir\":\"./data/listener\""));
    }

    #[test]
    fn integration_diagnostics_snapshot_includes_component_count() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
            "--diagnostics".to_owned(),
            "snapshot".to_owned(),
        ];
        let parsed = parse_args(args).expect("diagnostics args should parse");
        let config = NodeConfig {
            chain_id: parsed.chain_id,
            chain_version: parsed.chain_version,
            role: parsed.role,
            storage_dir: parsed.storage_dir,
            enable_gossip: parsed.enable_gossip,
            sync_mode: parsed.sync_mode,
        };
        let plan = bootstrap(config).expect("bootstrap should succeed");
        let report = build_bootstrap_report(
            &plan,
            parsed.profile,
            parsed.diagnostics_mode,
            RuntimeMode::bootstrap(),
            RuntimeExecutionBundle::default(),
        );
        let rendered = render_bootstrap_report(&report, parsed.output_mode);

        assert!(rendered.contains("\"diagnostics_mode\":\"snapshot\""));
        assert!(rendered.contains("\"component_count\":"));
    }

    #[test]
    fn integration_runtime_planning_renders_sorted_candidate_ids() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "planning".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-1".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
            "--proposal".to_owned(),
            "tx-2|did:kamn:agent:bbb|2|state-1".to_owned(),
            "--proposal".to_owned(),
            "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
        ];

        let parsed = parse_args(args).expect("planning args should parse");
        let report = execute(parsed).expect("planning execution should succeed");
        let rendered = render_bootstrap_report(&report, OutputMode::json());
        assert!(rendered.contains("\"runtime_mode\":\"planning\""));
        assert!(rendered.contains("\"planning_candidate_count\":2"));
        assert!(rendered.contains("\"planning_scheduled_candidate_ids\":[\"tx-1\",\"tx-2\"]"));
    }

    #[test]
    fn integration_runtime_recovery_check_renders_deterministic_decision_output() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|40|state-40|resume-1".to_owned(),
        ];

        let parsed = parse_args(args).expect("recovery-check args should parse");
        let report = execute(parsed).expect("recovery-check execution should succeed");
        let rendered = render_bootstrap_report(&report, OutputMode::json());
        assert!(rendered.contains("\"runtime_mode\":\"recovery-check\""));
        assert!(rendered.contains("\"recovery_expected_state_version\":42"));
        assert!(rendered.contains("\"recovery_expected_state_hash\":\"state-42\""));
        assert!(rendered.contains("\"recovery_attempt_count\":1"));
        assert!(rendered.contains("\"recovery_decisions\":[\"catch-up-required:40->42\"]"));
    }

    #[test]
    fn integration_runtime_daemon_renders_bounded_completion_output() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "3".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "25".to_owned(),
            "--daemon-peer-id".to_owned(),
            "peer-alpha".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "start-connect".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "handshake-succeeded".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "heartbeat-missed".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "heartbeat-restored".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let parsed = parse_args(args).expect("daemon args should parse");
        let report = execute(parsed).expect("daemon execution should succeed");
        let rendered = render_bootstrap_report(&report, OutputMode::json());
        assert!(rendered.contains("\"runtime_mode\":\"daemon\""));
        assert!(rendered.contains("\"daemon_max_ticks\":3"));
        assert!(rendered.contains("\"daemon_tick_interval_ms\":25"));
        assert!(rendered.contains("\"daemon_executed_ticks\":3"));
        assert!(rendered.contains("\"daemon_completion_reason\":\"tick-budget-exhausted\""));
        assert!(rendered.contains("\"daemon_peer_id\":\"peer-alpha\""));
        assert!(rendered.contains("\"daemon_peer_lifecycle_final_state\":\"active\""));
        assert!(
            rendered.contains(
                "\"daemon_peer_lifecycle_applied_events\":[\"start-connect\",\"handshake-succeeded\",\"heartbeat-missed\",\"heartbeat-restored\"]"
            )
        );
    }

    #[test]
    fn integration_runtime_kolme_live_renders_provider_contract_markers() {
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _env_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let (base_url, requests) = spawn_kolme_live_mock_server(vec![
            MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
            MockHttpReply::ok(
                r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
            ),
            MockHttpReply::ok(
                r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
            ),
        ]);
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            base_url,
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let parsed = parse_args(args).expect("kolme-live args should parse");
        let report = execute(parsed).expect("kolme-live execution should succeed");
        let rendered = render_bootstrap_report(&report, OutputMode::json());
        assert!(rendered.contains("\"runtime_mode\":\"kolme-live\""));
        assert!(rendered.contains(
            "\"kolme_live_provider_client_contract\":\"KolmeRuntimeCommitLiveProvider\""
        ));
        assert!(rendered.contains("\"kolme_live_signing_profile\":\"kolme-fork-secp256k1-v1\""));
        assert!(rendered.contains(
            "\"kolme_live_signer_profile_selector_env\":\"KAMN_KOLME_LIVE_SIGNER_PROFILE\""
        ));
        assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-primary\""));
        assert!(rendered.contains("\"kolme_live_signer_key_source\":\"env-local\""));
        assert!(rendered.contains(
            "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX\""
        ));
        assert!(rendered.contains("\"kolme_live_execution_status\":\"submitted;"));

        let recorded_requests = requests.lock().expect("request mutex should lock");
        assert_eq!(
            recorded_requests.len(),
            3,
            "live runtime should issue nonce, submit, and finality requests"
        );
        assert!(recorded_requests[0].contains("GET /get-next-nonce?pubkey="));
        assert!(recorded_requests[1].contains("PUT /broadcast HTTP/1.1"));
        assert!(recorded_requests[1].contains("X-Idempotency-Key: "));
        let signature =
            extract_json_string_field(request_body(recorded_requests[1].as_str()), "signature")
                .expect("submit request should contain signature JSON field");
        // Regression: #2197
        assert!(
            signature.len() == 128
                && signature
                    .as_bytes()
                    .iter()
                    .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
            "live runtime submit must not fall back to synthetic idempotency-key signatures"
        );
        assert!(recorded_requests[2]
            .contains("GET /runtime-commit/status?commit_id=kolme-commit%3Aab12cd34 HTTP/1.1"));
    }

    #[test]
    fn unit_kolme_live_signer_builds_direct_signed_wire_payload() {
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _env_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let request = KolmeRuntimeCommitRequest::deterministic(
            "op-node-live-2197",
            "state:node-live-2197",
            "kamn:did:agent:node-live-2197",
            1,
            "payload:node-live-2197",
        )
        .expect("request should build");

        let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
            r#"{"next_nonce":22,"account_id":"acct-2197"}"#,
        )]);
        let mut transport =
            KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
        let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
            base_url.as_str(),
            &mut transport,
            &request,
            None,
            None,
        )
        .expect("signed payload should be produced");

        assert_eq!(signer_selection.profile, "ops-primary");
        assert_eq!(
            signer_selection.private_key_env,
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
        );
        assert_eq!(signer_selection.key_source, "env-local");
        assert!(signed_wire_payload.contains("\"message\":\"{\\\"pubkey\\\":"));
        let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
            .expect("direct signed payload must include signature field");
        assert_eq!(
            signature.len(),
            128,
            "secp256k1 signature must be 64 bytes hex"
        );
        assert!(
            signature
                .as_bytes()
                .iter()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
            "signature must be lowercase hex"
        );
    }

    #[test]
    fn unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message() {
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _env_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let message = "{\"pubkey\":\"pk-adapter\",\"nonce\":7,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}";
        let (adapter, selection) =
            build_kolme_live_signer_adapter(None, None).expect("adapter should build");
        assert_eq!(selection.profile, "ops-primary");
        let (signature_hex, recovery_id) = adapter
            .sign_message(message)
            .expect("adapter signing should succeed");
        assert_eq!(signature_hex.len(), 128);
        assert!(
            signature_hex
                .as_bytes()
                .iter()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
            "adapter signature must be lowercase hex"
        );
        adapter
            .verify_message(message, signature_hex.as_str(), recovery_id)
            .expect("adapter signature verification should succeed");
    }

    #[test]
    fn regression_kolme_live_signer_adapter_rejects_malformed_signature_hex() {
        // Regression: #2297
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _env_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let (adapter, _selection) =
            build_kolme_live_signer_adapter(None, None).expect("adapter should build");
        assert!(
            matches!(
                adapter.verify_message(
                    "{\"pubkey\":\"pk-adapter\",\"nonce\":7,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}",
                    "zz",
                    0,
                ),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("runtime_commit_signature_hex contains invalid hex character")
            ),
            "malformed signature hex must fail closed in adapter verification"
        );
    }

    #[test]
    fn regression_kolme_live_signer_adapter_rejects_recovered_key_mismatch() {
        // Regression: #2297
        let primary = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
            TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        )
        .expect("primary adapter should build");
        let secondary = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
            TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY,
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        )
        .expect("secondary adapter should build");
        let message = "{\"pubkey\":\"pk-adapter\",\"nonce\":9,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}";
        let (signature_hex, recovery_id) = primary
            .sign_message(message)
            .expect("primary adapter signature should succeed");
        assert!(
            matches!(
                secondary.verify_message(message, signature_hex.as_str(), recovery_id),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("recovered public key does not match signer selection")
            ),
            "signature verification must fail closed when recovered key mismatches signer adapter key"
        );
    }

    #[test]
    fn unit_kolme_live_signer_profile_defaults_to_primary_key_env() {
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", None);

        let (profile, env_name) = resolve_kolme_live_signer_private_key_env_name(None)
            .expect("default profile selection should succeed");
        assert_eq!(profile, "ops-primary");
        assert_eq!(env_name, "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX");
    }

    #[test]
    fn regression_kolme_live_signer_profile_rejects_unsupported_value() {
        // Regression: #2222
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("legacy"));
        assert!(
            matches!(
                resolve_kolme_live_signer_private_key_env_name(None),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("KAMN_KOLME_LIVE_SIGNER_PROFILE has unsupported profile")
            ),
            "unsupported signer profile must fail closed"
        );
    }

    #[test]
    fn integration_kolme_live_signer_profile_secondary_uses_secondary_key_env() {
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
        let _secondary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
        );
        let request = KolmeRuntimeCommitRequest::deterministic(
            "op-node-live-2222",
            "state:node-live-2222",
            "kamn:did:agent:node-live-2222",
            1,
            "payload:node-live-2222",
        )
        .expect("request should build");

        let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
            r#"{"next_nonce":31,"account_id":"acct-2222"}"#,
        )]);
        let mut transport =
            KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
        let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
            base_url.as_str(),
            &mut transport,
            &request,
            None,
            None,
        )
        .expect("secondary profile signing should succeed");
        assert_eq!(signer_selection.profile, "ops-secondary");
        assert_eq!(
            signer_selection.private_key_env,
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
        );
        assert_eq!(signer_selection.key_source, "env-local");
        let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
            .expect("direct signed payload must include signature field");
        assert_eq!(signature.len(), 128);
    }

    #[test]
    fn integration_runtime_kolme_live_renders_secondary_signer_selection_markers() {
        // Regression: #2241
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
        let _secondary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
        );
        let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
            MockHttpReply::ok(r#"{"next_nonce":37,"account_id":"acct-live-secondary"}"#),
            MockHttpReply::ok(
                r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ef56ab78","finality":"final"}"#,
            ),
        ]);
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            base_url,
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let parsed = parse_args(args).expect("kolme-live args should parse");
        let report = execute(parsed).expect("kolme-live execution should succeed");
        let rendered = render_bootstrap_report(&report, OutputMode::json());
        assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-secondary\""));
        assert!(rendered.contains(
            "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY\""
        ));
        assert!(rendered.contains("\"kolme_live_signer_key_source\":\"env-local\""));
    }

    #[test]
    fn unit_kolme_live_native_direct_message_contains_required_fields() {
        let request = KolmeRuntimeCommitRequest::deterministic(
            "op-node-live-2207",
            "state:node-live-2207",
            "kamn:did:agent:node-live-2207",
            1,
            "payload:node-live-2207",
        )
        .expect("request should build");

        let message = render_kolme_live_native_direct_message(
            &request,
            "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
            19,
        )
        .expect("native direct message should render");

        assert!(message.contains(
            "\"pubkey\":\"02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344\""
        ));
        assert!(message.contains("\"nonce\":19"));
        assert!(message.contains("\"created\":\""));
        assert!(message.contains("\"messages\":["));
    }

    #[test]
    fn integration_kolme_live_nonce_resolver_fetches_next_nonce() {
        let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
            r#"{"next_nonce":27,"account_id":"acct-2207"}"#,
        )]);
        let mut transport =
            KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
        let signer_adapter = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
            TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        )
        .expect("deterministic signer adapter should build");
        let pubkey = signer_adapter.public_key_compressed_hex();

        let nonce = resolve_kolme_live_nonce(base_url.as_str(), &mut transport, pubkey.as_str())
            .expect("nonce should resolve");
        assert_eq!(nonce, 27);

        let recorded_requests = requests.lock().expect("request mutex should lock");
        assert_eq!(recorded_requests.len(), 1);
        assert!(recorded_requests[0].contains("GET /get-next-nonce?pubkey="));
    }

    #[test]
    fn regression_kolme_live_nonce_resolver_rejects_malformed_response() {
        // Regression: #2207
        let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
            r#"{"next_nonce":0,"account_id":"acct-2207"}"#,
        )]);
        let mut transport =
            KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
        let error = resolve_kolme_live_nonce(
            base_url.as_str(),
            &mut transport,
            "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
        )
        .expect_err("invalid nonce payload must fail closed");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("nonce response malformed")),
            "expected fail-closed nonce parser error"
        );
    }

    #[test]
    fn regression_kolme_live_signer_requires_primary_key_env_value() {
        // Regression: #2222
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
        assert!(
            matches!(
                build_kolme_live_signer_adapter(None, None),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX must be set")
            ),
            "missing primary signer private key env must fail closed"
        );
    }

    #[test]
    fn regression_issue_2279_kolme_live_signer_rejects_fallback_private_key_env_path() {
        // Regression: #2279
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _primary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let _fallback_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
        );
        assert!(
            matches!(
                build_kolme_live_signer_adapter(Some("ops-primary"), Some("env-local")),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("fallback_signer_secret_present_violation")
            ),
            "fallback signer private key env path must fail closed"
        );
    }

    #[test]
    fn regression_runtime_kolme_live_rejects_provider_marker_drift() {
        // Regression: #2176
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _env_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let (base_url, requests) = spawn_kolme_live_mock_server(vec![
            MockHttpReply::ok(r#"{"next_nonce":23,"account_id":"acct-2176"}"#),
            MockHttpReply::ok(
                r#"{"status":"submitted","provider":"unexpected-provider","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
            ),
        ]);
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            base_url,
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let parsed = parse_args(args).expect("kolme-live args should parse");
        assert!(
            matches!(
                execute(parsed),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("provider marker drift")
            ),
            "runtime must fail closed when provider marker drifts from configured hint"
        );
        let recorded_requests = requests.lock().expect("request mutex should lock");
        assert_eq!(
            recorded_requests.len(),
            2,
            "provider drift should fail after nonce lookup and submit response mapping"
        );
    }

    #[test]
    fn regression_runtime_kolme_live_rejects_missing_signer_private_key_env() {
        // Regression: #2220
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
        let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        )]);
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            base_url,
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let parsed = parse_args(args).expect("kolme-live args should parse");
        assert!(
            matches!(
                execute(parsed),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX must be set for signer profile ops-primary")
            ),
            "runtime must fail closed when signer private key env is missing"
        );
    }

    #[test]
    fn rejects_missing_role() {
        let args = vec!["kamn-node".to_owned()];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue("--role"))
        );
    }

    #[test]
    fn rejects_planning_without_expected_state_hash() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "planning".to_owned(),
            "--proposal".to_owned(),
            "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue("--expected-state-hash"))
        );
    }

    #[test]
    fn rejects_planning_without_proposal() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "planning".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue("--proposal"))
        );
    }

    #[test]
    fn rejects_recovery_check_without_expected_state_version() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-42|resume-1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue(
                "--expected-state-version"
            ))
        );
    }

    #[test]
    fn rejects_recovery_check_without_expected_state_hash() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-42|resume-1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue("--expected-state-hash"))
        );
    }

    #[test]
    fn rejects_recovery_check_without_rejoin_attempt() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue("--rejoin-attempt"))
        );
    }

    #[test]
    fn rejects_daemon_without_max_ticks() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "25".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"))
        );
    }

    #[test]
    fn rejects_daemon_without_tick_interval() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "3".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue(
                "--daemon-tick-interval-ms"
            ))
        );
    }

    #[test]
    fn rejects_kolme_live_without_base_url() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue("--kolme-live-base-url"))
        );
    }

    #[test]
    fn rejects_kolme_live_without_provider_hint() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue(
                "--kolme-live-provider-hint"
            ))
        );
    }

    #[test]
    fn rejects_kolme_live_without_signing_profile() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue(
                "--kolme-live-signing-profile"
            ))
        );
    }

    #[test]
    fn rejects_kolme_live_strict_signer_contracts_without_signer_profile_selector() {
        // Regression: #2246
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "env-local".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue(
                "--kolme-live-signer-profile"
            ))
        );
    }

    #[test]
    fn rejects_kolme_live_strict_signer_contracts_without_key_source() {
        // Regression: #2246
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::MissingArgumentValue(
                "--kolme-live-signer-key-source"
            ))
        );
    }

    #[test]
    fn rejects_kolme_live_strict_signer_contracts_with_unsupported_key_source() {
        // Regression: #2246
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "managed-external".to_owned(),
        ];
        assert!(
            matches!(
                parse_args(args),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("--kolme-live-signer-key-source has unsupported key source")
            ),
            "strict signer contracts must reject unsupported key source markers"
        );
    }

    #[test]
    fn parses_kolme_live_strict_signer_contracts_with_explicit_declarations() {
        // Regression: #2246
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "env-local".to_owned(),
        ];
        parse_args(args).expect("strict signer contract declarations should parse");
    }

    #[test]
    fn rejects_kolme_live_strict_signer_contracts_with_empty_signer_profile_selector() {
        // Regression: #2247
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            " ".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "env-local".to_owned(),
        ];
        assert!(
            matches!(
                parse_args(args),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("--kolme-live-signer-profile must not be empty")
            ),
            "strict signer contracts must reject empty signer profile selector"
        );
    }

    #[test]
    fn rejects_kolme_live_strict_signer_contracts_with_empty_key_source() {
        // Regression: #2247
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            " ".to_owned(),
        ];
        assert!(
            matches!(
                parse_args(args),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("--kolme-live-signer-key-source must not be empty")
            ),
            "strict signer contracts must reject empty key-source declaration"
        );
    }

    #[test]
    fn regression_kolme_live_strict_signer_contracts_reject_profile_selector_env_mismatch() {
        // Regression: #2247
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _primary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );

        assert!(
            matches!(
                build_kolme_live_signer_adapter(Some("ops-primary"), Some("env-local")),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("strict signer profile mismatch")
            ),
            "strict signer contracts must reject selector/env profile mismatch"
        );
    }

    #[test]
    fn integration_runtime_kolme_live_strict_signer_contracts_fail_closed_before_network_on_selector_env_mismatch(
    ) {
        // Regression: #2247
        let _lock = signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _primary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let (base_url, requests) = spawn_kolme_live_mock_server(vec![
            MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
            MockHttpReply::ok(
                r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
            ),
        ]);
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            base_url,
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "env-local".to_owned(),
        ];
        let parsed = parse_args(args).expect("strict kolme-live args should parse");
        assert!(
            matches!(
                execute(parsed),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("strict signer profile mismatch")
            ),
            "runtime must fail closed before network submit when strict signer selector conflicts with env marker"
        );
        let recorded_requests = requests.lock().expect("request mutex should lock");
        assert_eq!(
            recorded_requests.len(),
            0,
            "strict selector/env mismatch should fail before any live network request"
        );
    }

    #[test]
    fn rejects_kolme_live_with_invalid_signing_profile() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "synthetic-signing-profile".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidKolmeLiveSigningProfile(
                "synthetic-signing-profile".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_kolme_live_with_in_memory_provider_hint_marker() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "InMemoryKolmeRuntimeCommitClient".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidKolmeLiveProviderHint(
                "InMemoryKolmeRuntimeCommitClient".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_unknown_argument() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "approver".to_owned(),
            "--unknown".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::UnknownArgument("--unknown".to_owned()))
        );
    }

    #[test]
    fn rejects_invalid_output_mode() {
        // Regression: #307
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "approver".to_owned(),
            "--output".to_owned(),
            "yaml".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidOutputMode("yaml".to_owned()))
        );
    }

    #[test]
    fn rejects_invalid_profile_value() {
        // Regression: #310
        let args = vec![
            "kamn-node".to_owned(),
            "--profile".to_owned(),
            "local-unknown".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidNodeProfile("local-unknown".to_owned()))
        );
    }

    #[test]
    fn rejects_invalid_runtime_mode() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "service".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidRuntimeMode("service".to_owned()))
        );
    }

    #[test]
    fn rejects_malformed_proposal_argument() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "planning".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-1".to_owned(),
            "--proposal".to_owned(),
            "tx-1|did:kamn:agent:aaa|state-1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidProposalArgument(
                "tx-1|did:kamn:agent:aaa|state-1".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_malformed_rejoin_attempt_argument() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-42".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidRejoinAttemptArgument(
                "node-a|42|state-42".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_invalid_expected_state_version_argument() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "0".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-42|resume-1".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidExpectedStateVersion("0".to_owned()))
        );
    }

    #[test]
    fn rejects_invalid_daemon_control_argument() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "abc".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "25".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidDaemonControlArgument("abc".to_owned()))
        );
    }

    #[test]
    fn rejects_invalid_daemon_lifecycle_event_argument() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "3".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "25".to_owned(),
            "--daemon-peer-id".to_owned(),
            "peer-alpha".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "resume".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidDaemonLifecycleEvent(
                "resume".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_invalid_diagnostics_mode() {
        // Regression: #313
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--diagnostics".to_owned(),
            "extended".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidDiagnosticsMode("extended".to_owned()))
        );
    }

    #[test]
    fn regression_runtime_planning_rejects_duplicate_candidate_ids() {
        // Regression: #335
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "planning".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-1".to_owned(),
            "--proposal".to_owned(),
            "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
            "--proposal".to_owned(),
            "tx-1|did:kamn:agent:bbb|2|state-1".to_owned(),
        ];
        let parsed = parse_args(args).expect("planning args should parse");
        assert_eq!(
            execute(parsed),
            Err(ConfigError::RuntimePlanner(
                "duplicate proposal candidate id: tx-1".to_owned()
            ))
        );
    }

    #[test]
    fn regression_runtime_planning_rejects_stale_state_hash() {
        // Regression: #335
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "planning".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-1".to_owned(),
            "--proposal".to_owned(),
            "tx-1|did:kamn:agent:aaa|1|state-2".to_owned(),
        ];
        let parsed = parse_args(args).expect("planning args should parse");
        assert_eq!(
            execute(parsed),
            Err(ConfigError::RuntimePlanner(
                "proposal candidate state hash mismatch: expected state-1, found state-2"
                    .to_owned()
            ))
        );
    }

    #[test]
    fn regression_runtime_recovery_rejects_replay_resume_token() {
        // Regression: #336
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-42|resume-1".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-42|resume-1".to_owned(),
        ];
        let parsed = parse_args(args).expect("recovery-check args should parse");
        assert_eq!(
            execute(parsed),
            Err(ConfigError::RuntimeRecovery(
                "rejoin resume token replayed: resume-1".to_owned()
            ))
        );
    }

    #[test]
    fn regression_runtime_recovery_rejects_state_version_mismatch() {
        // Regression: #336
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|43|state-43|resume-1".to_owned(),
        ];
        let parsed = parse_args(args).expect("recovery-check args should parse");
        assert_eq!(
            execute(parsed),
            Err(ConfigError::RuntimeRecovery(
                "rejoin state version mismatch: expected 42, found 43".to_owned()
            ))
        );
    }

    #[test]
    fn regression_runtime_recovery_rejects_state_hash_mismatch() {
        // Regression: #336
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "recovery-check".to_owned(),
            "--expected-state-version".to_owned(),
            "42".to_owned(),
            "--expected-state-hash".to_owned(),
            "state-42".to_owned(),
            "--rejoin-attempt".to_owned(),
            "node-a|42|state-41|resume-1".to_owned(),
        ];
        let parsed = parse_args(args).expect("recovery-check args should parse");
        assert_eq!(
            execute(parsed),
            Err(ConfigError::RuntimeRecovery(
                "rejoin state hash mismatch: expected state-42, found state-41".to_owned()
            ))
        );
    }

    #[test]
    fn regression_runtime_daemon_rejects_zero_tick_budget() {
        // Regression: #348
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "0".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "25".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidDaemonControlArgument("0".to_owned()))
        );
    }

    #[test]
    fn regression_runtime_daemon_rejects_invalid_lifecycle_transition() {
        // Regression: #349
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "3".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "25".to_owned(),
            "--daemon-peer-id".to_owned(),
            "peer-alpha".to_owned(),
            "--daemon-lifecycle-event".to_owned(),
            "handshake-succeeded".to_owned(),
        ];
        let parsed = parse_args(args).expect("daemon args should parse");
        assert_eq!(
            execute(parsed),
            Err(ConfigError::RuntimeDaemonLifecycle(
                "invalid peer lifecycle transition from Disconnected via HandshakeSucceeded"
                    .to_owned()
            ))
        );
    }
}
