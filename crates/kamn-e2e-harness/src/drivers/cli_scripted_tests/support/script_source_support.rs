pub(crate) const HEALTH_DETERMINISTIC_OPT_IN_SCRIPT: &str = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
if command != "health":
    sys.stderr.write("unexpected command")
    sys.exit(2)

if os.environ.get("KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY") != "1":
    sys.stderr.write("deterministic identity opt-in missing")
    sys.exit(3)

sys.stdout.write("status=ok")
"#;

pub(crate) const S03_QUERY_MISMATCH_SCRIPT: &str = r#"#!/usr/bin/env python3
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
if command == "create-channel":
    sys.stdout.write("channel_id=channel-1 status=created")
elif command == "send-message":
    sys.stdout.write("message_id=message-1 status=sent")
elif command == "query-message":
    sys.stdout.write("message_id=message-2 status=sent")
elif command == "list-messages":
    sys.stdout.write("channel_id=channel-1 messages=[message-1]")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;

pub(crate) const S03_LIST_MISMATCH_SCRIPT: &str = r#"#!/usr/bin/env python3
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
if command == "create-channel":
    sys.stdout.write("channel_id=channel-1 status=created")
elif command == "send-message":
    sys.stdout.write("message_id=message-1 status=sent")
elif command == "query-message":
    sys.stdout.write("message_id=message-1 status=sent")
elif command == "list-messages":
    sys.stdout.write("channel_id=channel-2 messages=[message-1]")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;

pub(crate) const S08_CONTINUITY_SCRIPT: &str = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
agent_name = os.environ.get("KAMN_AGENT_NAME", "")

if command == "send-message":
    if agent_name.endswith("pre-send"):
        sys.stdout.write("message_id=message-pre status=sent")
    elif agent_name.endswith("post-send"):
        sys.stdout.write("message_id=message-post status=sent")
    else:
        sys.stdout.write("message_id=message-fallback status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-fallback"
    sys.stdout.write(f"message_id={message_id} status=sent")
elif command == "health":
    sys.stdout.write("status=ok")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;

pub(crate) const S10_TOPOLOGY_SCRIPT: &str = r#"#!/usr/bin/env python3
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
if command == "send-message":
    sys.stdout.write("message_id=message-primary status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-primary"
    sys.stdout.write(f"message_id={message_id} status=sent")
elif command == "health":
    sys.stdout.write("status=ok")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;

pub(crate) const S11_ROTATION_SCRIPT: &str = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
agent_name = os.environ.get("KAMN_AGENT_NAME", "")
payload = sys.argv[-1] if len(sys.argv) > 0 else ""
primary_agent_name = os.environ.get("KAMN_E2E_S11_PRIMARY_AGENT_NAME", "kamn-e2e-cli-s11-primary")
rotated_agent_name = os.environ.get("KAMN_E2E_S11_ROTATED_AGENT_NAME", f"{primary_agent_name}-rotated")
stale_payload = os.environ.get("KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD", "{\"message\":\"cli-scripted-live-s11-stale\"}")

if command == "send-message":
    if agent_name == primary_agent_name and payload == stale_payload:
        sys.stderr.write("service_api_auth_replay_nonce_detected")
        sys.exit(1)
    if agent_name == primary_agent_name:
        sys.stdout.write("message_id=message-primary status=sent")
    elif agent_name == rotated_agent_name:
        sys.stdout.write("message_id=message-rotated status=sent")
    else:
        sys.stdout.write("message_id=message-fallback status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-fallback"
    sys.stdout.write(f"message_id={message_id} status=sent")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;

pub(crate) const S14_BATCH_MERKLE_SCRIPT: &str = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
agent_name = os.environ.get("KAMN_AGENT_NAME", "")

if command == "send-message":
    if agent_name.endswith("-batch-a"):
        sys.stdout.write("message_id=message-batch-a status=sent")
    elif agent_name.endswith("-batch-b"):
        sys.stdout.write("message_id=message-batch-b status=sent")
    else:
        sys.stdout.write("message_id=message-fallback status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-fallback"
    sys.stdout.write(f"message_id={message_id} status=sent")
elif command == "verify-proof":
    message_id = sys.argv[-4] if len(sys.argv) >= 4 else "message-fallback"
    block_height = sys.argv[-2] if len(sys.argv) >= 2 else "1"
    sys.stdout.write(
        f"message_id={message_id} verified=true finality=FINAL block_height={block_height}"
    )
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;

pub(crate) fn s06_success_payload_script() -> String {
    format!(
        r#"#!/usr/bin/env python3
import sys
sys.stdout.write({payload:?})
"#,
        payload = "message_id=s06-live-proof block_height=1 finality=FINAL verified=true"
    )
}
