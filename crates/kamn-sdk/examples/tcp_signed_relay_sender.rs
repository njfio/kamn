use kamn_sdk::{AgentDid, SdkError, TcpSignedEnvelope, TcpTransportAdapter, TcpTransportConfig};
use std::env;

const DEFAULT_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

#[derive(Debug, Clone)]
struct SenderConfig {
    addr: String,
    from: AgentDid,
    to: AgentDid,
    nonce: u64,
    state_hash: String,
    body: String,
    private_key_hex: String,
}

fn parse_args() -> Result<SenderConfig, SdkError> {
    let mut addr = "127.0.0.1:17881".to_owned();
    let mut from = AgentDid::parse("kamn:did:agent:sender-1")?;
    let mut to = AgentDid::parse("kamn:did:agent:listener-1")?;
    let mut nonce: u64 = 1;
    let mut state_hash = "state:tcp-relay-demo".to_owned();
    let mut body = "hello-from-tcp-relay-demo".to_owned();
    let mut private_key_hex = DEFAULT_PRIVATE_KEY_HEX.to_owned();

    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or(SdkError::InvalidInput {
            field: "cli",
            reason: "missing value for argument",
        })?;
        match flag.as_str() {
            "--addr" => addr = value,
            "--from" => from = AgentDid::parse(value)?,
            "--to" => to = AgentDid::parse(value)?,
            "--nonce" => {
                nonce = value.parse::<u64>().map_err(|_| SdkError::InvalidInput {
                    field: "nonce",
                    reason: "must be an unsigned integer",
                })?
            }
            "--state-hash" => state_hash = value,
            "--body" => body = value,
            "--private-key-hex" => private_key_hex = value,
            _ => {
                return Err(SdkError::InvalidInput {
                    field: "cli",
                    reason: "unknown argument",
                });
            }
        }
    }

    if state_hash.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "state_hash",
            reason: "must not be empty",
        });
    }
    if body.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "body",
            reason: "must not be empty",
        });
    }
    if private_key_hex.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "signer_private_key",
            reason: "must not be empty",
        });
    }

    Ok(SenderConfig {
        addr,
        from,
        to,
        nonce,
        state_hash,
        body,
        private_key_hex,
    })
}

fn sanitize(error: &SdkError) -> String {
    error.to_string().replace('\n', " ")
}

fn run() -> Result<(), SdkError> {
    let config = parse_args()?;
    let transport_config =
        TcpTransportConfig::new(config.addr.as_str())?.with_connect_retries(30)?;
    let adapter = TcpTransportAdapter::new(transport_config);

    let envelope = TcpSignedEnvelope::new(
        config.from,
        config.to,
        config.nonce,
        config.state_hash,
        config.body,
        config.private_key_hex.as_str(),
    )?;

    adapter.send(&envelope)?;

    println!("status=ok");
    println!("adapter=tcp");
    println!("addr={}", config.addr);
    println!("from={}", envelope.from);
    println!("to={}", envelope.to);
    println!("nonce={}", envelope.nonce);
    println!("state_hash={}", envelope.state_hash);
    println!("body={}", envelope.body);
    println!("signer_public_key={}", envelope.signer_public_key);
    println!("signature={}", envelope.signature);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        println!("status=error");
        println!("error={}", sanitize(&error));
        std::process::exit(1);
    }
}
