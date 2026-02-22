use kamn_agent_lib::{AgentLibError, KamnAgentHandle};

const UNSUPPORTED_REASON: &str =
    "operation is not yet supported by the current service route surface";

/// Backend abstraction used by MCP tool dispatch.
pub trait McpToolBackend {
    /// Registers the local agent identity.
    fn register(&self) -> Result<String, AgentLibError>;
    /// Sends one message payload.
    fn send_message(&self, payload: &str) -> Result<String, AgentLibError>;
    /// Creates one channel payload.
    fn create_channel(&self, payload: &str) -> Result<String, AgentLibError>;
    /// Lists messages for one channel identifier.
    fn list_messages(&self, channel_id: &str) -> Result<String, AgentLibError>;
    /// Queries one message status by identifier.
    fn query_message(&self, message_id: &str) -> Result<String, AgentLibError>;
    /// Creates one task payload.
    fn create_task(&self, payload: &str) -> Result<String, AgentLibError>;
    /// Reads service health.
    fn health(&self) -> Result<String, AgentLibError>;
}

impl McpToolBackend for KamnAgentHandle {
    fn register(&self) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"did":"{}"}}"#,
            escape_json(self.identity().did().as_str())
        ))
    }

    fn send_message(&self, payload: &str) -> Result<String, AgentLibError> {
        let receipt = KamnAgentHandle::send_message(self, payload)?;
        Ok(format!(
            r#"{{"message_id":"{}","status":"{}","runtime_mode":"{}"}}"#,
            escape_json(receipt.message_id.as_str()),
            escape_json(receipt.status.as_str()),
            escape_json(receipt.runtime_mode.as_str()),
        ))
    }

    fn create_channel(&self, payload: &str) -> Result<String, AgentLibError> {
        let receipt = KamnAgentHandle::create_channel(self, payload)?;
        Ok(format!(
            r#"{{"channel_id":"{}","status":"{}"}}"#,
            escape_json(receipt.channel_id.as_str()),
            escape_json(receipt.status.as_str()),
        ))
    }

    fn list_messages(&self, channel_id: &str) -> Result<String, AgentLibError> {
        let listing = KamnAgentHandle::list_messages(self, channel_id)?;
        let messages = listing
            .messages
            .iter()
            .map(|value| format!(r#""{}""#, escape_json(value.as_str())))
            .collect::<Vec<_>>()
            .join(",");
        Ok(format!(
            r#"{{"channel_id":"{}","messages":[{}]}}"#,
            escape_json(listing.channel_id.as_str()),
            messages,
        ))
    }

    fn query_message(&self, message_id: &str) -> Result<String, AgentLibError> {
        let status = KamnAgentHandle::query_message(self, message_id)?;
        Ok(format!(
            r#"{{"message_id":"{}","status":"{}"}}"#,
            escape_json(status.message_id.as_str()),
            escape_json(status.status.as_str()),
        ))
    }

    fn create_task(&self, payload: &str) -> Result<String, AgentLibError> {
        let receipt = KamnAgentHandle::create_task(self, payload)?;
        Ok(format!(
            r#"{{"task_id":"{}","state":"{}"}}"#,
            escape_json(receipt.task_id.as_str()),
            escape_json(receipt.state.as_str()),
        ))
    }

    fn health(&self) -> Result<String, AgentLibError> {
        let health = KamnAgentHandle::health(self)?;
        Ok(format!(
            r#"{{"status":"{}","runtime_mode":"{}","role":"{}","observability_source":"{}","observability_health":"{}"}}"#,
            escape_json(health.status.as_str()),
            escape_json(health.runtime_mode.as_str()),
            escape_json(health.role.as_str()),
            escape_json(health.observability_source.as_str()),
            escape_json(health.observability_health.as_str()),
        ))
    }
}

/// Dispatches one JSON tool request into a JSON response envelope.
pub fn dispatch_tool_request_json<B: McpToolBackend>(
    backend: &B,
    request_json: &str,
) -> Result<String, String> {
    let tool = json_string_field(request_json, "tool")?;
    let request_id = json_optional_string_field(request_json, "id")
        .unwrap_or_else(|| "request-unknown".to_owned());

    let operation = match tool.as_str() {
        "register" => backend.register(),
        "send_message" => {
            backend.send_message(required_string_arg(request_json, "payload")?.as_str())
        }
        "create_channel" => {
            backend.create_channel(required_string_arg(request_json, "payload")?.as_str())
        }
        "list_messages" => {
            backend.list_messages(required_string_arg(request_json, "channel_id")?.as_str())
        }
        "query_message" => {
            backend.query_message(required_string_arg(request_json, "message_id")?.as_str())
        }
        "create_task" => {
            backend.create_task(required_string_arg(request_json, "payload")?.as_str())
        }
        "health" => backend.health(),
        "accept_task" | "complete_task" | "fund_escrow" | "release_escrow" | "verify_proof" => {
            return Ok(unsupported_response_json(
                request_id.as_str(),
                tool.as_str(),
                UNSUPPORTED_REASON,
            ));
        }
        unknown => {
            return Ok(unsupported_response_json(
                request_id.as_str(),
                unknown,
                "unknown tool name",
            ));
        }
    };

    match operation {
        Ok(result) => Ok(success_response_json(
            request_id.as_str(),
            tool.as_str(),
            result.as_str(),
        )),
        Err(AgentLibError::UnsupportedOperation(message)) => Ok(unsupported_response_json(
            request_id.as_str(),
            tool.as_str(),
            message,
        )),
        Err(error) => Ok(backend_error_response_json(
            request_id.as_str(),
            tool.as_str(),
            error.to_string().as_str(),
        )),
    }
}

/// Deterministic invalid-request envelope for malformed input payloads.
pub fn invalid_request_response_json(error_message: &str) -> String {
    format!(
        r#"{{"ok":false,"error":{{"kind":"invalid_request","message":"{}"}}}}"#,
        escape_json(error_message),
    )
}

fn required_string_arg(payload: &str, key: &str) -> Result<String, String> {
    json_string_field(payload, key)
}

fn success_response_json(request_id: &str, tool: &str, result_payload: &str) -> String {
    format!(
        r#"{{"ok":true,"id":"{}","tool":"{}","result":{}}}"#,
        escape_json(request_id),
        escape_json(tool),
        result_payload,
    )
}

fn unsupported_response_json(request_id: &str, tool: &str, reason: &str) -> String {
    format!(
        r#"{{"ok":false,"id":"{}","tool":"{}","error":{{"kind":"unsupported_operation","message":"{}"}}}}"#,
        escape_json(request_id),
        escape_json(tool),
        escape_json(reason),
    )
}

fn backend_error_response_json(request_id: &str, tool: &str, message: &str) -> String {
    format!(
        r#"{{"ok":false,"id":"{}","tool":"{}","error":{{"kind":"backend_error","message":"{}"}}}}"#,
        escape_json(request_id),
        escape_json(tool),
        escape_json(message),
    )
}

fn json_optional_string_field(payload: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = payload.find(marker.as_str())? + marker.len();
    let rest = &payload[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

fn json_string_field(payload: &str, key: &str) -> Result<String, String> {
    json_optional_string_field(payload, key).ok_or_else(|| format!("missing required field: {key}"))
}

fn escape_json(input: &str) -> String {
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
