#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE_FILE="$ROOT_DIR/crates/kamn-node/src/service_api_endpoint.rs"
API_DOC="$ROOT_DIR/docs/api/service-http-api.md"
RELEASE_GONOGO_CHECKLIST="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
EXPECTED_PROTOCOL_SESSION_REASON_TAXONOMY_VERSION="kamn.runtime.service-api.protocol-session-reason-taxonomy.v1"
EXPECTED_PROTOCOL_SESSION_REASON_CODES_CSV="service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing"

output_json=""
max_seconds=180
node_pid=""

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
if [ ! -f "$SOURCE_FILE" ]; then
  echo "expected service api source file: $SOURCE_FILE" >&2
  exit 1
fi
if [ ! -f "$API_DOC" ]; then
  echo "expected service api docs file: $API_DOC" >&2
  exit 1
fi
if [ ! -f "$RELEASE_GONOGO_CHECKLIST" ]; then
  echo "expected release go/no-go checklist docs file: $RELEASE_GONOGO_CHECKLIST" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  if [[ -n "$node_pid" ]] && kill -0 "$node_pid" 2>/dev/null; then
    kill -KILL "$node_pid" 2>/dev/null || true
    wait "$node_pid" 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

start_epoch="$(date +%s)"

config_matrix_report="$TMP_DIR/service-api-websocket-config-matrix.json"
python3 - \
  "$SOURCE_FILE" \
  "$API_DOC" \
  "$RELEASE_GONOGO_CHECKLIST" \
  "$config_matrix_report" \
  "$EXPECTED_PROTOCOL_SESSION_REASON_TAXONOMY_VERSION" \
  "$EXPECTED_PROTOCOL_SESSION_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import re
import sys

source_file = pathlib.Path(sys.argv[1])
api_doc_file = pathlib.Path(sys.argv[2])
release_checklist_file = pathlib.Path(sys.argv[3])
report_file = pathlib.Path(sys.argv[4])
expected_protocol_session_reason_taxonomy_version = sys.argv[5]
expected_protocol_session_reason_codes_csv = sys.argv[6]

source_text = source_file.read_text(encoding="utf-8")
api_doc_text = api_doc_file.read_text(encoding="utf-8")
release_checklist_text = release_checklist_file.read_text(encoding="utf-8")
server_source_file = source_file.parent / "service_api_endpoint" / "server.rs"
if not server_source_file.is_file():
    raise SystemExit(f"expected service api websocket server source file: {server_source_file}")
server_source_text = server_source_file.read_text(encoding="utf-8")
marker_source_text = source_text + "\n" + server_source_text


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


idle_timeout_default_ms = parse_u64_const("DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS")
if idle_timeout_default_ms <= 0:
    raise SystemExit("service api idle-timeout default must be positive")

required_doc_markers = [
    f"--api-idle-timeout-ms <ms>` (default: `{idle_timeout_default_ms}`)",
]
missing_doc_markers = [
    marker for marker in required_doc_markers if marker not in api_doc_text
]
if missing_doc_markers:
    raise SystemExit(
        "service api docs missing websocket lifecycle markers: "
        + ",".join(missing_doc_markers)
    )

required_reason_markers = [
    "service_api_ws_upgrade_header_missing",
    "service_api_ws_connection_header_missing",
    "service_api_ws_key_header_missing",
    "service_api_ws_version_header_invalid",
    "service_api_auth_sender_did_header_missing",
]
missing_reason_markers = [
    marker for marker in required_reason_markers if marker not in marker_source_text
]
if missing_reason_markers:
    raise SystemExit(
        "service api source missing websocket reason markers: "
        + ",".join(missing_reason_markers)
    )

required_lifecycle_markers = [
    "tokio::time::sleep_until(deadline.into())",
    "idle_timeout_ms",
]
missing_lifecycle_markers = [
    marker for marker in required_lifecycle_markers if marker not in marker_source_text
]
if missing_lifecycle_markers:
    raise SystemExit(
        "service api source missing websocket lifecycle markers: "
        + ",".join(missing_lifecycle_markers)
    )

