use std::env;
use std::process::ExitCode;

use kamn_core::{bootstrap, ConfigError, NodeConfig, NodeRole};

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeCli {
    role: NodeRole,
    chain_id: String,
    chain_version: String,
    storage_dir: String,
    enable_gossip: bool,
}

fn parse_args<I>(args: I) -> Result<NodeCli, ConfigError>
where
    I: IntoIterator<Item = String>,
{
    let mut role: Option<NodeRole> = None;
    let mut chain_id = String::from("kamn-devnet");
    let mut chain_version = String::from("v0.1.0");
    let mut storage_dir = String::from("./data");
    let mut enable_gossip = true;

    let mut iter = args.into_iter();
    let _bin = iter.next();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--role" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--role"))?;
                role = Some(value.parse::<NodeRole>()?);
            }
            "--chain-id" => {
                chain_id = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--chain-id"))?;
            }
            "--chain-version" => {
                chain_version = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--chain-version"))?;
            }
            "--storage-dir" => {
                storage_dir = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--storage-dir"))?;
            }
            "--disable-gossip" => {
                enable_gossip = false;
            }
            unknown => {
                return Err(ConfigError::UnknownArgument(unknown.to_owned()));
            }
        }
    }

    let role = role.ok_or(ConfigError::MissingArgumentValue("--role"))?;

    Ok(NodeCli {
        role,
        chain_id,
        chain_version,
        storage_dir,
        enable_gossip,
    })
}

fn run() -> Result<(), ConfigError> {
    let cli = parse_args(env::args())?;

    let config = NodeConfig {
        chain_id: cli.chain_id,
        chain_version: cli.chain_version,
        role: cli.role,
        storage_dir: cli.storage_dir,
        enable_gossip: cli.enable_gossip,
    };

    let plan = bootstrap(config)?;

    println!(
        "KAMN node bootstrap\n  role: {}\n  chain: {} ({})\n  storage: {}\n  gossip: {}\n  components: {}",
        plan.config.role.as_str(),
        plan.config.chain_id,
        plan.config.chain_version,
        plan.config.storage_dir,
        if plan.config.enable_gossip {
            "enabled"
        } else {
            "disabled"
        },
        plan.wiring.all_components().join(", ")
    );

    Ok(())
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
    use super::parse_args;
    use kamn_core::{ConfigError, NodeRole};

    #[test]
    fn parses_required_role_and_defaults() {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.role, NodeRole::Processor);
        assert_eq!(parsed.chain_id, "kamn-devnet");
        assert_eq!(parsed.chain_version, "v0.1.0");
        assert_eq!(parsed.storage_dir, "./data");
        assert!(parsed.enable_gossip);
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
}
