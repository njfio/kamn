use kamn_sdk::AgentDid;
use std::env;
use std::fs;
use std::io::Read;
use std::net::TcpListener;
use std::path::Path;

#[derive(Debug, Clone)]
struct ListenerConfig {
    addr: String,
    expected_from: String,
    expected_to: String,
    expected_session_id: String,
    expected_state_hash: String,
    nonce_state_file: Option<String>,
}

#[derive(Debug, Clone)]
struct WireMessage {
    from: String,
    to: String,
    session_id: String,
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
        expected_session_id: "session:localhost-demo:v1".to_owned(),
        expected_state_hash: "state:localhost-demo".to_owned(),
        nonce_state_file: None,
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
            "--expected-session-id" => config.expected_session_id = value,
            "--state-hash" => config.expected_state_hash = value,
            "--nonce-state-file" => config.nonce_state_file = Some(value),
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
    if config.expected_session_id.trim().is_empty() {
        return Err("expected session id must not be empty".to_owned());
    }
    if let Some(path) = config.nonce_state_file.as_ref() {
        if path.trim().is_empty() {
            return Err("nonce state file path must not be empty".to_owned());
        }
    }
    Ok(config)
}

fn signature_for_fields(
    from: &str,
    session_id: &str,
    nonce: u64,
    state_hash: &str,
    body: &str,
) -> String {
    format!(
        "sig:ed25519:baseline-v1:{from}:{session_id}:{nonce}:{state_hash}:{}",
        body.len()
    )
}

fn parse_wire_message(payload: &str) -> Result<WireMessage, String> {
    let mut from: Option<String> = None;
    let mut to: Option<String> = None;
    let mut session_id: Option<String> = None;
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
            "session_id" => session_id = Some(value.to_owned()),
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
        session_id: session_id.ok_or_else(|| "wire message missing session_id".to_owned())?,
        nonce: nonce.ok_or_else(|| "wire message missing nonce".to_owned())?,
        state_hash: state_hash.ok_or_else(|| "wire message missing state_hash".to_owned())?,
        body: body.ok_or_else(|| "wire message missing body".to_owned())?,
        signature: signature.ok_or_else(|| "wire message missing signature".to_owned())?,
    })
}

fn read_highest_nonce(path: &str) -> Result<Option<u64>, String> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read nonce state file {}: {error}",
            path.display()
        )
    })?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let nonce = trimmed
        .parse::<u64>()
        .map_err(|_| format!("invalid nonce state file {} contents", path.display()))?;
    Ok(Some(nonce))
}

fn write_highest_nonce(path: &str, nonce: u64) -> Result<(), String> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create nonce state directory {}: {error}",
                    parent.display()
                )
            })?;
        }
    }
    fs::write(path, format!("{nonce}\n")).map_err(|error| {
        format!(
            "failed to write nonce state file {}: {error}",
            path.display()
        )
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
    if wire.session_id != config.expected_session_id {
        return Err(format!(
            "unexpected session id: found {}, expected {}",
            wire.session_id, config.expected_session_id
        ));
    }
    if wire.state_hash != config.expected_state_hash {
        return Err(format!(
            "unexpected state hash: found {}, expected {}",
            wire.state_hash, config.expected_state_hash
        ));
    }

    let expected_signature = signature_for_fields(
        &wire.from,
        &wire.session_id,
        wire.nonce,
        &wire.state_hash,
        &wire.body,
    );
    if wire.signature != expected_signature {
        return Err("signature verification failed".to_owned());
    }
    if let Some(nonce_state_file) = config.nonce_state_file.as_deref() {
        if let Some(highest_nonce) = read_highest_nonce(nonce_state_file)? {
            if wire.nonce <= highest_nonce {
                return Err(format!(
                    "replay nonce detected: nonce {} <= highest {}",
                    wire.nonce, highest_nonce
                ));
            }
        }
        write_highest_nonce(nonce_state_file, wire.nonce)?;
    }

    println!("status=ok");
    println!("verified=true");
    println!("peer_addr={peer_addr}");
    println!("from={}", wire.from);
    println!("to={}", wire.to);
    println!("session_id={}", wire.session_id);
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