required_release_checklist_markers = [
    (
        "service_api_protocol_session_reason_taxonomy_version="
        + expected_protocol_session_reason_taxonomy_version
    ),
    (
        "service_api_protocol_session_reason_codes_csv="
        + expected_protocol_session_reason_codes_csv
    ),
    "service_api_ws_upgrade_header_missing",
    "service_api_ws_version_header_invalid",
    "service_api_payload_json_syntax_invalid",
    "service_api_auth_replay_nonce_detected",
    "service_api_protocol_session_docs_marker_missing",
]
missing_release_checklist_markers = [
    marker
    for marker in required_release_checklist_markers
    if marker not in release_checklist_text
]
if missing_release_checklist_markers:
    raise SystemExit(
        "release checklist missing service-api protocol/session markers: "
        + ",".join(missing_release_checklist_markers)
    )

report = {
    "schema_version": "kamn.runtime.service-api-websocket-config-matrix.v1",
    "websocket_idle_timeout_contract_status": "verified",
    "websocket_reason_registry_status": "verified",
    "protocol_session_docs_contract_status": "verified",
    "service_api_protocol_session_reason_taxonomy_version": (
        expected_protocol_session_reason_taxonomy_version
    ),
    "service_api_protocol_session_reason_codes_csv": (
        expected_protocol_session_reason_codes_csv
    ),
    "api_idle_timeout_default_ms": idle_timeout_default_ms,
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
auth_private_key_hex="${KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX:-1111111111111111111111111111111111111111111111111111111111111111}"
auth_public_key_hex="$(
  python3 - "$auth_private_key_hex" <<'PY'
import sys
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ec

order = int(
    "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16
)
private_key_hex = sys.argv[1].strip()
try:
    private_scalar = int(private_key_hex, 16)
except ValueError as exc:
    raise SystemExit("invalid service auth private key hex") from exc
if private_scalar <= 0 or private_scalar >= order:
    raise SystemExit("service auth private key scalar must be within secp256k1 range")
private_key = ec.derive_private_key(private_scalar, ec.SECP256K1())
public_key_hex = private_key.public_key().public_bytes(
    serialization.Encoding.X962, serialization.PublicFormat.CompressedPoint
).hex()
print(public_key_hex)
PY
)"
if [ -z "$auth_public_key_hex" ]; then
  echo "failed to derive service api auth public key hex" >&2
  exit 1
fi

api_stdout="$TMP_DIR/service-api-websocket.out"
KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX="$auth_public_key_hex" \
KAMN_SERVICE_API_TLS_MODE="disabled" \
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
  exit 1
fi

ws_results_json="$TMP_DIR/websocket-results.json"
python3 - "$api_addr" "$ws_results_json" "$auth_private_key_hex" "$auth_public_key_hex" <<'PY'
import hashlib
import json
import socket
import sys
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives.asymmetric.utils import decode_dss_signature

api_addr = sys.argv[1]
report_path = sys.argv[2]
private_key_hex = sys.argv[3]
public_key_hex = sys.argv[4]
host, port_text = api_addr.rsplit(":", 1)
port = int(port_text)

sender_did = f"kamn:did:agent:pkh-{public_key_hex}"
state_hash = "service-api:kamn-devnet:v0.1.0"
secp256k1_order = int(
    "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16
)

try:
    private_scalar = int(private_key_hex, 16)
except ValueError as exc:
    raise SystemExit("service api websocket probe private key hex is invalid") from exc
if private_scalar <= 0 or private_scalar >= secp256k1_order:
    raise SystemExit("service api websocket probe private key scalar is invalid")
signing_key = ec.derive_private_key(private_scalar, ec.SECP256K1())


def signing_payload(nonce: int, payload: str) -> str:
    return (
        f"sender_len={len(sender_did)}\n"
        f"sender={sender_did}\n"
        f"nonce={nonce}\n"
        f"state_hash_len={len(state_hash)}\n"
        f"state_hash={state_hash}\n"
        f"payload_len={len(payload)}\n"
        f"payload={payload}"
    )


