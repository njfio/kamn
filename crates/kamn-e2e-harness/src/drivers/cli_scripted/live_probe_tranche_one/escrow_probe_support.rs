use super::super::{
    parse_text_output_field, run_cli_command_capture_stdout_with_agent_name,
    validate_live_s05_release_escrow_response, DEFAULT_S05_AGENT_NAME,
    DEFAULT_S05_FUND_ESCROW_PAYLOAD,
};
use super::{cli_binary, endpoint, env_payload, validate_non_empty};

pub(super) fn run_live_s05_cli_escrow_settlement_probe() -> Result<(), String> {
    let agent_name = super::super::env_var_or_default("KAMN_AGENT_NAME", DEFAULT_S05_AGENT_NAME);
    let escrow_id = fund_escrow(agent_name.as_str())?;
    release_escrow(agent_name.as_str(), escrow_id.as_str())
}

fn fund_escrow(agent_name: &str) -> Result<String, String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "fund-escrow",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            env_payload(
                "KAMN_E2E_S05_FUND_ESCROW_PAYLOAD",
                DEFAULT_S05_FUND_ESCROW_PAYLOAD,
            )
            .as_str(),
        ],
        "cli live s05 fund-escrow",
        format!("{agent_name}-fund").as_str(),
    )?;
    let escrow_id = require_field(output.as_str(), "escrow_id", "cli live s05 fund-escrow")?;
    validate_non_empty(
        escrow_id,
        "cli live s05 fund-escrow returned empty escrow_id",
    )?;
    validate_non_empty(
        require_field(output.as_str(), "state", "cli live s05 fund-escrow")?,
        "cli live s05 fund-escrow returned empty state",
    )?;
    Ok(escrow_id.to_owned())
}

fn release_escrow(agent_name: &str, escrow_id: &str) -> Result<(), String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "release-escrow",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            escrow_id,
        ],
        "cli live s05 release-escrow",
        format!("{agent_name}-release").as_str(),
    )?;
    validate_live_s05_release_escrow_response(
        escrow_id,
        require_field(output.as_str(), "escrow_id", "cli live s05 release-escrow")?,
        require_field(output.as_str(), "state", "cli live s05 release-escrow")?,
        "cli live s05 release-escrow",
    )
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}
