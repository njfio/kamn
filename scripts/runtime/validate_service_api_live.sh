#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo build --quiet -p kamn-node
NODE_BIN="$ROOT_DIR/target/debug/kamn-node"
popd >/dev/null

if [ ! -x "$NODE_BIN" ]; then
  echo "expected built kamn-node binary to be executable" >&2
  exit 1
fi

api_port="$(python3 - <<'PY'
import socket
sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
)"
api_addr="127.0.0.1:${api_port}"

api_stdout="$TMP_DIR/service-api.out"
"$NODE_BIN" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode api \
  --api-bind "$api_addr" \
  --api-max-requests 10 \
  --api-idle-timeout-ms 5000 \
  --output json >"$api_stdout" 2>&1 &
node_pid=$!

auth_sender_did="kamn:did:agent:service-api-validator"
auth_state_hash="service-api:kamn-devnet:v0.1.0"

build_signature() {
  local nonce="$1"
  local payload="$2"
  printf 'sig:ed25519:baseline-v1:%s:%s:%s:%s' \
    "$auth_sender_did" \
    "$nonce" \
    "$auth_state_hash" \
    "${#payload}"
}

wait_for_ready=0
for _ in $(seq 1 120); do
  if curl -fsS "http://${api_addr}/healthz" >/dev/null 2>&1; then
    wait_for_ready=1
    break
  fi
  if ! kill -0 "$node_pid" 2>/dev/null; then
    break
  fi
  sleep 0.05
done

if [ "$wait_for_ready" -ne 1 ]; then
  cat "$api_stdout" >&2
  echo "expected service api endpoint to become ready" >&2
  kill -KILL "$node_pid" 2>/dev/null || true
  wait "$node_pid" 2>/dev/null || true
  exit 1
fi

message_response_file="$TMP_DIR/message-send.json"
channel_response_file="$TMP_DIR/channel-create.json"
task_response_file="$TMP_DIR/task-create.json"
health_response_file="$TMP_DIR/healthz.json"
metrics_response_file="$TMP_DIR/metrics.txt"
agent_response_file="$TMP_DIR/agent.json"
message_get_response_file="$TMP_DIR/message-get.json"
channel_get_response_file="$TMP_DIR/channel-get.json"
task_get_response_file="$TMP_DIR/task-get.json"

nonce_counter=1
message_send_body='{"message":"hello"}'
message_send_signature="$(build_signature "$nonce_counter" "$message_send_body")"
curl -fsS -X POST "http://${api_addr}/v1/messages/send" \
  -H 'content-type: application/json' \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce_counter}" \
  -H "X-KAMN-Request-Signature: ${message_send_signature}" \
  --data "$message_send_body" >"$message_response_file"

message_id="$(python3 - "$message_response_file" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["message_id"])
PY
)"

nonce_counter="$((nonce_counter + 1))"
message_get_signature="$(build_signature "$nonce_counter" "")"
curl -fsS "http://${api_addr}/v1/messages/${message_id}" \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce_counter}" \
  -H "X-KAMN-Request-Signature: ${message_get_signature}" >"$message_get_response_file"

nonce_counter="$((nonce_counter + 1))"
channel_create_body='{"name":"operators"}'
channel_create_signature="$(build_signature "$nonce_counter" "$channel_create_body")"
curl -fsS -X POST "http://${api_addr}/v1/channels/create" \
  -H 'content-type: application/json' \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce_counter}" \
  -H "X-KAMN-Request-Signature: ${channel_create_signature}" \
  --data "$channel_create_body" >"$channel_response_file"
channel_id="$(python3 - "$channel_response_file" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["channel_id"])
PY
)"

nonce_counter="$((nonce_counter + 1))"
channel_get_signature="$(build_signature "$nonce_counter" "")"
curl -fsS "http://${api_addr}/v1/channels/${channel_id}/messages" \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce_counter}" \
  -H "X-KAMN-Request-Signature: ${channel_get_signature}" >"$channel_get_response_file"

nonce_counter="$((nonce_counter + 1))"
task_create_body='{"title":"task"}'
task_create_signature="$(build_signature "$nonce_counter" "$task_create_body")"
curl -fsS -X POST "http://${api_addr}/v1/tasks/create" \
  -H 'content-type: application/json' \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce_counter}" \
  -H "X-KAMN-Request-Signature: ${task_create_signature}" \
  --data "$task_create_body" >"$task_response_file"
task_id="$(python3 - "$task_response_file" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["task_id"])
PY
)"

nonce_counter="$((nonce_counter + 1))"
task_get_signature="$(build_signature "$nonce_counter" "")"
curl -fsS "http://${api_addr}/v1/tasks/${task_id}" \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce_counter}" \
  -H "X-KAMN-Request-Signature: ${task_get_signature}" >"$task_get_response_file"
nonce_counter="$((nonce_counter + 1))"
agent_get_signature="$(build_signature "$nonce_counter" "")"
curl -fsS "http://${api_addr}/v1/agents/kamn:did:agent:alpha" \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce_counter}" \
  -H "X-KAMN-Request-Signature: ${agent_get_signature}" >"$agent_response_file"
curl -fsS "http://${api_addr}/healthz" >"$health_response_file"
curl -fsS "http://${api_addr}/metrics" >"$metrics_response_file"

set +e
wait "$node_pid"
node_exit_code=$?
set -e
if [ "$node_exit_code" -ne 0 ]; then
  cat "$api_stdout" >&2
  echo "expected service api process to exit cleanly after request budget" >&2
  exit 1
fi

if ! grep -q '"status":"ok"' "$health_response_file"; then
  cat "$health_response_file" >&2
  echo "expected health endpoint status marker" >&2
  exit 1
fi
if ! grep -q 'kamn_service_api_health' "$metrics_response_file"; then
  cat "$metrics_response_file" >&2
  echo "expected metrics endpoint marker" >&2
  exit 1
fi
if ! grep -q '"messages":\[\]' "$channel_get_response_file"; then
  cat "$channel_get_response_file" >&2
  echo "expected channel messages endpoint payload marker" >&2
  exit 1
fi
if ! grep -q '"reputation_score":500' "$agent_response_file"; then
  cat "$agent_response_file" >&2
  echo "expected agent endpoint payload marker" >&2
  exit 1
fi
if ! grep -q '"state":"submitted"' "$task_get_response_file"; then
  cat "$task_get_response_file" >&2
  echo "expected task endpoint payload marker" >&2
  exit 1
fi
if ! grep -q '"status":"created"' "$message_get_response_file"; then
  cat "$message_get_response_file" >&2
  echo "expected message endpoint payload marker" >&2
  exit 1
fi

set +e
failure_stdout="$($NODE_BIN \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode api \
  --output json 2>&1)"
failure_code=$?
set -e
if [ "$failure_code" -eq 0 ]; then
  echo "expected runtime-mode api without --api-bind to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$failure_stdout" | grep -q -- '--api-bind'; then
  printf '%s\n' "$failure_stdout" >&2
  echo "expected missing --api-bind reason marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/service-api-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.runtime.service-api-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "route_contract_status": "verified",
  "failure_case_status": "verified",
  "message_id": "${message_id}",
  "channel_id": "${channel_id}",
  "task_id": "${task_id}",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "route_contract_status=verified"
echo "failure_case_status=verified"
