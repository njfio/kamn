mod command_activation_harness;

use command_activation_harness::{parsed, parsed_json, with_contract_server};
use kamn_cli::{dispatch, CommandKind};

#[test]
fn spec_c01_cli_health_command_executes_supported_path() {
    with_contract_server(1, |endpoint| {
        let output = dispatch(&parsed(CommandKind::Health, endpoint, &[]))
            .expect("health command should succeed");
        assert!(
            output.text.contains("status=ok"),
            "health output should include status marker: {output:?}"
        );
    });
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
    with_contract_server(1, |endpoint| {
        let output = dispatch(&parsed_json(CommandKind::Health, endpoint, &[]))
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
    });
}
