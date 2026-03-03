use kamn_sdk::{
    service_public_key_for_private_key, AgentDid, SdkError, TcpTransportAdapter, TcpTransportConfig,
};
use std::env;

const DEFAULT_SIGNER_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
const DEFAULT_EXPECTED_FROM_METHOD_ID: &str = "sender-1";
const DEFAULT_EXPECTED_TO_DID: &str = "kamn:did:agent:listener-1";

#[derive(Debug, Clone)]
struct ListenerConfig {
    addr: String,
    expected_from: AgentDid,
    expected_to: AgentDid,
    expected_state_hash: String,
}

fn default_expected_from() -> Result<AgentDid, SdkError> {
    let signer_public_key = service_public_key_for_private_key(DEFAULT_SIGNER_PRIVATE_KEY_HEX)
        .map_err(|_| SdkError::InvalidInput {
            field: "expected_from",
            reason: "failed to derive default key-bound sender did",
        })?;
    AgentDid::with_public_key_hex_binding(
        DEFAULT_EXPECTED_FROM_METHOD_ID,
        signer_public_key.as_str(),
    )
    .map_err(|_| SdkError::InvalidInput {
        field: "expected_from",
        reason: "failed to derive default key-bound sender did",
    })
}

fn parse_args() -> Result<ListenerConfig, SdkError> {
    let mut addr = "127.0.0.1:17881".to_owned();
    let mut expected_from = default_expected_from()?;
    let mut expected_to = AgentDid::parse(DEFAULT_EXPECTED_TO_DID)?;
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
    println!("signer_public_key={}", received.envelope.signer_public_key);
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
