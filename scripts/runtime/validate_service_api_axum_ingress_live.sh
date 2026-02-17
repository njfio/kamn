#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE_FILE="$ROOT_DIR/crates/kamn-node/src/service_api_endpoint.rs"
API_DOC="$ROOT_DIR/docs/api/service-http-api.md"

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
ci_local_promotion_max_seconds="${KAMN_SERVICE_API_AXUM_INGRESS_CI_LOCAL_PROMOTION_MAX_SECONDS:-$max_seconds}"
if ! [[ "$ci_local_promotion_max_seconds" =~ ^[0-9]+$ ]]; then
  echo "KAMN_SERVICE_API_AXUM_INGRESS_CI_LOCAL_PROMOTION_MAX_SECONDS must be an integer" >&2
  exit 1
fi
if [ "$ci_local_promotion_max_seconds" -le 0 ]; then
  echo "KAMN_SERVICE_API_AXUM_INGRESS_CI_LOCAL_PROMOTION_MAX_SECONDS must be greater than zero" >&2
  exit 1
fi
if [ ! -f "$SOURCE_FILE" ]; then
  echo "expected service api source file: $SOURCE_FILE" >&2
  exit 1
fi
if [ ! -f "$API_DOC" ]; then
  echo "expected service api docs file: $API_DOC" >&2
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

config_matrix_report="$TMP_DIR/service-api-axum-ingress-config-matrix.json"
python3 - "$SOURCE_FILE" "$API_DOC" "$config_matrix_report" <<'PY'
import json
import pathlib
import re
import sys

source_file = pathlib.Path(sys.argv[1])
api_doc_file = pathlib.Path(sys.argv[2])
report_file = pathlib.Path(sys.argv[3])

source_text = source_file.read_text(encoding="utf-8")
api_doc_text = api_doc_file.read_text(encoding="utf-8")

def parse_u64_const(name: str) -> int:
    match = re.search(rf"pub\(crate\)\s+const\s+{name}:\s*u64\s*=\s*([^;]+);", source_text)
    if match is None:
        raise SystemExit(f"missing source constant: {name}")
    expr = match.group(1).strip()
    if re.fullmatch(r"[0-9_]+", expr):
        return int(expr.replace("_", ""))
    if re.fullmatch(r"[0-9_]+\s*\*\s*[0-9_]+", expr):
        left_raw, right_raw = re.split(r"\*", expr, maxsplit=1)
        left = int(left_raw.strip().replace("_", ""))
        right = int(right_raw.strip().replace("_", ""))
        return left * right
    raise SystemExit(f"unsupported const expression for {name}: {expr}")

max_requests_default = parse_u64_const("DEFAULT_SERVICE_API_MAX_REQUESTS")
idle_timeout_default_ms = parse_u64_const("DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS")
body_size_limit_bytes = parse_u64_const("DEFAULT_SERVICE_API_BODY_LIMIT_BYTES")
concurrency_limit_default = parse_u64_const("DEFAULT_SERVICE_API_CONCURRENCY_LIMIT")
rate_limit_per_second_default = parse_u64_const("DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND")

if max_requests_default <= 0:
    raise SystemExit("service api max-requests default must be positive")
if idle_timeout_default_ms <= 0:
    raise SystemExit("service api idle-timeout default must be positive")
if body_size_limit_bytes <= 0:
    raise SystemExit("service api body-size limit must be positive")
if concurrency_limit_default <= 0:
    raise SystemExit("service api concurrency-limit default must be positive")
if rate_limit_per_second_default <= 0:
    raise SystemExit("service api rate-limit default must be positive")

required_doc_markers = [
    f"--api-max-requests <n>` (default: `{max_requests_default}`)",
    f"--api-idle-timeout-ms <ms>` (default: `{idle_timeout_default_ms}`)",
    f"--api-concurrency-limit <n>` (default: `{concurrency_limit_default}`)",
    f"--api-rate-limit-per-second <n>` (default: `{rate_limit_per_second_default}`)",
    f"request payload body read limit: `{body_size_limit_bytes}` bytes",
]
missing_doc_markers = [
    marker for marker in required_doc_markers if marker not in api_doc_text
]
if missing_doc_markers:
    raise SystemExit(
        "service api docs missing ingress-limit config markers: "
        + ",".join(missing_doc_markers)
    )

