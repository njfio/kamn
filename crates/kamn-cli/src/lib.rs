#![warn(missing_docs)]
//! CLI scaffold for KAMN agent operations.

/// Command modules.
pub mod commands;

#[path = "cli_args.rs"]
mod cli_args;
#[path = "cli_dispatch.rs"]
mod cli_dispatch;
#[path = "cli_models.rs"]
mod cli_models;
#[path = "cli_parse_mapping.rs"]
mod cli_parse_mapping;

use cli_args::{is_help_request_impl, parse_cli_args_impl, render_help_text_impl};
use cli_dispatch::dispatch_impl;
pub use cli_models::{CommandKind, CommandOutput, OutputFormat, ParsedCliArgs};
use cli_parse_mapping::{parse_command_kind, parse_output_format};

impl OutputFormat {
    fn parse(raw: &str) -> Result<Self, String> {
        parse_output_format(raw)
    }
}

impl CommandKind {
    fn parse(raw: &str) -> Result<Self, String> {
        parse_command_kind(raw)
    }
}

/// Returns whether CLI arguments include the help flag.
pub fn is_help_request<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    is_help_request_impl(args)
}

/// Parses CLI arguments for phase-2 command surface contracts.
pub fn parse_cli_args<I, S>(args: I) -> Result<ParsedCliArgs, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    parse_cli_args_impl(args)
}

/// Dispatches one parsed command to the corresponding phase-2 command module.
pub fn dispatch(parsed: &ParsedCliArgs) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    dispatch_impl(parsed)
}

/// Renders deterministic help text for CLI usage output.
pub fn render_help_text() -> String {
    render_help_text_impl()
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
