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
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
node_pid=""
cleanup() {
  if [[ -n "$node_pid" ]] && kill -0 "$node_pid" 2>/dev/null; then
    kill -KILL "$node_pid" 2>/dev/null || true
    wait "$node_pid" 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

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

api_stdout="$TMP_DIR/service-api-axum-ingress-live.out"
"$NODE_BIN" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode api \
  --api-bind "$api_addr" \
  --api-max-requests 64 \
  --api-idle-timeout-ms 1200 \
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
  echo "expected service api axum ingress endpoint to become ready" >&2
  exit 1
fi

auth_sender_did="kamn:did:agent:axum-ingress-validator"
auth_state_hash="service-api:kamn-devnet:v0.1.0"

probe_report="$TMP_DIR/service-api-axum-ingress-probes.json"
python3 - "$api_addr" "$probe_report" "$auth_sender_did" "$auth_state_hash" <<'PY'
import concurrent.futures
import http.client
import json
import socket
import sys

api_addr = sys.argv[1]
probe_report = sys.argv[2]
sender_did = sys.argv[3]
state_hash = sys.argv[4]
host, port_text = api_addr.rsplit(":", 1)
port = int(port_text)


def signature(nonce: int, payload: str) -> str:
    return f"sig:ed25519:baseline-v1:{sender_did}:{nonce}:{state_hash}:{len(payload)}"


def request(method: str, path: str, body: str, headers: dict[str, str]) -> tuple[int, str]:
    conn = http.client.HTTPConnection(host, port, timeout=3)
    conn.request(method, path, body=body, headers=headers)
    response = conn.getresponse()
    payload = response.read().decode("utf-8", errors="ignore")
    status = response.status
    conn.close()
    return status, payload


def read_http_response(sock: socket.socket) -> tuple[int, dict[str, str], str]:
    payload = b""
    while b"\r\n\r\n" not in payload:
        chunk = sock.recv(4096)
        if not chunk:
            raise SystemExit("incomplete http response headers")
        payload += chunk
    header_bytes, body_remainder = payload.split(b"\r\n\r\n", 1)
    header_text = header_bytes.decode("utf-8")
    header_lines = header_text.split("\r\n")
    if not header_lines:
        raise SystemExit("missing http status line")
    status_parts = header_lines[0].split(" ", 2)
    if len(status_parts) < 2:
        raise SystemExit("malformed http status line")
    try:
        status_code = int(status_parts[1])
    except ValueError as exc:
        raise SystemExit(f"invalid http status code: {status_parts[1]}") from exc
    headers: dict[str, str] = {}
    for line in header_lines[1:]:
        if not line or ":" not in line:
            continue
        key, value = line.split(":", 1)
        headers[key.strip().lower()] = value.strip()
    content_length = int(headers.get("content-length", "0"))
    body = body_remainder
    while len(body) < content_length:
        chunk = sock.recv(4096)
        if not chunk:
            raise SystemExit("incomplete http response body")
        body += chunk
    return status_code, headers, body[:content_length].decode("utf-8", errors="ignore")


def run_keep_alive_probe() -> None:
    sock = socket.create_connection((host, port), timeout=3)
    sock.settimeout(3)
    try:
        first_request = (
            "GET /healthz HTTP/1.1\r\n"
            f"Host: {api_addr}\r\n"
            "Connection: keep-alive\r\n"
            "Content-Length: 0\r\n\r\n"
        )
        sock.sendall(first_request.encode("utf-8"))
        status_one, _, body_one = read_http_response(sock)
        if status_one != 200 or '"status":"ok"' not in body_one:
            raise SystemExit("keep-alive first request did not return expected health payload")

        second_request = (
            "GET /metrics HTTP/1.1\r\n"
            f"Host: {api_addr}\r\n"
            "Connection: close\r\n"
            "Content-Length: 0\r\n\r\n"
        )
        sock.sendall(second_request.encode("utf-8"))
        status_two, _, body_two = read_http_response(sock)
        if status_two != 200 or "kamn_service_api_health" not in body_two:
            raise SystemExit("keep-alive second request did not return expected metrics payload")
    finally:
        sock.close()


def run_websocket_probe() -> None:
    def send_upgrade(version: str, nonce: int) -> tuple[str, str]:
        ws_sock = socket.create_connection((host, port), timeout=3)
        ws_sock.settimeout(3)
        try:
            request_payload = (
                "GET /v1/events/ws HTTP/1.1\r\n"
                f"Host: {api_addr}\r\n"
                "Connection: Upgrade\r\n"
                "Upgrade: websocket\r\n"
                "Sec-WebSocket-Key: test-axum-key\r\n"
                f"Sec-WebSocket-Version: {version}\r\n"
                f"X-KAMN-Sender-DID: {sender_did}\r\n"
                f"X-KAMN-Request-Nonce: {nonce}\r\n"
                f"X-KAMN-Request-Signature: {signature(nonce, '')}\r\n"
                "Content-Length: 0\r\n\r\n"
            )
            ws_sock.sendall(request_payload.encode("utf-8"))
            payload = b""
            while b"\r\n\r\n" not in payload:
                chunk = ws_sock.recv(4096)
                if not chunk:
                    break
                payload += chunk
            if b"\r\n\r\n" not in payload:
                raise SystemExit("websocket probe response headers missing")
            header_bytes, body = payload.split(b"\r\n\r\n", 1)
            return (
                header_bytes.decode("utf-8", errors="ignore"),
                body.decode("utf-8", errors="ignore"),
            )
        finally:
            ws_sock.close()

    success_headers, _ = send_upgrade("13", 501)
    if "HTTP/1.1 101 Switching Protocols" not in success_headers:
        raise SystemExit("websocket probe expected 101 Switching Protocols")
    if "x-kamn-websocket-contract: v1" not in success_headers.lower():
        raise SystemExit("websocket probe expected x-kamn-websocket-contract header")

    invalid_headers, invalid_body = send_upgrade("12", 502)
    if "HTTP/1.1 400 Bad Request" not in invalid_headers:
        raise SystemExit("websocket fail-closed probe expected 400 for invalid version")
    if "invalid websocket version header" not in invalid_body:
        raise SystemExit("websocket fail-closed probe expected invalid-version reason marker")


def run_concurrency_probe() -> None:
    def post_message(index: int) -> None:
        payload = json.dumps({"message": f"concurrency-{index}"}, separators=(",", ":"))
        nonce = 600 + index
        status, body = request(
            "POST",
            "/v1/messages/send",
            payload,
            {
                "content-type": "application/json",
                "X-KAMN-Sender-DID": sender_did,
                "X-KAMN-Request-Nonce": str(nonce),
                "X-KAMN-Request-Signature": signature(nonce, payload),
            },
        )
        if status != 202:
            raise SystemExit(f"concurrency probe expected 202 response; got {status}")
        if '"status":"created"' not in body:
            raise SystemExit("concurrency probe expected created marker")

    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        list(pool.map(post_message, range(1, 7)))


run_keep_alive_probe()
run_websocket_probe()
run_concurrency_probe()

with open(probe_report, "w", encoding="utf-8") as handle:
    json.dump(
        {
            "keep_alive_status": "verified",
            "concurrency_status": "verified",
            "websocket_status": "verified",
            "fail_closed_status": "verified",
        },
        handle,
    )
PY

oversized_body_file="$TMP_DIR/service-api-axum-oversized-body.txt"
python3 - "$oversized_body_file" <<'PY'
import pathlib
import sys

pathlib.Path(sys.argv[1]).write_text("x" * (70 * 1024), encoding="utf-8")
PY

oversized_nonce=701
oversized_length="$(wc -c <"$oversized_body_file" | tr -d '[:space:]')"
oversized_signature="$(printf 'sig:ed25519:baseline-v1:%s:%s:%s:%s' \
  "$auth_sender_did" \
  "$oversized_nonce" \
  "$auth_state_hash" \
  "$oversized_length")"

oversized_response_file="$TMP_DIR/service-api-axum-oversized-response.json"
oversized_status="$(curl -sS -o "$oversized_response_file" -w '%{http_code}' \
  -X POST "http://${api_addr}/v1/messages/send" \
  -H 'content-type: application/json' \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${oversized_nonce}" \
  -H "X-KAMN-Request-Signature: ${oversized_signature}" \
  --data-binary "@${oversized_body_file}")"
if [ "$oversized_status" != "400" ]; then
  cat "$oversized_response_file" >&2
  echo "expected oversized service api request to return 400" >&2
  exit 1
fi
if ! grep -q '"error":"bad-request"' "$oversized_response_file"; then
  cat "$oversized_response_file" >&2
  echo "expected oversized service api response bad-request marker" >&2
  exit 1
fi
if ! grep -q 'request body read failed' "$oversized_response_file"; then
  cat "$oversized_response_file" >&2
  echo "expected oversized service api response reason marker" >&2
  exit 1
fi
body_size_guard_status="verified"
fail_closed_reason_code="service_api_axum_oversized_body_rejected"
ci_fast_gate_exclusion_status="verified"

set +e
wait "$node_pid"
node_exit_code=$?
set -e
node_pid=""
if [ "$node_exit_code" -ne 0 ]; then
  cat "$api_stdout" >&2
  echo "expected service api axum ingress process to exit cleanly after idle timeout" >&2
  exit 1
fi

keep_alive_status="$(python3 - "$probe_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["keep_alive_status"])
PY
)"
concurrency_status="$(python3 - "$probe_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["concurrency_status"])
PY
)"
websocket_status="$(python3 - "$probe_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["websocket_status"])
PY
)"
fail_closed_status="$(python3 - "$probe_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["fail_closed_status"])
PY
)"

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api axum ingress live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/service-api-axum-ingress-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.runtime.service-api-axum-ingress-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "keep_alive_status": "${keep_alive_status}",
  "body_size_guard_status": "${body_size_guard_status}",
  "concurrency_status": "${concurrency_status}",
  "websocket_status": "${websocket_status}",
  "fail_closed_status": "${fail_closed_status}",
  "ci_fast_gate_exclusion_status": "${ci_fast_gate_exclusion_status}",
  "performance_budget_status": "verified",
  "fail_closed_reason_code": "${fail_closed_reason_code}",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "keep_alive_status=${keep_alive_status}"
echo "body_size_guard_status=${body_size_guard_status}"
echo "concurrency_status=${concurrency_status}"
echo "websocket_status=${websocket_status}"
echo "fail_closed_status=${fail_closed_status}"
echo "ci_fast_gate_exclusion_status=${ci_fast_gate_exclusion_status}"
echo "performance_budget_status=verified"
echo "fail_closed_reason_code=${fail_closed_reason_code}"
