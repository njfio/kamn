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

api_stdout="$TMP_DIR/service-api-websocket.out"
"$NODE_BIN" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode api \
  --api-bind "$api_addr" \
  --api-max-requests 7 \
  --api-idle-timeout-ms 5000 \
  --output json >"$api_stdout" 2>&1 &
node_pid=$!

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
  echo "expected websocket api endpoint to become ready" >&2
  kill -KILL "$node_pid" 2>/dev/null || true
  wait "$node_pid" 2>/dev/null || true
  exit 1
fi

ws_results_json="$TMP_DIR/websocket-results.json"
python3 - "$api_addr" "$ws_results_json" <<'PY'
import json
import socket
import sys

api_addr = sys.argv[1]
report_path = sys.argv[2]
host, port_text = api_addr.rsplit(":", 1)
port = int(port_text)

sender_did = "kamn:did:agent:websocket-live-validator"
state_hash = "service-api:kamn-devnet:v0.1.0"


def signature(nonce: int, payload: str) -> str:
    return f"sig:ed25519:baseline-v1:{sender_did}:{nonce}:{state_hash}:{len(payload)}"


def send_raw_request(request: str) -> bytes:
    sock = socket.create_connection((host, port), timeout=2)
    try:
        sock.sendall(request.encode("utf-8"))
        sock.shutdown(socket.SHUT_WR)
        chunks: list[bytes] = []
        while True:
            try:
                chunk = sock.recv(4096)
            except socket.timeout:
                break
            if not chunk:
                break
            chunks.append(chunk)
        return b"".join(chunks)
    finally:
        sock.close()


def split_response(response: bytes) -> tuple[str, bytes]:
    marker = b"\r\n\r\n"
    index = response.find(marker)
    if index == -1:
        raise SystemExit("websocket response header terminator missing")
    header = response[: index + len(marker)].decode("utf-8")
    return header, response[index + len(marker) :]


def parse_small_text_frame(payload: bytes) -> str:
    if len(payload) < 2:
        raise SystemExit("websocket frame missing")
    if payload[0] != 0x81:
        raise SystemExit("websocket opcode mismatch")
    length = payload[1] & 0x7F
    if payload[1] & 0x80:
        raise SystemExit("server websocket frame must not be masked")
    if len(payload) < 2 + length:
        raise SystemExit("websocket frame payload truncated")
    return payload[2 : 2 + length].decode("utf-8")


def ws_upgrade_request(nonce: int, version: str, include_upgrade: bool, include_auth: bool) -> str:
    headers = [
        f"Host: {api_addr}",
        "Sec-WebSocket-Key: test-live-key",
        f"Sec-WebSocket-Version: {version}",
    ]
    if include_upgrade:
        headers.extend(["Connection: Upgrade", "Upgrade: websocket"])
    if include_auth:
        headers.extend(
            [
                f"X-KAMN-Sender-DID: {sender_did}",
                f"X-KAMN-Request-Nonce: {nonce}",
                f"X-KAMN-Request-Signature: {signature(nonce, '')}",
            ]
        )
    header_block = "\r\n".join(headers)
    return (
        "GET /v1/events/ws HTTP/1.1\r\n"
        + header_block
        + "\r\nContent-Length: 0\r\n\r\n"
    )


result = {
    "websocket_upgrade_status": "unknown",
    "fail_closed_status": "unknown",
}

# Success: proper upgrade + auth + version
success_response = send_raw_request(
    ws_upgrade_request(nonce=1, version="13", include_upgrade=True, include_auth=True)
)
success_header, success_payload = split_response(success_response)
if "HTTP/1.1 101 Switching Protocols" not in success_header:
    raise SystemExit("expected websocket success status line")
if "X-KAMN-WebSocket-Contract: v1" not in success_header:
    raise SystemExit("expected websocket contract header")
success_event = parse_small_text_frame(success_payload)
if "\"event\":\"state-transition\"" not in success_event:
    raise SystemExit("expected websocket event marker")
