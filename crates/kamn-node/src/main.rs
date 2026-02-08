use std::env;
use std::process::ExitCode;

use kamn_core::{
    bootstrap, BootstrapPlan, ConfigError, DeterministicProposalPlanner, NodeConfig, NodeRole,
    ProposalCandidate, SyncMode,
};

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

    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "bootstrap" => Ok(Self::bootstrap()),
            "planning" => Ok(Self::planning()),
            other => Err(ConfigError::InvalidRuntimeMode(other.to_owned())),
        }
    }

    fn as_str(self) -> &'static str {
        match self.kind {
            RuntimeModeKind::Bootstrap => "bootstrap",
            RuntimeModeKind::Planning => "planning",
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
    expected_state_hash: Option<String>,
    proposals: Vec<ProposalCandidate>,
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
struct NodeBootstrapReport {
    runtime_mode: String,
    diagnostics_mode: String,
    component_count: usize,
    planning_expected_state_hash: Option<String>,
    planning_candidate_count: Option<usize>,
    planning_scheduled_candidate_ids: Option<Vec<String>>,
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
    let mut expected_state_hash: Option<String> = None;
    let mut proposals: Vec<ProposalCandidate> = Vec::new();
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

    Ok(NodeCli {
        profile,
        role,
        chain_id,
        chain_version,
        storage_dir,
        enable_gossip,
        sync_mode,
        runtime_mode,
        expected_state_hash,
        proposals,
        output_mode,
        diagnostics_mode,
    })
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

fn run() -> Result<(), ConfigError> {
    let cli = parse_args(env::args())?;
    let output_mode = cli.output_mode;
    let report = execute(cli)?;
    println!("{}", render_bootstrap_report(&report, output_mode));

    Ok(())
}

fn execute(cli: NodeCli) -> Result<NodeBootstrapReport, ConfigError> {
    let config = NodeConfig {
        chain_id: cli.chain_id.clone(),
        chain_version: cli.chain_version.clone(),
        role: cli.role,
        storage_dir: cli.storage_dir.clone(),
        enable_gossip: cli.enable_gossip,
        sync_mode: cli.sync_mode,
    };

    let plan = bootstrap(config)?;
    let planning = match cli.runtime_mode.kind {
        RuntimeModeKind::Bootstrap => None,
        RuntimeModeKind::Planning => {
            let expected_state_hash = cli
                .expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let planner = DeterministicProposalPlanner::new(&expected_state_hash);
            let proposal_plan = planner
                .plan(cli.proposals)
                .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))?;
            Some(PlanningExecution {
                expected_state_hash,
                candidate_count: proposal_plan.ordered_candidates().len(),
                scheduled_candidate_ids: proposal_plan.ordered_candidate_ids(),
            })
        }
    };
    let report = build_bootstrap_report(
        &plan,
        cli.profile,
        cli.diagnostics_mode,
        cli.runtime_mode,
        planning,
    );

    Ok(report)
}

fn build_bootstrap_report(
    plan: &BootstrapPlan,
    profile: Option<LocalProfile>,
    diagnostics_mode: DiagnosticsMode,
    runtime_mode: RuntimeMode,
    planning: Option<PlanningExecution>,
) -> NodeBootstrapReport {
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
    NodeBootstrapReport {
        runtime_mode: runtime_mode.as_str().to_owned(),
        diagnostics_mode: diagnostics_mode.as_str().to_owned(),
        component_count: components.len(),
        planning_expected_state_hash,
        planning_candidate_count,
        planning_scheduled_candidate_ids,
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
    format!(
        "KAMN node bootstrap\n  runtime_mode: {}\n  diagnostics_mode: {}\n  profile: {}\n  role: {}\n  chain: {} ({})\n  storage: {}\n  gossip: {}\n  sync_mode: {}\n  sync_startup: {}\n  sync_recovery: {}\n  state_version: {}\n  pending_migrations: {}\n  component_count: {}\n  planning_expected_state_hash: {}\n  planning_candidate_count: {}\n  planning_scheduled_candidate_ids: {}\n  components: {}",
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
    let components = report
        .components
        .iter()
        .map(|component| format!("\"{}\"", json_escape(component)))
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\"runtime_mode\":\"{}\",\"diagnostics_mode\":\"{}\",\"profile\":{},\"role\":\"{}\",\"chain_id\":\"{}\",\"chain_version\":\"{}\",\"storage_dir\":\"{}\",\"gossip_enabled\":{},\"sync_mode\":\"{}\",\"sync_startup\":\"{}\",\"sync_recovery\":\"{}\",\"state_version\":{},\"pending_migrations\":{},\"component_count\":{},\"planning_expected_state_hash\":{},\"planning_candidate_count\":{},\"planning_scheduled_candidate_ids\":{},\"components\":[{}]}}",
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
        build_bootstrap_report, execute, parse_args, render_bootstrap_report, DiagnosticsMode,
        LocalProfile, NodeBootstrapReport, OutputMode, RuntimeMode,
    };
    use kamn_core::{bootstrap, ConfigError, NodeConfig, NodeRole, SyncMode};

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
        assert!(parsed.proposals.is_empty());
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
            None,
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
            None,
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
            None,
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
            "daemon".to_owned(),
        ];
        assert_eq!(
            parse_args(args),
            Err(ConfigError::InvalidRuntimeMode("daemon".to_owned()))
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
}