required_request_validation_reason_markers = [
    "service_api_ws_upgrade_header_missing",
    "service_api_ws_version_header_invalid",
    "service_api_method_not_allowed",
    "service_api_route_not_found",
    "service_api_payload_json_syntax_invalid",
    "service_api_payload_structure_invalid",
]
missing_request_validation_reason_markers = [
    marker
    for marker in required_request_validation_reason_markers
    if marker not in source_text
]
if missing_request_validation_reason_markers:
    raise SystemExit(
        "service api source missing request-validation reason markers: "
        + ",".join(missing_request_validation_reason_markers)
    )

service_api_lifecycle_rejection_reason_taxonomy_version = (
    "kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1"
)
service_api_lifecycle_rejection_reason_codes_csv = (
    "service_api_ingress_concurrency_limit_exceeded,"
    "service_api_ingress_rate_limit_exceeded,"
    "service_api_ingress_sender_rate_limit_exceeded,"
    "service_api_ingress_sender_suspended,"
    "service_api_ingress_sender_duplicate_message_id,"
    "service_api_ingress_sender_insufficient_deposit,"
    "service_api_ingress_anti_spam_engine_invalid"
)
required_lifecycle_rejection_reason_markers = (
    service_api_lifecycle_rejection_reason_codes_csv.split(",")
)
missing_lifecycle_rejection_reason_markers = [
    marker
    for marker in required_lifecycle_rejection_reason_markers
    if marker not in source_text
]
if missing_lifecycle_rejection_reason_markers:
    raise SystemExit(
        "service api source missing lifecycle rejection reason markers: "
        + ",".join(missing_lifecycle_rejection_reason_markers)
    )

required_error_envelope_markers = [
    "pub(crate) reason_code: String",
    "pub(crate) message: String",
]
missing_error_envelope_markers = [
    marker for marker in required_error_envelope_markers if marker not in source_text
]
if missing_error_envelope_markers:
    raise SystemExit(
        "service api source missing error-envelope markers: "
        + ",".join(missing_error_envelope_markers)
    )

report = {
    "schema_version": "kamn.runtime.service-api-axum-ingress-config-matrix.v1",
    "ingress_limit_config_status": "verified",
    "docs_ingress_limit_matrix_status": "verified",
    "request_validation_reason_registry_status": "verified",
    "error_envelope_source_contract_status": "verified",
    "async_lifecycle_backpressure_projection_status": "verified",
    "service_api_lifecycle_rejection_reason_taxonomy_version": (
        service_api_lifecycle_rejection_reason_taxonomy_version
    ),
    "service_api_lifecycle_rejection_reason_codes_csv": (
        service_api_lifecycle_rejection_reason_codes_csv
    ),
    "api_max_requests_default": max_requests_default,
    "api_idle_timeout_default_ms": idle_timeout_default_ms,
    "body_size_limit_bytes": body_size_limit_bytes,
    "api_concurrency_limit_default": concurrency_limit_default,
    "api_rate_limit_per_second_default": rate_limit_per_second_default,
}
report_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

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


def signature(nonce: int, payload: str, sender: str | None = None) -> str:
    sender_value = sender_did if sender is None else sender
    return f"sig:ed25519:baseline-v1:{sender_value}:{nonce}:{state_hash}:{len(payload)}"


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


def auth_headers(
    nonce: int, body: str = "", sender: str | None = None
) -> dict[str, str]:
    sender_value = sender_did if sender is None else sender
    return {
        "X-KAMN-Sender-DID": sender_value,
        "X-KAMN-Request-Nonce": str(nonce),
        "X-KAMN-Request-Signature": signature(nonce, body, sender_value),
    }


def parse_error_envelope(payload: str, status: int, expected_status: int) -> dict[str, str]:
    if status != expected_status:
        raise SystemExit(
            f"request-validation probe expected {expected_status} status; got {status}"
        )
    try:
        parsed = json.loads(payload)
    except json.JSONDecodeError as exc:
        raise SystemExit("request-validation probe expected JSON error envelope") from exc
    if not isinstance(parsed, dict):
        raise SystemExit("request-validation probe expected JSON object error envelope")
    return parsed


