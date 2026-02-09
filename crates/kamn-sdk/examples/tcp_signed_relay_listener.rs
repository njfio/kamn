use kamn_sdk::{AgentDid, SdkError, TcpTransportAdapter, TcpTransportConfig};
use std::env;

#[derive(Debug, Clone)]
struct ListenerConfig {
    addr: String,
    expected_from: AgentDid,
    expected_to: AgentDid,
    expected_state_hash: String,
}

fn parse_args() -> Result<ListenerConfig, SdkError> {
    let mut addr = "127.0.0.1:17881".to_owned();
    let mut expected_from = AgentDid::parse("kamn:did:agent:sender-1")?;
    let mut expected_to = AgentDid::parse("kamn:did:agent:listener-1")?;
    let mut expected_state_hash = "state:tcp-relay-demo".to_owned();

    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or(SdkError::InvalidInput {
            field: "cli",
            reason: "missing value for argument",
        })?;
        match flag.as_str() {
            "--addr" => addr = value,
            "--expected-from" => expected_from = AgentDid::parse(value)?,
            "--expected-to" => expected_to = AgentDid::parse(value)?,
            "--state-hash" => expected_state_hash = value,
            _ => {
                return Err(SdkError::InvalidInput {
                    field: "cli",
                    reason: "unknown argument",
                });
            }
        }
    }

    if expected_state_hash.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "state_hash",
            reason: "must not be empty",
        });
    }

    Ok(ListenerConfig {
        addr,
        expected_from,
        expected_to,
        expected_state_hash,
    })
}

fn sanitize(error: &SdkError) -> String {
    error.to_string().replace('\n', " ")
}

fn run() -> Result<(), SdkError> {
    let config = parse_args()?;
    let transport_config = TcpTransportConfig::new(config.addr.as_str())?;
    let adapter = TcpTransportAdapter::new(transport_config);

    println!("status=listening");
    println!("addr={}", config.addr);

    let received = adapter.listen_once()?;

    if received.envelope.from != config.expected_from {
        return Err(SdkError::InvalidInput {
            field: "from",
            reason: "unexpected sender did",
        });
    }
    if received.envelope.to != config.expected_to {
        return Err(SdkError::InvalidInput {
            field: "to",
            reason: "unexpected recipient did",
        });
    }
    if received.envelope.state_hash != config.expected_state_hash {
        return Err(SdkError::InvalidInput {
            field: "state_hash",
            reason: "unexpected state hash",
        });
    }

    println!("status=ok");
    println!("verified=true");
    println!("adapter=tcp");
    println!("peer_addr={}", received.peer_addr);
    println!("from={}", received.envelope.from);
    println!("to={}", received.envelope.to);
    println!("nonce={}", received.envelope.nonce);
    println!("state_hash={}", received.envelope.state_hash);
    println!("body={}", received.envelope.body);
    println!("signature={}", received.envelope.signature);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        println!("status=error");
        println!("error={}", sanitize(&error));
        std::process::exit(1);
    }
}
