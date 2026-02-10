#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_api_conformance_harness.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_live_api_conformance_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"

OUTPUT_JSON="/tmp/kolme-local-live-api-conformance-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-live-api-conformance-policy.json"
MAX_SECONDS="${KAMN_KOLME_LOCAL_LIVE_API_CONFORMANCE_MAX_SECONDS:-180}"
FORK_CHAIN_VERSION="v0.15.2"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --policy-output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --policy-output-json" >&2
        exit 1
      fi
      POLICY_OUTPUT_JSON="$2"
      shift 2
      ;;
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --fork-chain-version)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-chain-version" >&2
        exit 1
      fi
      FORK_CHAIN_VERSION="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_live_api_conformance_contract_lane.sh [options]

Options:
  --output-json <path>         Conformance harness summary output.
  --policy-output-json <path>  Policy checker report output.
  --max-seconds <n>            Total runtime budget in seconds.
  --fork-chain-version <val>   Required fork-info chain_version query value.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$RUNNER" ]; then
  echo "expected local Kolme live API conformance harness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local Kolme live API conformance policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ]; then
  echo "expected Kolme devnet ops documentation to exist" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
SERVER_PID=""
trap 'rm -rf "$TMP_DIR"; if [ -n "$SERVER_PID" ]; then kill "$SERVER_PID" >/dev/null 2>&1 || true; wait "$SERVER_PID" 2>/dev/null || true; fi' EXIT

pick_port() {
  python3 - <<'PY'
import socket

sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

cat >"$TMP_DIR/mock_kolme_api.py" <<'PY'
from __future__ import annotations

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs, urlparse

PORT = int(sys.argv[1])
CHAIN_VERSION = sys.argv[2]


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        return

    def do_GET(self) -> None:  # noqa: N802
        parsed = urlparse(self.path)
        query = parse_qs(parsed.query)
        if parsed.path == "/healthz":
            body = b"Healthy!"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if parsed.path == "/fork-info":
            versions = query.get("chain_version", [])
            if versions != [CHAIN_VERSION]:
                body = b"invalid chain_version"
                self.send_response(400)
                self.send_header("Content-Type", "text/plain; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            payload = {"first_block": 100, "last_block": 120}
            body = json.dumps(payload, sort_keys=True).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if parsed.path == "/get-next-nonce":
            pubkeys = query.get("pubkey", [])
            if not pubkeys or not pubkeys[0]:
                body = b"missing pubkey"
                self.send_response(400)
                self.send_header("Content-Type", "text/plain; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            body = b"7"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        body = b"not found"
        self.send_response(404)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_PUT(self) -> None:  # noqa: N802
        parsed = urlparse(self.path)
        if parsed.path != "/broadcast":
            body = b"not found"
            self.send_response(404)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length)
        try:
            payload = json.loads(raw.decode("utf-8"))
        except json.JSONDecodeError:
            body = b"invalid json"
            self.send_response(400)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        required = ("message", "signature", "recovery_id")
        if not isinstance(payload, dict) or any(key not in payload for key in required):
            body = b"invalid payload"
            self.send_response(400)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        response = json.dumps({"status": "accepted", "tx_hash": "0xabc"}, sort_keys=True).encode(
            "utf-8"
        )
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)


HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
PY

start_epoch="$(date +%s)"

bash "$RUNNER" \
  --mode dry-run \
  --fork-chain-version "$FORK_CHAIN_VERSION" \
  --output-json "$OUTPUT_JSON" \
  >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

PORT="$(pick_port)"
python3 "$TMP_DIR/mock_kolme_api.py" "$PORT" "$FORK_CHAIN_VERSION" >"$TMP_DIR/server.log" 2>&1 &
SERVER_PID="$!"

for _ in $(seq 1 40); do
  if curl --silent --show-error --fail "http://127.0.0.1:${PORT}/healthz" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --base-url "http://127.0.0.1:${PORT}" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --max-seconds "$MAX_SECONDS" \
    --probe-max-seconds 20 \
    --native-max-seconds 40 \
    --output-json "$OUTPUT_JSON" \
    >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code live_api_conformance_passed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

if ! grep -q "run_local_kolme_live_api_conformance_harness.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local live API conformance harness runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_api_conformance_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local live API conformance policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_api_conformance_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local live API conformance contract lane" >&2
  exit 1
fi

if ! grep -q "Regression: #1483" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local live API conformance regression marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "local Kolme live API conformance contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "local Kolme live API conformance contract lane tests passed."