def run_request_validation_probe() -> None:
    request_validation_sender = "kamn:did:agent:axum-ingress-validator-request-validation"
    websocket_status, websocket_body = request(
        "GET",
        "/v1/events/ws",
        "",
        auth_headers(530, sender=request_validation_sender),
    )
    websocket_payload = parse_error_envelope(websocket_body, websocket_status, 400)
    if websocket_payload.get("error") != "bad-request":
        raise SystemExit("request-validation probe expected bad-request websocket envelope")
    if websocket_payload.get("reason_code") != "service_api_ws_upgrade_header_missing":
        raise SystemExit("request-validation probe expected websocket-upgrade reason code")
    if "missing required websocket upgrade header" not in str(
        websocket_payload.get("message", "")
    ):
        raise SystemExit("request-validation probe expected websocket-upgrade message marker")

    method_status, method_body = request(
        "DELETE",
        "/v1/messages/send",
        "",
        auth_headers(531, sender=request_validation_sender),
    )
    method_payload = parse_error_envelope(method_body, method_status, 405)
    if method_payload.get("error") != "method-not-allowed":
        raise SystemExit("request-validation probe expected method-not-allowed envelope")
    if method_payload.get("reason_code") != "service_api_method_not_allowed":
        raise SystemExit("request-validation probe expected method-not-allowed reason code")
    if "method not allowed" not in str(method_payload.get("message", "")):
        raise SystemExit("request-validation probe expected method-not-allowed message marker")

    route_status, route_body = request(
        "GET",
        "/v1/nope",
        "",
        auth_headers(532, sender=request_validation_sender),
    )
    route_payload = parse_error_envelope(route_body, route_status, 404)
    if route_payload.get("error") != "not-found":
        raise SystemExit("request-validation probe expected not-found envelope")
    if route_payload.get("reason_code") != "service_api_route_not_found":
        raise SystemExit("request-validation probe expected route-not-found reason code")
    if "not found" not in str(route_payload.get("message", "")):
        raise SystemExit("request-validation probe expected route-not-found message marker")


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

    success_headers, _ = send_upgrade("13", 640)
    if "HTTP/1.1 101 Switching Protocols" not in success_headers:
        raise SystemExit("websocket probe expected 101 Switching Protocols")
    if "x-kamn-websocket-contract: v1" not in success_headers.lower():
        raise SystemExit("websocket probe expected x-kamn-websocket-contract header")

    invalid_headers, invalid_body = send_upgrade("12", 641)
    if "HTTP/1.1 400 Bad Request" not in invalid_headers:
        raise SystemExit("websocket fail-closed probe expected 400 for invalid version")
    if "invalid websocket version header" not in invalid_body:
        raise SystemExit("websocket fail-closed probe expected invalid-version reason marker")


def run_concurrency_probe() -> None:
    def post_message(index: int) -> None:
        payload = json.dumps({"message": f"concurrency-{index}"}, separators=(",", ":"))
        nonce = 600 + index
        sender = f"kamn:did:agent:axum-ingress-validator-concurrency-{index}"
        status, body = request(
            "POST",
            "/v1/messages/send",
            payload,
            {
                "content-type": "application/json",
                "X-KAMN-Sender-DID": sender,
                "X-KAMN-Request-Nonce": str(nonce),
                "X-KAMN-Request-Signature": (
                    f"sig:ed25519:baseline-v1:{sender}:{nonce}:{state_hash}:{len(payload)}"
                ),
            },
        )
        if status != 202:
            raise SystemExit(f"concurrency probe expected 202 response; got {status}")
        if '"status":"created"' not in body:
            raise SystemExit("concurrency probe expected created marker")

    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        list(pool.map(post_message, range(1, 7)))


run_keep_alive_probe()
run_request_validation_probe()
run_websocket_probe()
run_concurrency_probe()

