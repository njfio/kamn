use crate::report_render::render_bootstrap_report;
use crate::{ConfigError, NodeBootstrapReport, OutputMode};
use std::io::Write;

pub(crate) fn emit_bootstrap_report_output(
    report: &NodeBootstrapReport,
    output_mode: OutputMode,
) -> Result<(), ConfigError> {
    let rendered = render_bootstrap_report(report, output_mode);
    write_stdout_line(rendered.as_str())
}

pub(crate) fn write_stderr_line(line: &str) -> Result<(), ConfigError> {
    write_line_to_stream(line, &mut std::io::stderr())
}

fn write_stdout_line(line: &str) -> Result<(), ConfigError> {
    write_line_to_stream(line, &mut std::io::stdout())
}

fn write_line_to_stream(line: &str, stream: &mut impl Write) -> Result<(), ConfigError> {
    stream
        .write_all(line.as_bytes())
        .map_err(map_stream_write_error)?;
    stream.write_all(b"\n").map_err(map_stream_write_error)?;
    stream.flush().map_err(map_stream_write_error)?;
    Ok(())
}

fn map_stream_write_error(error: std::io::Error) -> ConfigError {
    ConfigError::RuntimeDaemonLifecycle(error.to_string())
}
