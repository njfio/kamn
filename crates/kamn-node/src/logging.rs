use kamn_core::ConfigError;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const KAMN_NODE_LOG_LEVEL_ENV: &str = "KAMN_NODE_LOG_LEVEL";
pub(crate) const KAMN_NODE_LOG_FORMAT_ENV: &str = "KAMN_NODE_LOG_FORMAT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NodeLogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl NodeLogLevel {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeLogFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodeLogConfig {
    pub(crate) level: NodeLogLevel,
    pub(crate) format: NodeLogFormat,
}

impl NodeLogConfig {
    pub(crate) fn default_config() -> Self {
        Self {
            level: NodeLogLevel::Info,
            format: NodeLogFormat::Text,
        }
    }

    fn allows(self, event_level: NodeLogLevel) -> bool {
        event_level <= self.level
    }
}

pub(crate) fn resolve_log_config_from_env() -> Result<NodeLogConfig, ConfigError> {
    let level_value = read_env_var(KAMN_NODE_LOG_LEVEL_ENV)?;
    let format_value = read_env_var(KAMN_NODE_LOG_FORMAT_ENV)?;
    resolve_log_config_from_inputs(level_value.as_deref(), format_value.as_deref())
}

pub(crate) fn resolve_log_config_from_inputs(
    level: Option<&str>,
    format: Option<&str>,
) -> Result<NodeLogConfig, ConfigError> {
    let base = NodeLogConfig::default_config();
    let resolved_level = match level {
        Some(raw) => parse_node_log_level(raw.trim())?,
        None => base.level,
    };
    let resolved_format = match format {
        Some(raw) => parse_node_log_format(raw.trim())?,
        None => base.format,
    };
    Ok(NodeLogConfig {
        level: resolved_level,
        format: resolved_format,
    })
}

pub(crate) fn log_info(event: &str, fields: &[(&str, &str)]) -> Result<(), ConfigError> {
    emit_log_event(NodeLogLevel::Info, event, fields)
}

pub(crate) fn log_warn(event: &str, fields: &[(&str, &str)]) -> Result<(), ConfigError> {
    emit_log_event(NodeLogLevel::Warn, event, fields)
}

pub(crate) fn log_error(event: &str, fields: &[(&str, &str)]) -> Result<(), ConfigError> {
    emit_log_event(NodeLogLevel::Error, event, fields)
}

pub(crate) fn emit_log_event(
    level: NodeLogLevel,
    event: &str,
    fields: &[(&str, &str)],
) -> Result<(), ConfigError> {
    let config = resolve_log_config_from_env()?;
    if !config.allows(level) {
        return Ok(());
    }
    let line = render_log_event_line(config, level, event, fields);
    record_test_log_line(line.as_str());
    eprintln!("{line}");
    Ok(())
}

pub(crate) fn render_log_event_line(
    config: NodeLogConfig,
    level: NodeLogLevel,
    event: &str,
    fields: &[(&str, &str)],
) -> String {
    let timestamp_ms = current_unix_timestamp_ms();
    match config.format {
        NodeLogFormat::Text => render_text_log_event_line(timestamp_ms, level, event, fields),
        NodeLogFormat::Json => render_json_log_event_line(timestamp_ms, level, event, fields),
    }
}

fn read_env_var(name: &'static str) -> Result<Option<String>, ConfigError> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidLogConfig(format!(
            "{name} must be valid UTF-8"
        ))),
    }
}

fn parse_node_log_level(value: &str) -> Result<NodeLogLevel, ConfigError> {
    match value.to_ascii_lowercase().as_str() {
        "error" => Ok(NodeLogLevel::Error),
        "warn" => Ok(NodeLogLevel::Warn),
        "info" => Ok(NodeLogLevel::Info),
        "debug" => Ok(NodeLogLevel::Debug),
        "trace" => Ok(NodeLogLevel::Trace),
        _ => Err(ConfigError::InvalidLogConfig(format!(
            "{KAMN_NODE_LOG_LEVEL_ENV} must be one of: error,warn,info,debug,trace"
        ))),
    }
}

fn parse_node_log_format(value: &str) -> Result<NodeLogFormat, ConfigError> {
    match value.to_ascii_lowercase().as_str() {
        "text" => Ok(NodeLogFormat::Text),
        "json" => Ok(NodeLogFormat::Json),
        _ => Err(ConfigError::InvalidLogConfig(format!(
            "{KAMN_NODE_LOG_FORMAT_ENV} must be one of: text,json"
        ))),
    }
}

fn current_unix_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn render_text_log_event_line(
    timestamp_ms: u128,
    level: NodeLogLevel,
    event: &str,
    fields: &[(&str, &str)],
) -> String {
    let mut line = format!(
        "ts_unix_ms={timestamp_ms} level={} event={}",
        level.as_str(),
        render_text_field_value(event)
    );
    for (key, value) in fields {
        line.push(' ');
        line.push_str(key);
        line.push('=');
        line.push_str(render_text_field_value(value).as_str());
    }
    line
}

fn render_json_log_event_line(
    timestamp_ms: u128,
    level: NodeLogLevel,
    event: &str,
    fields: &[(&str, &str)],
) -> String {
    let mut line = format!(
        "{{\"ts_unix_ms\":{timestamp_ms},\"level\":\"{}\",\"event\":\"{}\"",
        level.as_str(),
        escape_json_string(event)
    );
    if fields.is_empty() {
        line.push('}');
        return line;
    }
    line.push_str(",\"fields\":{");
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            line.push(',');
        }
        line.push('"');
        line.push_str(escape_json_string(key).as_str());
        line.push_str("\":\"");
        line.push_str(escape_json_string(value).as_str());
        line.push('"');
    }
    line.push_str("}}");
    line
}

fn render_text_field_value(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_owned();
    }
    if value
        .bytes()
        .all(|byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b':' | b'/' ))
    {
        return value.to_owned();
    }
    format!("\"{}\"", escape_text_string(value))
}

fn escape_text_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_json_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
thread_local! {
    static TEST_LOG_CAPTURE: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn capture_test_logs<T, F>(operation: F) -> (T, Vec<String>)
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    TEST_LOG_CAPTURE.with(|capture| {
        *capture.borrow_mut() = Some(Vec::new());
    });
    let outcome = std::panic::catch_unwind(operation);
    let logs = TEST_LOG_CAPTURE.with(|capture| capture.borrow_mut().take().unwrap_or_default());
    match outcome {
        Ok(result) => (result, logs),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

#[cfg(test)]
fn record_test_log_line(line: &str) {
    TEST_LOG_CAPTURE.with(|capture| {
        if let Some(lines) = capture.borrow_mut().as_mut() {
            lines.push(line.to_owned());
        }
    });
}

#[cfg(not(test))]
fn record_test_log_line(_line: &str) {}