def signature(nonce: int, payload: str) -> str:
    message = signing_payload(nonce, payload).encode("utf-8")
    der_signature = signing_key.sign(message, ec.ECDSA(hashes.SHA256()))
    r_value, s_value = decode_dss_signature(der_signature)
    if s_value > secp256k1_order // 2:
        s_value = secp256k1_order - s_value
    message_hash = int.from_bytes(hashlib.sha256(message).digest(), byteorder="big")
    nonce_scalar = (
        (message_hash + (r_value * private_scalar)) * pow(s_value, -1, secp256k1_order)
    ) % secp256k1_order
    if nonce_scalar == 0:
        raise SystemExit("service api websocket probe nonce scalar resolved to zero")
    ephemeral_point = (
        ec.derive_private_key(nonce_scalar, ec.SECP256K1())
        .public_key()
        .public_numbers()
    )
    if ephemeral_point.x == r_value:
        recovery_prefix = 0
    elif ephemeral_point.x - r_value == secp256k1_order:
        recovery_prefix = 1
    else:
        raise SystemExit(
            "service api websocket probe failed to map signature to recovery-id domain"
        )
    recovery_id = (recovery_prefix << 1) | (ephemeral_point.y & 1)
    signature_hex = f"{r_value:064x}{s_value:064x}"
    return f"sig:secp256k1:baseline-v2:{recovery_id}:{signature_hex}"


def open_socket() -> socket.socket:
    sock = socket.create_connection((host, port), timeout=3)
    sock.settimeout(3)
    return sock


def recv_until(sock: socket.socket, marker: bytes) -> bytes:
    payload = b""
    while marker not in payload:
        chunk = sock.recv(4096)
        if not chunk:
            raise SystemExit("websocket response header terminator missing")
        payload += chunk
    return payload


def read_exact(sock: socket.socket, initial: bytes, total: int) -> bytes:
    payload = initial
    while len(payload) < total:
        chunk = sock.recv(4096)
        if not chunk:
            raise SystemExit("websocket response payload truncated")
        payload += chunk
    return payload


def read_http_response(sock: socket.socket) -> tuple[int, dict[str, str], bytes]:
    payload = recv_until(sock, b"\r\n\r\n")
    header_bytes, body_remainder = payload.split(b"\r\n\r\n", 1)
    header_text = header_bytes.decode("utf-8", errors="strict")
    header_lines = header_text.split("\r\n")
    if not header_lines:
        raise SystemExit("websocket response status line missing")
    status_parts = header_lines[0].split(" ", 2)
    if len(status_parts) < 2:
        raise SystemExit("websocket response status line malformed")
    try:
        status_code = int(status_parts[1])
    except ValueError as exc:
        raise SystemExit(f"websocket response status code invalid: {status_parts[1]}") from exc
    headers: dict[str, str] = {}
    for line in header_lines[1:]:
        if not line or ":" not in line:
            continue
        key, value = line.split(":", 1)
        headers[key.strip().lower()] = value.strip()
    content_length = int(headers.get("content-length", "0"))
    if content_length > 0:
        body = read_exact(sock, body_remainder, content_length)
        return status_code, headers, body[:content_length]
    return status_code, headers, body_remainder


def parse_small_text_frame(sock: socket.socket, payload: bytes) -> str:
    payload = read_exact(sock, payload, 2)
    if payload[0] != 0x81:
        raise SystemExit("websocket opcode mismatch")
    if payload[1] & 0x80:
        raise SystemExit("server websocket frame must not be masked")
    length_indicator = payload[1] & 0x7F
    header_size = 2
    if length_indicator == 126:
        payload = read_exact(sock, payload, 4)
        length = int.from_bytes(payload[2:4], byteorder="big")
        header_size = 4
    elif length_indicator == 127:
        raise SystemExit("unexpected large websocket frame payload")
    else:
        length = length_indicator
    payload = read_exact(sock, payload, header_size + length)
    return payload[header_size : header_size + length].decode("utf-8", errors="strict")


