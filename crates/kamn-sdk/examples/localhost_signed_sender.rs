use kamn_sdk::AgentDid;
use std::env;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct SenderConfig {
    addr: String,
    from: String,
    to: String,
    session_id: String,
    nonce: u64,
    state_hash: String,
    body: String,
}

fn parse_args() -> Result<SenderConfig, String> {
    let mut config = SenderConfig {
        addr: "127.0.0.1:17879".to_owned(),
        from: "kamn:did:agent:sender-1".to_owned(),
        to: "kamn:did:agent:listener-1".to_owned(),
        session_id: "session:localhost-demo:v1".to_owned(),
        nonce: 1,
        state_hash: "state:localhost-demo".to_owned(),
        body: "hello-from-localhost-demo".to_owned(),
    };

    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for argument {flag}"))?;
        match flag.as_str() {
            "--addr" => config.addr = value,
            "--from" => config.from = value,
            "--to" => config.to = value,
            "--session-id" => config.session_id = value,
            "--nonce" => {
                config.nonce = value
                    .parse::<u64>()
                    .map_err(|_| "nonce must be an unsigned integer".to_owned())?;
            }
            "--state-hash" => config.state_hash = value,
            "--body" => config.body = value,
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    AgentDid::parse(config.from.clone()).map_err(|error| format!("invalid --from did: {error}"))?;
    AgentDid::parse(config.to.clone()).map_err(|error| format!("invalid --to did: {error}"))?;

    if config.body.trim().is_empty() {
        return Err("body must not be empty".to_owned());
    }
    if config.state_hash.trim().is_empty() {
        return Err("state hash must not be empty".to_owned());
    }
    if config.session_id.trim().is_empty() {
        return Err("session id must not be empty".to_owned());
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

fn connect_with_retry(addr: &str) -> Result<TcpStream, String> {
    let mut last_error = String::new();
    for _attempt in 0..20 {
        match TcpStream::connect(addr) {
            Ok(stream) => return Ok(stream),
            Err(error) => {
                last_error = error.to_string();
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    Err(format!(
        "failed to connect to {addr} after retries: {last_error}"
    ))
}

fn sanitize(value: &str) -> String {
    value.replace('\n', " ")
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let signature = signature_for_fields(
        &config.from,
        &config.session_id,
        config.nonce,
        &config.state_hash,
        &config.body,
    );
    let wire_payload = format!(
        "from={}\nto={}\nsession_id={}\nnonce={}\nstate_hash={}\nbody={}\nsignature={}\n",
        config.from,
        config.to,
        config.session_id,
        config.nonce,
        config.state_hash,
        config.body,
        signature
    );

    let mut stream = connect_with_retry(&config.addr)?;
    stream
        .write_all(wire_payload.as_bytes())
        .map_err(|error| format!("failed to write payload to listener: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("failed to flush payload: {error}"))?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|error| format!("failed to shutdown socket write-half: {error}"))?;

    println!("status=ok");
    println!("addr={}", config.addr);
    println!("from={}", config.from);
    println!("to={}", config.to);
    println!("session_id={}", config.session_id);
    println!("nonce={}", config.nonce);
    println!("state_hash={}", config.state_hash);
    println!("body={}", config.body);
    println!("signature={signature}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        println!("status=error");
        println!("error={}", sanitize(&error));
        std::process::exit(1);
    }
}
