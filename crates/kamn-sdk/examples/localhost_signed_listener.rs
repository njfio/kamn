use kamn_sdk::AgentDid;
use std::env;
use std::io::Read;
use std::net::TcpListener;

#[derive(Debug, Clone)]
struct ListenerConfig {
    addr: String,
    expected_from: String,
    expected_to: String,
    expected_state_hash: String,
}

#[derive(Debug, Clone)]
struct WireMessage {
    from: String,
    to: String,
    nonce: u64,
    state_hash: String,
    body: String,
    signature: String,
}

fn parse_args() -> Result<ListenerConfig, String> {
    let mut config = ListenerConfig {
        addr: "127.0.0.1:17879".to_owned(),
        expected_from: "kamn:did:agent:sender-1".to_owned(),
        expected_to: "kamn:did:agent:listener-1".to_owned(),
        expected_state_hash: "state:localhost-demo".to_owned(),
    };

    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for argument {flag}"))?;
        match flag.as_str() {
            "--addr" => config.addr = value,
            "--expected-from" => config.expected_from = value,
            "--expected-to" => config.expected_to = value,
            "--state-hash" => config.expected_state_hash = value,
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    AgentDid::parse(config.expected_from.clone())
        .map_err(|error| format!("invalid --expected-from did: {error}"))?;
    AgentDid::parse(config.expected_to.clone())
        .map_err(|error| format!("invalid --expected-to did: {error}"))?;

    if config.expected_state_hash.trim().is_empty() {
        return Err("state hash must not be empty".to_owned());
    }
    Ok(config)
}

fn signature_for_fields(from: &str, nonce: u64, state_hash: &str, body: &str) -> String {
    format!(
        "sig:ed25519:baseline-v1:{from}:{nonce}:{state_hash}:{}",
        body.len()
    )
}

fn parse_wire_message(payload: &str) -> Result<WireMessage, String> {
    let mut from: Option<String> = None;
    let mut to: Option<String> = None;
    let mut nonce: Option<u64> = None;
    let mut state_hash: Option<String> = None;
    let mut body: Option<String> = None;
    let mut signature: Option<String> = None;

    for line in payload.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "from" => from = Some(value.to_owned()),
            "to" => to = Some(value.to_owned()),
            "nonce" => {
                let parsed = value
                    .parse::<u64>()
                    .map_err(|_| "nonce must be an unsigned integer".to_owned())?;
                nonce = Some(parsed);
            }
            "state_hash" => state_hash = Some(value.to_owned()),
            "body" => body = Some(value.to_owned()),
            "signature" => signature = Some(value.to_owned()),
            _ => {}
        }
    }

    Ok(WireMessage {
        from: from.ok_or_else(|| "wire message missing from".to_owned())?,
        to: to.ok_or_else(|| "wire message missing to".to_owned())?,
        nonce: nonce.ok_or_else(|| "wire message missing nonce".to_owned())?,
        state_hash: state_hash.ok_or_else(|| "wire message missing state_hash".to_owned())?,
        body: body.ok_or_else(|| "wire message missing body".to_owned())?,
        signature: signature.ok_or_else(|| "wire message missing signature".to_owned())?,
    })
}

fn sanitize(value: &str) -> String {
    value.replace('\n', " ")
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let listener = TcpListener::bind(&config.addr)
        .map_err(|error| format!("failed to bind listener on {}: {error}", config.addr))?;
    println!("status=listening");
    println!("addr={}", config.addr);

    let (mut stream, peer_addr) = listener
        .accept()
        .map_err(|error| format!("failed to accept inbound connection: {error}"))?;
    let mut payload = String::new();
    stream
        .read_to_string(&mut payload)
        .map_err(|error| format!("failed to read inbound payload: {error}"))?;

    let wire = parse_wire_message(&payload)?;
    AgentDid::parse(wire.from.clone()).map_err(|error| format!("invalid sender did: {error}"))?;
    AgentDid::parse(wire.to.clone())
        .map_err(|error| format!("invalid recipient did in payload: {error}"))?;

    if wire.from != config.expected_from {
        return Err(format!(
            "unexpected sender did: found {}, expected {}",
            wire.from, config.expected_from
        ));
    }
    if wire.to != config.expected_to {
        return Err(format!(
            "unexpected recipient did: found {}, expected {}",
            wire.to, config.expected_to
        ));
    }
    if wire.state_hash != config.expected_state_hash {
        return Err(format!(
            "unexpected state hash: found {}, expected {}",
            wire.state_hash, config.expected_state_hash
        ));
    }

    let expected_signature =
        signature_for_fields(&wire.from, wire.nonce, &wire.state_hash, &wire.body);
    if wire.signature != expected_signature {
        return Err("signature verification failed".to_owned());
    }

    println!("status=ok");
    println!("verified=true");
    println!("peer_addr={peer_addr}");
    println!("from={}", wire.from);
    println!("to={}", wire.to);
    println!("nonce={}", wire.nonce);
    println!("state_hash={}", wire.state_hash);
    println!("body={}", wire.body);
    println!("signature={}", wire.signature);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        println!("status=error");
        println!("error={}", sanitize(&error));
        std::process::exit(1);
    }
}