result["websocket_upgrade_status"] = "verified"

# Fail-closed: invalid websocket version
invalid_version_response = send_raw_request(
    ws_upgrade_request(nonce=2, version="12", include_upgrade=True, include_auth=True)
)
invalid_version_header, invalid_version_body = split_response(invalid_version_response)
if "HTTP/1.1 400 Bad Request" not in invalid_version_header:
    raise SystemExit("expected invalid websocket version rejection")
if "invalid websocket version header" not in invalid_version_body.decode("utf-8", errors="ignore"):
    raise SystemExit("expected invalid websocket version reason marker")

# Fail-closed: missing upgrade headers
missing_upgrade_response = send_raw_request(
    ws_upgrade_request(nonce=3, version="13", include_upgrade=False, include_auth=True)
)
missing_upgrade_header, missing_upgrade_body = split_response(missing_upgrade_response)
if "HTTP/1.1 400 Bad Request" not in missing_upgrade_header:
    raise SystemExit("expected missing upgrade header rejection")
if "missing required websocket upgrade header" not in missing_upgrade_body.decode(
    "utf-8", errors="ignore"
):
    raise SystemExit("expected missing upgrade reason marker")

# Fail-closed: unauthorized websocket request
unauthorized_response = send_raw_request(
    ws_upgrade_request(nonce=4, version="13", include_upgrade=True, include_auth=False)
)
unauthorized_header, unauthorized_body = split_response(unauthorized_response)
if "HTTP/1.1 401 Unauthorized" not in unauthorized_header:
    raise SystemExit("expected unauthorized websocket rejection")
if "\"error\":\"unauthorized\"" not in unauthorized_body.decode("utf-8", errors="ignore"):
    raise SystemExit("expected unauthorized error marker")

result["fail_closed_status"] = "verified"

with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(result, handle)
PY

websocket_upgrade_status="$(python3 - "$ws_results_json" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["websocket_upgrade_status"])
PY
)"
if [ "$websocket_upgrade_status" != "verified" ]; then
  echo "expected websocket upgrade status to be verified" >&2
  exit 1
fi

fail_closed_status="$(python3 - "$ws_results_json" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["fail_closed_status"])
PY
)"
if [ "$fail_closed_status" != "verified" ]; then
  echo "expected websocket fail-closed status to be verified" >&2
  exit 1
fi

health_file="$TMP_DIR/health.json"
metrics_file="$TMP_DIR/metrics.txt"
health_status="$(curl -sS -o "$health_file" -w '%{http_code}' "http://${api_addr}/healthz")"
if [ "$health_status" != "200" ]; then
  cat "$health_file" >&2
  echo "expected websocket live health probe to return 200" >&2
  exit 1
fi
if ! grep -q '"status":"ok"' "$health_file"; then
  cat "$health_file" >&2
  echo "expected websocket live health payload marker" >&2
  exit 1
fi
metrics_status="$(curl -sS -o "$metrics_file" -w '%{http_code}' "http://${api_addr}/metrics")"
if [ "$metrics_status" != "200" ]; then
  cat "$metrics_file" >&2
  echo "expected websocket live metrics probe to return 200" >&2
  exit 1
fi
if ! grep -q 'kamn_service_api_health' "$metrics_file"; then
  cat "$metrics_file" >&2
  echo "expected websocket live metrics marker" >&2
  exit 1
fi
probe_status="verified"

set +e
wait "$node_pid"
node_exit_code=$?
set -e
if [ "$node_exit_code" -ne 0 ]; then
  cat "$api_stdout" >&2
  echo "expected websocket live process to exit cleanly after request budget" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "websocket live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/websocket-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.runtime.service-api-websocket-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "websocket_upgrade_status": "${websocket_upgrade_status}",
  "fail_closed_status": "${fail_closed_status}",
  "probe_status": "${probe_status}",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "websocket_upgrade_status=${websocket_upgrade_status}"
echo "fail_closed_status=${fail_closed_status}"
echo "probe_status=${probe_status}"