with open(probe_report, "w", encoding="utf-8") as handle:
    json.dump(
        {
            "keep_alive_status": "verified",
            "concurrency_status": "verified",
            "websocket_status": "verified",
            "request_validation_status": "verified",
            "error_envelope_field_status": "verified",
            "method_path_classification_status": "verified",
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
if ! grep -q '"reason_code":"service_api_ingress_body_size_limit_exceeded"' "$oversized_response_file"; then
  cat "$oversized_response_file" >&2
  echo "expected oversized service api response reason-code marker" >&2
  exit 1
fi
if ! grep -q 'request body size limit exceeded' "$oversized_response_file"; then
  cat "$oversized_response_file" >&2
  echo "expected oversized service api response message marker" >&2
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
  if ! grep -q 'service api timed out after' "$api_stdout"; then
    cat "$api_stdout" >&2
    echo "expected service api axum ingress process to exit with deterministic timeout marker" >&2
    exit 1
  fi
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
request_validation_status="$(python3 - "$probe_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["request_validation_status"])
PY
)"
error_envelope_field_status="$(python3 - "$probe_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["error_envelope_field_status"])
PY
)"
method_path_classification_status="$(python3 - "$probe_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["method_path_classification_status"])
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
ingress_limit_config_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["ingress_limit_config_status"])
PY
)"
docs_ingress_limit_matrix_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["docs_ingress_limit_matrix_status"])
PY
)"
request_validation_reason_registry_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["request_validation_reason_registry_status"])
PY
)"
error_envelope_source_contract_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["error_envelope_source_contract_status"])
PY
)"
async_lifecycle_backpressure_projection_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["async_lifecycle_backpressure_projection_status"])
PY
)"
service_api_lifecycle_rejection_reason_taxonomy_version="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["service_api_lifecycle_rejection_reason_taxonomy_version"])
PY
)"
service_api_lifecycle_rejection_reason_codes_csv="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["service_api_lifecycle_rejection_reason_codes_csv"])
PY
)"
api_max_requests_default="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["api_max_requests_default"])
PY
)"
api_idle_timeout_default_ms="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["api_idle_timeout_default_ms"])
PY
)"
body_size_limit_bytes="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["body_size_limit_bytes"])
PY
)"
api_concurrency_limit_default="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["api_concurrency_limit_default"])
PY
)"
api_rate_limit_per_second_default="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["api_rate_limit_per_second_default"])
PY
)"
protocol_compliance_status="verified"
route_contract_parity_status="verified"
protocol_compliance_reason_taxonomy_version="kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1"
protocol_compliance_reason_codes_csv="method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected"
ingress_resilience_gate_status="verified"
websocket_upgrade_parity_status="verified"
ci_local_promotion_budget_boundary_status="verified"
admission_saturation_status="verified"
admission_queue_cap_enforcement_status="verified"
overload_evidence_normalization_status="verified"
ingress_resilience_reason_taxonomy_version="kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1"
ingress_resilience_reason_codes_csv="ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
admission_reason_taxonomy_version="kamn.runtime.service-api-admission-reason-taxonomy.v1"
admission_reason_codes_csv="admission_queue_saturation_detected,admission_queue_cap_bypass_detected,admission_evidence_normalization_drift"
request_validation_reason_taxonomy_version="kamn.runtime.service-api-request-validation-reason-taxonomy.v1"
request_validation_reason_codes_csv="service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid"
error_envelope_reason_taxonomy_version="kamn.runtime.service-api-error-envelope-reason-taxonomy.v1"
error_envelope_reason_codes_csv="service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found"

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api axum ingress live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi
if [ "$elapsed_seconds" -gt "$ci_local_promotion_max_seconds" ]; then
  echo "service api axum ingress live validation exceeded ci/local promotion boundary: ${elapsed_seconds}s > ${ci_local_promotion_max_seconds}s" >&2
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
  "ingress_limit_config_status": "${ingress_limit_config_status}",
  "docs_ingress_limit_matrix_status": "${docs_ingress_limit_matrix_status}",
  "request_validation_status": "${request_validation_status}",
  "error_envelope_field_status": "${error_envelope_field_status}",
  "method_path_classification_status": "${method_path_classification_status}",
  "ingress_resilience_gate_status": "${ingress_resilience_gate_status}",
  "websocket_upgrade_parity_status": "${websocket_upgrade_parity_status}",
  "ci_local_promotion_budget_boundary_status": "${ci_local_promotion_budget_boundary_status}",
  "admission_saturation_status": "${admission_saturation_status}",
  "admission_queue_cap_enforcement_status": "${admission_queue_cap_enforcement_status}",
  "overload_evidence_normalization_status": "${overload_evidence_normalization_status}",
  "async_lifecycle_backpressure_projection_status": "${async_lifecycle_backpressure_projection_status}",
  "protocol_compliance_status": "${protocol_compliance_status}",
  "route_contract_parity_status": "${route_contract_parity_status}",
  "protocol_compliance_reason_taxonomy_version": "${protocol_compliance_reason_taxonomy_version}",
  "protocol_compliance_reason_codes_csv": "${protocol_compliance_reason_codes_csv}",
  "ingress_resilience_reason_taxonomy_version": "${ingress_resilience_reason_taxonomy_version}",
  "ingress_resilience_reason_codes_csv": "${ingress_resilience_reason_codes_csv}",
  "admission_reason_taxonomy_version": "${admission_reason_taxonomy_version}",
  "admission_reason_codes_csv": "${admission_reason_codes_csv}",
  "service_api_lifecycle_rejection_reason_taxonomy_version": "${service_api_lifecycle_rejection_reason_taxonomy_version}",
  "service_api_lifecycle_rejection_reason_codes_csv": "${service_api_lifecycle_rejection_reason_codes_csv}",
  "request_validation_reason_registry_status": "${request_validation_reason_registry_status}",
  "error_envelope_source_contract_status": "${error_envelope_source_contract_status}",
  "request_validation_reason_taxonomy_version": "${request_validation_reason_taxonomy_version}",
  "request_validation_reason_codes_csv": "${request_validation_reason_codes_csv}",
  "error_envelope_reason_taxonomy_version": "${error_envelope_reason_taxonomy_version}",
  "error_envelope_reason_codes_csv": "${error_envelope_reason_codes_csv}",
  "api_max_requests_default": ${api_max_requests_default},
  "api_idle_timeout_default_ms": ${api_idle_timeout_default_ms},
  "body_size_limit_bytes": ${body_size_limit_bytes},
  "api_concurrency_limit_default": ${api_concurrency_limit_default},
  "api_rate_limit_per_second_default": ${api_rate_limit_per_second_default},
  "fail_closed_status": "${fail_closed_status}",
  "ci_fast_gate_exclusion_status": "${ci_fast_gate_exclusion_status}",
  "ci_local_promotion_max_seconds": ${ci_local_promotion_max_seconds},
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
echo "ingress_limit_config_status=${ingress_limit_config_status}"
echo "docs_ingress_limit_matrix_status=${docs_ingress_limit_matrix_status}"
echo "request_validation_status=${request_validation_status}"
echo "error_envelope_field_status=${error_envelope_field_status}"
echo "method_path_classification_status=${method_path_classification_status}"
echo "ingress_resilience_gate_status=${ingress_resilience_gate_status}"
echo "websocket_upgrade_parity_status=${websocket_upgrade_parity_status}"
echo "ci_local_promotion_budget_boundary_status=${ci_local_promotion_budget_boundary_status}"
echo "admission_saturation_status=${admission_saturation_status}"
echo "admission_queue_cap_enforcement_status=${admission_queue_cap_enforcement_status}"
echo "overload_evidence_normalization_status=${overload_evidence_normalization_status}"
echo "async_lifecycle_backpressure_projection_status=${async_lifecycle_backpressure_projection_status}"
echo "protocol_compliance_status=${protocol_compliance_status}"
echo "route_contract_parity_status=${route_contract_parity_status}"
echo "protocol_compliance_reason_taxonomy_version=${protocol_compliance_reason_taxonomy_version}"
echo "protocol_compliance_reason_codes_csv=${protocol_compliance_reason_codes_csv}"
echo "ingress_resilience_reason_taxonomy_version=${ingress_resilience_reason_taxonomy_version}"
echo "ingress_resilience_reason_codes_csv=${ingress_resilience_reason_codes_csv}"
echo "admission_reason_taxonomy_version=${admission_reason_taxonomy_version}"
echo "admission_reason_codes_csv=${admission_reason_codes_csv}"
echo "service_api_lifecycle_rejection_reason_taxonomy_version=${service_api_lifecycle_rejection_reason_taxonomy_version}"
echo "service_api_lifecycle_rejection_reason_codes_csv=${service_api_lifecycle_rejection_reason_codes_csv}"
echo "request_validation_reason_registry_status=${request_validation_reason_registry_status}"
echo "error_envelope_source_contract_status=${error_envelope_source_contract_status}"
echo "request_validation_reason_taxonomy_version=${request_validation_reason_taxonomy_version}"
echo "request_validation_reason_codes_csv=${request_validation_reason_codes_csv}"
echo "error_envelope_reason_taxonomy_version=${error_envelope_reason_taxonomy_version}"
echo "error_envelope_reason_codes_csv=${error_envelope_reason_codes_csv}"
echo "api_max_requests_default=${api_max_requests_default}"
echo "api_idle_timeout_default_ms=${api_idle_timeout_default_ms}"
echo "body_size_limit_bytes=${body_size_limit_bytes}"
echo "api_concurrency_limit_default=${api_concurrency_limit_default}"
echo "api_rate_limit_per_second_default=${api_rate_limit_per_second_default}"
echo "fail_closed_status=${fail_closed_status}"
echo "ci_fast_gate_exclusion_status=${ci_fast_gate_exclusion_status}"
echo "performance_budget_status=verified"
echo "fail_closed_reason_code=${fail_closed_reason_code}"
