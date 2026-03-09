use super::script_core_support::write_executable_python_script;
use std::path::Path;

const S03_TEMPLATE: &str = r#"#!/usr/bin/env python3
import json
import re
import sys

query_message_id = __QUERY_MESSAGE_ID__
list_channel_id = __LIST_CHANNEL_ID__
include_messages = __INCLUDE_MESSAGES__

stream = sys.stdin.read()
request_ids = re.findall(r'"id":"([^"]+)"', stream)
request_id = request_ids[-1] if request_ids else "probe-request"
tool_names = re.findall(r'"name":"([^"]+)"', stream)
tool_name = tool_names[-1] if tool_names else ""

result = {"ok": True}
if tool_name == "create_channel":
    result.update({"channel_id":"channel-1","status":"created"})
elif tool_name == "send_message":
    result.update({"message_id":"message-1","status":"sent","channel_id":"channel-1"})
elif tool_name == "query_message":
    result.update({"message_id": query_message_id, "status":"sent"})
elif tool_name == "list_messages":
    result.update({"channel_id": list_channel_id})
    if include_messages:
        result.update({"messages":["message-1"]})
else:
    result.update({"error":"unsupported_tool"})

init_payload = {"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}
tool_payload = {"jsonrpc":"2.0","id":request_id,"result":result}

def frame(payload):
    body = json.dumps(payload, separators=(",", ":"))
    return f"Content-Length: {len(body)}\r\n\r\n{body}"

sys.stdout.write(frame(init_payload) + frame(tool_payload))
"#;

const S08_TEMPLATE: &str = r#"#!/usr/bin/env python3
import json
import re
import sys

agent_name = ""
if "--agent-name" in sys.argv:
    index = sys.argv.index("--agent-name")
    if index + 1 < len(sys.argv):
        agent_name = sys.argv[index + 1]

stream = sys.stdin.read()
request_ids = re.findall(r'"id":"([^"]+)"', stream)
request_id = request_ids[-1] if request_ids else "probe-request"
tool_names = re.findall(r'"name":"([^"]+)"', stream)
tool_name = tool_names[-1] if tool_names else ""

result = {"ok": True}
if tool_name == "send_message":
    if agent_name.endswith("pre-send"):
        result.update({"message_id": "message-pre", "status": "sent"})
    elif agent_name.endswith("post-send"):
        result.update({"message_id": "message-post", "status": "sent"})
    else:
        result.update({"message_id": "message-fallback", "status": "sent"})
elif tool_name == "query_message":
    message_match = re.search(r'"message_id":"([^"]+)"', stream)
    query_message_id = message_match.group(1) if message_match else "message-fallback"
    result.update({"message_id": query_message_id, "status": "sent"})
elif tool_name == "health":
    result.update({"status": "ok"})
else:
    result.update({"error": "unsupported_tool"})

init_payload = {"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}
tool_payload = {"jsonrpc":"2.0","id":request_id,"result":result}

def frame(payload):
    body = json.dumps(payload, separators=(",", ":"))
    return f"Content-Length: {len(body)}\r\n\r\n{body}"

sys.stdout.write(frame(init_payload) + frame(tool_payload))
"#;

const S11_TEMPLATE: &str = r#"#!/usr/bin/env python3
import json
import re
import sys

agent_name = ""
if "--agent-name" in sys.argv:
    index = sys.argv.index("--agent-name")
    if index + 1 < len(sys.argv):
        agent_name = sys.argv[index + 1]

stream = sys.stdin.read()
request_ids = re.findall(r'"id":"([^"]+)"', stream)
request_id = request_ids[-1] if request_ids else "probe-request"
tool_names = re.findall(r'"name":"([^"]+)"', stream)
tool_name = tool_names[-1] if tool_names else ""

result = {"ok": True}
if tool_name == "send_message":
    if "stale-primary" in request_id:
        result = {"ok": False, "error": {"kind": "backend_error", "message": "service_api_auth_replay_nonce_detected"}}
    elif agent_name.endswith("primary"):
        result.update({"message_id": "message-primary", "status": "sent"})
    elif agent_name.endswith("rotated"):
        result.update({"message_id": "message-rotated", "status": "sent"})
    else:
        result.update({"message_id": "message-fallback", "status": "sent"})
elif tool_name == "query_message":
    message_match = re.search(r'"message_id":"([^"]+)"', stream)
    query_message_id = message_match.group(1) if message_match else "message-fallback"
    result.update({"message_id": query_message_id, "status": "sent"})
else:
    result.update({"error": "unsupported_tool"})

init_payload = {"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}
tool_payload = {"jsonrpc":"2.0","id":request_id,"result":result}

def frame(payload):
    body = json.dumps(payload, separators=(",", ":"))
    return f"Content-Length: {len(body)}\r\n\r\n{body}"

sys.stdout.write(frame(init_payload) + frame(tool_payload))
"#;

pub(crate) fn write_mcp_s03_probe_script(
    script_path: &Path,
    query_message_id: &str,
    list_channel_id: &str,
    include_messages: bool,
) {
    let script = render_s03_script(query_message_id, list_channel_id, include_messages);
    write_executable_python_script(script_path, script.as_str());
}

pub(crate) fn write_mcp_s08_probe_script(script_path: &Path) {
    write_executable_python_script(script_path, S08_TEMPLATE);
}

pub(crate) fn write_mcp_s11_probe_script(script_path: &Path) {
    write_executable_python_script(script_path, S11_TEMPLATE);
}

fn render_s03_script(query_message_id: &str, list_channel_id: &str, include_messages: bool) -> String {
    let include_messages_literal = if include_messages { "True" } else { "False" };
    S03_TEMPLATE
        .replace("__QUERY_MESSAGE_ID__", &format!("{query_message_id:?}"))
        .replace("__LIST_CHANNEL_ID__", &format!("{list_channel_id:?}"))
        .replace("__INCLUDE_MESSAGES__", include_messages_literal)
}
