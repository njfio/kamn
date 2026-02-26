use kamn_core::ConfigError;
use std::env;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const KAMN_NODE_LOG_LEVEL_ENV: &str = "KAMN_NODE_LOG_LEVEL";
pub(crate) const KAMN_NODE_LOG_FORMAT_ENV: &str = "KAMN_NODE_LOG_FORMAT";
const LOG_FIELD_CORRELATION_ID: &str = "correlation_id";
const LOG_FIELD_REASON_CODE: &str = "reason_code";
const LOG_DEFAULT_CORRELATION_ID: &str = "none";
const LOG_DEFAULT_REASON_CODE: &str = "none";

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

static LOG_CONFIG_CACHE: RwLock<Option<NodeLogConfig>> = RwLock::new(None);

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
    let config = resolve_cached_log_config()?;
    if !config.allows(level) {
        return Ok(());
    }
    let line = render_log_event_line(config, level, event, fields);
    record_test_log_line(line.as_str());
    eprintln!("{line}");
    Ok(())
}

fn resolve_cached_log_config() -> Result<NodeLogConfig, ConfigError> {
    if let Some(config) = read_cached_log_config() {
        return Ok(config);
    }

    let resolved = resolve_log_config_from_env()?;
    write_cached_log_config_if_absent(resolved);
    Ok(read_cached_log_config().unwrap_or(resolved))
}