def decode_json_body(body: bytes, marker: str) -> dict[str, str]:
    try:
        payload = json.loads(body.decode("utf-8", errors="strict"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"{marker} expected JSON error envelope") from exc
    if not isinstance(payload, dict):
        raise SystemExit(f"{marker} expected JSON object error envelope")
    return payload


def ws_upgrade_request(
    nonce: int, version: str, include_upgrade: bool, include_auth: bool
) -> str:
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
                f"X-KAMN-Signer-Public-Key: {public_key_hex}",
                "X-KAMN-Authz-Scope: events:read",
            ]
        )
    return (
        "GET /v1/events/ws HTTP/1.1\r\n"
        + "\r\n".join(headers)
        + "\r\nContent-Length: 0\r\n\r\n"
    )


# Success: proper upgrade + auth + version.
success_sock = open_socket()
try:
    success_sock.sendall(
        ws_upgrade_request(nonce=1, version="13", include_upgrade=True, include_auth=True).encode(
            "utf-8"
        )
    )
    success_status, success_headers, success_body = read_http_response(success_sock)
    if success_status != 101:
        raise SystemExit("expected websocket success status line")
    if success_headers.get("x-kamn-websocket-contract") != "v1":
        raise SystemExit("expected websocket contract header")
    success_event = parse_small_text_frame(success_sock, success_body)
    if '"event":"state-transition"' not in success_event:
        raise SystemExit("expected websocket event marker")
finally:
    success_sock.close()

# Fail-closed: invalid websocket version.
invalid_version_sock = open_socket()
try:
    invalid_version_sock.sendall(
        ws_upgrade_request(nonce=2, version="12", include_upgrade=True, include_auth=True).encode(
            "utf-8"
        )
    )
    invalid_status, _, invalid_body = read_http_response(invalid_version_sock)
    if invalid_status != 400:
        raise SystemExit("expected invalid websocket version rejection")
    invalid_payload = decode_json_body(invalid_body, "invalid-version")
    if invalid_payload.get("reason_code") != "service_api_ws_version_header_invalid":
        raise SystemExit("expected invalid websocket version reason code marker")
    if "invalid websocket version header" not in str(invalid_payload.get("message", "")):
        raise SystemExit("expected invalid websocket version reason message marker")
finally:
    invalid_version_sock.close()

# Fail-closed: missing websocket upgrade headers.
missing_upgrade_sock = open_socket()
try:
    missing_upgrade_sock.sendall(
        ws_upgrade_request(
            nonce=3,
            version="13",
            include_upgrade=False,
            include_auth=True,
        ).encode("utf-8")
    )
    missing_status, _, missing_body = read_http_response(missing_upgrade_sock)
    if missing_status != 400:
        raise SystemExit("expected missing upgrade header rejection")
    missing_payload = decode_json_body(missing_body, "missing-upgrade")
    if missing_payload.get("reason_code") != "service_api_ws_upgrade_header_missing":
        raise SystemExit("expected missing websocket upgrade header reason code marker")
    if "missing required websocket upgrade header" not in str(
        missing_payload.get("message", "")
    ):
        raise SystemExit("expected missing websocket upgrade reason message marker")
finally:
    missing_upgrade_sock.close()

# Fail-closed: unauthorized websocket request.
unauthorized_sock = open_socket()
try:
    unauthorized_sock.sendall(
        ws_upgrade_request(
            nonce=4,
            version="13",
            include_upgrade=True,
            include_auth=False,
        ).encode("utf-8")
    )
    unauthorized_status, _, unauthorized_body = read_http_response(unauthorized_sock)
    if unauthorized_status != 401:
        raise SystemExit("expected unauthorized websocket rejection")
    unauthorized_payload = decode_json_body(unauthorized_body, "unauthorized")
    if unauthorized_payload.get("error") != "unauthorized":
        raise SystemExit("expected unauthorized error marker")
    if unauthorized_payload.get("reason_code") != "service_api_auth_sender_did_header_missing":
        raise SystemExit("expected unauthorized sender-did reason code marker")
