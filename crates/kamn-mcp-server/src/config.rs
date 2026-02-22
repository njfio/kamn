/// Parsed `kamn-mcp-server` configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerConfig {
    /// KAMN service endpoint URL.
    pub endpoint: String,
    /// Logical agent name.
    pub agent_name: String,
    /// Agent key-file path.
    pub key_file: String,
}

impl McpServerConfig {
    /// Parses command-line arguments into a deterministic server configuration.
    pub fn from_args<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut args = args
            .into_iter()
            .map(|value| value.as_ref().to_owned())
            .collect::<Vec<_>>();

        if !args.is_empty() && !args[0].starts_with("--") {
            args.remove(0);
        }

        let mut endpoint = std::env::var("KAMN_ENDPOINT").ok();
        let mut agent_name: Option<String> = None;
        let mut key_file: Option<String> = None;

        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            index += 1;

            let value = if index < args.len() {
                args[index].as_str()
            } else {
                ""
            };

            match flag {
                "--endpoint" => {
                    if value.is_empty() {
                        return Err("missing value for --endpoint".to_owned());
                    }
                    endpoint = Some(value.to_owned());
                    index += 1;
                }
                "--agent-name" => {
                    if value.is_empty() {
                        return Err("missing value for --agent-name".to_owned());
                    }
                    agent_name = Some(value.to_owned());
                    index += 1;
                }
                "--key-file" => {
                    if value.is_empty() {
                        return Err("missing value for --key-file".to_owned());
                    }
                    key_file = Some(value.to_owned());
                    index += 1;
                }
                other => {
                    return Err(format!("unknown flag: {other}"));
                }
            }
        }

        Ok(Self {
            endpoint: endpoint.ok_or_else(|| "endpoint not provided".to_owned())?,
            agent_name: agent_name.ok_or_else(|| "agent name not provided".to_owned())?,
            key_file: key_file.ok_or_else(|| "key file not provided".to_owned())?,
        })
    }
}
