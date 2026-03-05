mod command_activation_harness;

use command_activation_harness::{
    parsed, parsed_json, reserve_loopback_addr, run_cli_contract_server, wait_for_server_ready,
};
use kamn_cli::{dispatch, CommandKind};
use std::thread;

#[test]
fn spec_c01_cli_health_command_executes_supported_path() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 1));
    wait_for_server_ready();

    let output = dispatch(&parsed(
        CommandKind::Health,
        format!("http://{bind_addr}").as_str(),
        &[],
    ))
    .expect("health command should succeed");
    assert!(
        output.text.contains("status=ok"),
        "health output should include status marker: {output:?}"
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c11_cli_help_command_renders_usage_surface() {
    let text_output = dispatch(&parsed(CommandKind::Help, "http://localhost:18080", &[]))
        .expect("help command should succeed");
    assert!(
        text_output.text.contains("usage=kamn-cli <command>"),
        "help text should include usage: {text_output:?}"
    );
    assert!(
        text_output.text.contains("commands=register"),
        "help text should include command inventory: {text_output:?}"
    );
    assert!(
        text_output
            .text
            .contains("flags=--help, -h, --format, --endpoint"),
        "help text should include known flags: {text_output:?}"
    );

    let json_output = dispatch(&parsed_json(
        CommandKind::Help,
        "http://localhost:18080",
        &[],
    ))
    .expect("help command should succeed in json mode");
    assert!(
        json_output
            .json
            .contains("\"flags\":[\"--help\",\"-h\",\"--format\",\"--endpoint\"]"),
        "help json should expose deterministic flag set: {json_output:?}"
    );
}

#[test]
fn spec_c06_cli_json_output_contract_renders_structured_health_projection() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 1));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");
    let output = dispatch(&parsed_json(CommandKind::Health, endpoint.as_str(), &[]))
        .expect("health command should succeed");
    assert!(
        output.json.starts_with('{'),
        "json output should start with object marker: {}",
        output.json
    );
    assert!(
        output.json.contains("\"status\":\"ok\""),
        "json output should include status projection: {}",
        output.json
    );
    assert!(
        !output.json.contains("\"result\":"),
        "json output should not wrap result as escaped blob: {}",
        output.json
    );
    assert!(
        output.text.contains("status=ok"),
        "text projection should still include key=value markers: {}",
        output.text
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}
