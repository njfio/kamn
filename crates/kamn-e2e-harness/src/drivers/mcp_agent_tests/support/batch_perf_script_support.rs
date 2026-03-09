use super::script_core_support::write_executable_python_script;
use std::path::Path;

const S14_TEMPLATE: &str = r#"#!/usr/bin/env python3
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
    if agent_name.endswith("batch-a"):
        result.update({"message_id": "message-batch-a", "status": "sent"})
    elif agent_name.endswith("batch-b"):
        result.update({"message_id": "message-batch-b", "status": "sent"})
    else:
        result.update({"message_id": "message-fallback", "status": "sent"})
elif tool_name == "query_message":
    message_match = re.search(r'"message_id":"([^"]+)"', stream)
    query_message_id = message_match.group(1) if message_match else "message-fallback"
    result.update({"message_id": query_message_id, "status": "sent"})
elif tool_name == "verify_proof":
    message_match = re.search(r'"message_id":"([^"]+)"', stream)
    verify_message_id = message_match.group(1) if message_match else "message-fallback"
    block_height_match = re.search(r'"block_height":"([0-9]+)"', stream)
    block_height = int(block_height_match.group(1)) if block_height_match else 1
    result.update({"message_id": verify_message_id, "verified": True, "finality": "FINAL", "block_height": block_height})
else:
    result.update({"error": "unsupported_tool"})

init_payload = {"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}
tool_payload = {"jsonrpc":"2.0","id":request_id,"result":result}

def frame(payload):
    body = json.dumps(payload, separators=(",", ":"))
    return f"Content-Length: {len(body)}\r\n\r\n{body}"

sys.stdout.write(frame(init_payload) + frame(tool_payload))
"#;

const S15_TEMPLATE: &str = r#"#!/usr/bin/env python3
import json
import re
import sys

stream = sys.stdin.read()
request_ids = re.findall(r'"id":"([^"]+)"', stream)
request_id = request_ids[-1] if request_ids else "probe-request"
tool_names = re.findall(r'"name":"([^"]+)"', stream)
tool_name = tool_names[-1] if tool_names else ""

result = {"ok": True}
if tool_name == "send_message":
    if request_id.endswith("-0"):
        result.update({"message_id": "message-s15-0", "status": "sent"})
    elif request_id.endswith("-1"):
        result.update({"message_id": "message-s15-1", "status": "sent"})
    elif request_id.endswith("-2"):
        result.update({"message_id": "message-s15-2", "status": "sent"})
    else:
        result.update({"message_id": "message-s15-fallback", "status": "sent"})
elif tool_name == "query_message":
    message_match = re.search(r'"message_id":"([^"]+)"', stream)
    query_message_id = message_match.group(1) if message_match else "message-s15-fallback"
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

pub(crate) fn write_mcp_s14_probe_script(script_path: &Path) {
    write_executable_python_script(script_path, S14_TEMPLATE);
}

pub(crate) fn write_mcp_s15_probe_script(script_path: &Path) {
    write_executable_python_script(script_path, S15_TEMPLATE);
}