fn read_cached_log_config() -> Option<NodeLogConfig> {
    match LOG_CONFIG_CACHE.read() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

fn write_cached_log_config_if_absent(config: NodeLogConfig) {
    match LOG_CONFIG_CACHE.write() {
        Ok(mut guard) => {
            if guard.is_none() {
                *guard = Some(config);
            }
        }
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            if guard.is_none() {
                *guard = Some(config);
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn reset_cached_log_config_for_tests() {
    match LOG_CONFIG_CACHE.write() {
        Ok(mut guard) => *guard = None,
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            *guard = None;
        }
    }
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
    let normalized_fields = normalize_log_fields(fields);
    let mut line = format!(
        "ts_unix_ms={timestamp_ms} level={} event={}",
        level.as_str(),
        render_text_field_value(event)
    );
    for &(key, value) in &normalized_fields {
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
    let normalized_fields = normalize_log_fields(fields);
    let mut line = format!(
        "{{\"ts_unix_ms\":{timestamp_ms},\"level\":\"{}\",\"event\":\"{}\"",
        level.as_str(),
        escape_json_string(event)
    );
    if normalized_fields.is_empty() {
        line.push('}');
        return line;
    }
    line.push_str(",\"fields\":{");
    for (index, (key, value)) in normalized_fields.iter().enumerate() {
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

fn normalize_log_fields<'a>(fields: &'a [(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
    let mut normalized = Vec::with_capacity(fields.len() + 2);
    let mut has_correlation_id = false;
    let mut has_reason_code = false;

    for &(key, value) in fields {
        if key == LOG_FIELD_CORRELATION_ID {
            if value.trim().is_empty() {
                continue;
            }
            has_correlation_id = true;
            normalized.push((key, value));
            continue;
        }
        if key == LOG_FIELD_REASON_CODE {
            if value.trim().is_empty() {
                continue;
            }
            has_reason_code = true;
            normalized.push((key, value));
            continue;
        }
        normalized.push((key, value));
    }

    if !has_correlation_id {
        normalized.push((LOG_FIELD_CORRELATION_ID, LOG_DEFAULT_CORRELATION_ID));
    }
    if !has_reason_code {
        normalized.push((LOG_FIELD_REASON_CODE, LOG_DEFAULT_REASON_CODE));
    }

    normalized
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
static TEST_LOG_CAPTURE: std::sync::Mutex<Option<Vec<String>>> = std::sync::Mutex::new(None);

#[cfg(test)]
fn with_test_log_capture_mut<T, F>(operation: F) -> T
where
    F: FnOnce(&mut Option<Vec<String>>) -> T,
{
    match TEST_LOG_CAPTURE.lock() {
        Ok(mut guard) => operation(&mut guard),
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            operation(&mut guard)
        }
    }
}

#[cfg(test)]
pub(crate) fn capture_test_logs<T, F>(operation: F) -> (T, Vec<String>)
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    with_test_log_capture_mut(|capture| *capture = Some(Vec::new()));
    let outcome = std::panic::catch_unwind(operation);
    let logs = with_test_log_capture_mut(|capture| capture.take().unwrap_or_default());
    match outcome {
        Ok(result) => (result, logs),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

#[cfg(test)]
fn record_test_log_line(line: &str) {
    with_test_log_capture_mut(|capture| {
        if let Some(lines) = capture.as_mut() {
            lines.push(line.to_owned());
        }
    });
}

#[cfg(not(test))]
fn record_test_log_line(_line: &str) {}

#[cfg(test)]
mod tests {
    use super::{
        capture_test_logs, emit_log_event, reset_cached_log_config_for_tests, NodeLogLevel,
        KAMN_NODE_LOG_FORMAT_ENV, KAMN_NODE_LOG_LEVEL_ENV,
    };
    use std::env;
    use std::sync::Mutex;

    static LOG_ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn spec_c01_emit_log_event_uses_cached_config_until_reset() {
        let _guard = lock_log_env_test_guard();
        reset_cached_log_config_for_tests();
        let _level_guard = EnvVarTestGuard::set(KAMN_NODE_LOG_LEVEL_ENV, Some("error"));
        let _format_guard = EnvVarTestGuard::set(KAMN_NODE_LOG_FORMAT_ENV, Some("text"));

        let (first_result, first_logs) =
            capture_test_logs(|| emit_log_event(NodeLogLevel::Info, "cache-check-1", &[]));
        assert!(first_result.is_ok(), "first log emission should not error");
        assert!(
            first_logs.is_empty(),
            "error level should suppress info event before cache change check"
        );

        env::set_var(KAMN_NODE_LOG_LEVEL_ENV, "trace");
        let (second_result, second_logs) =
            capture_test_logs(|| emit_log_event(NodeLogLevel::Info, "cache-check-2", &[]));
        assert!(
            second_result.is_ok(),
            "second log emission should not error"
        );
        assert!(
            second_logs.is_empty(),
            "cached config should remain in effect until explicit reset"
        );
    }

    #[test]
    fn regression_cached_log_config_reset_applies_updated_env() {
        let _guard = lock_log_env_test_guard();
        reset_cached_log_config_for_tests();
        let _level_guard = EnvVarTestGuard::set(KAMN_NODE_LOG_LEVEL_ENV, Some("error"));
        let _format_guard = EnvVarTestGuard::set(KAMN_NODE_LOG_FORMAT_ENV, Some("text"));

        let (initial_result, initial_logs) =
            capture_test_logs(|| emit_log_event(NodeLogLevel::Info, "cache-reset-1", &[]));
        assert!(
            initial_result.is_ok(),
            "initial log emission should not error"
        );
        assert!(
            initial_logs.is_empty(),
            "error level should suppress info before cache reset"
        );

        env::set_var(KAMN_NODE_LOG_LEVEL_ENV, "trace");
        reset_cached_log_config_for_tests();
        let (after_reset_result, after_reset_logs) =
            capture_test_logs(|| emit_log_event(NodeLogLevel::Info, "cache-reset-2", &[]));
        assert!(
            after_reset_result.is_ok(),
            "post-reset log emission should not error"
        );
        assert_eq!(
            after_reset_logs.len(),
            1,
            "info event should emit after cache reset"
        );
        assert!(
            after_reset_logs[0].contains("event=cache-reset-2"),
            "rendered line should include emitted event"
        );
    }

    #[test]
    fn unit_cached_log_config_preserves_json_format_rendering() {
        let _guard = lock_log_env_test_guard();
        reset_cached_log_config_for_tests();
        let _level_guard = EnvVarTestGuard::set(KAMN_NODE_LOG_LEVEL_ENV, Some("info"));
        let _format_guard = EnvVarTestGuard::set(KAMN_NODE_LOG_FORMAT_ENV, Some("json"));

        let (result, logs) =
            capture_test_logs(|| emit_log_event(NodeLogLevel::Info, "json-format", &[]));
        assert!(result.is_ok(), "json-format log emission should not error");
        assert_eq!(logs.len(), 1, "json format path should emit one log line");
        assert!(
            logs[0].starts_with("{\"ts_unix_ms\":"),
            "json format must keep json payload structure"
        );
        assert!(
            logs[0].contains("\"event\":\"json-format\""),
            "json log line should include event field"
        );
    }

    fn lock_log_env_test_guard() -> std::sync::MutexGuard<'static, ()> {
        LOG_ENV_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    struct EnvVarTestGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarTestGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = env::var(key).ok();
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarTestGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_deref() {
                env::set_var(self.key, previous);
            } else {
                env::remove_var(self.key);
            }
            reset_cached_log_config_for_tests();
        }
    }
}