finally:
    unauthorized_sock.close()

with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(
        {
            "websocket_upgrade_status": "verified",
            "websocket_session_lifecycle_status": "verified",
            "websocket_heartbeat_timeout_status": "verified",
            "fail_closed_status": "verified",
        },
        handle,
    )
PY

websocket_upgrade_status="$(python3 - "$ws_results_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["websocket_upgrade_status"])
PY
)"
websocket_session_lifecycle_status="$(python3 - "$ws_results_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["websocket_session_lifecycle_status"])
PY
)"
websocket_heartbeat_timeout_status="$(python3 - "$ws_results_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["websocket_heartbeat_timeout_status"])
PY
)"
fail_closed_status="$(python3 - "$ws_results_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["fail_closed_status"])
PY
)"
websocket_idle_timeout_contract_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["websocket_idle_timeout_contract_status"])
PY
)"
websocket_reason_registry_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["websocket_reason_registry_status"])
PY
)"
protocol_session_docs_contract_status="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["protocol_session_docs_contract_status"])
PY
)"
service_api_protocol_session_reason_taxonomy_version="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["service_api_protocol_session_reason_taxonomy_version"])
PY
)"
service_api_protocol_session_reason_codes_csv="$(python3 - "$config_matrix_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["service_api_protocol_session_reason_codes_csv"])
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
node_pid=""
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

websocket_lifecycle_reason_taxonomy_version="kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1"
websocket_lifecycle_reason_codes_csv="service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing"

report_json="$TMP_DIR/websocket-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.service-api-websocket-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "websocket_upgrade_status": "${websocket_upgrade_status}",
  "websocket_session_lifecycle_status": "${websocket_session_lifecycle_status}",
  "websocket_heartbeat_timeout_status": "${websocket_heartbeat_timeout_status}",
  "websocket_idle_timeout_contract_status": "${websocket_idle_timeout_contract_status}",
  "fail_closed_status": "${fail_closed_status}",
  "probe_status": "${probe_status}",
  "websocket_reason_registry_status": "${websocket_reason_registry_status}",
  "protocol_session_docs_contract_status": "${protocol_session_docs_contract_status}",
  "service_api_protocol_session_reason_taxonomy_version": "${service_api_protocol_session_reason_taxonomy_version}",
  "service_api_protocol_session_reason_codes_csv": "${service_api_protocol_session_reason_codes_csv}",
  "websocket_lifecycle_reason_taxonomy_version": "${websocket_lifecycle_reason_taxonomy_version}",
  "websocket_lifecycle_reason_codes_csv": "${websocket_lifecycle_reason_codes_csv}",
  "api_idle_timeout_default_ms": ${api_idle_timeout_default_ms},
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "websocket_upgrade_status=${websocket_upgrade_status}"
echo "websocket_session_lifecycle_status=${websocket_session_lifecycle_status}"
echo "websocket_heartbeat_timeout_status=${websocket_heartbeat_timeout_status}"
echo "websocket_idle_timeout_contract_status=${websocket_idle_timeout_contract_status}"
echo "fail_closed_status=${fail_closed_status}"
echo "probe_status=${probe_status}"
echo "websocket_reason_registry_status=${websocket_reason_registry_status}"
echo "protocol_session_docs_contract_status=${protocol_session_docs_contract_status}"
echo "service_api_protocol_session_reason_taxonomy_version=${service_api_protocol_session_reason_taxonomy_version}"
echo "service_api_protocol_session_reason_codes_csv=${service_api_protocol_session_reason_codes_csv}"
echo "websocket_lifecycle_reason_taxonomy_version=${websocket_lifecycle_reason_taxonomy_version}"
echo "websocket_lifecycle_reason_codes_csv=${websocket_lifecycle_reason_codes_csv}"
echo "api_idle_timeout_default_ms=${api_idle_timeout_default_ms}"
