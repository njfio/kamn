#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_bootstrap_readiness_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"

OUTPUT_JSON="/tmp/kolme-local-fork-bootstrap-readiness-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-fork-bootstrap-readiness-policy.json"
MAX_SECONDS="${KAMN_KOLME_LOCAL_FORK_BOOTSTRAP_READINESS_MAX_SECONDS:-120}"
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
Usage: run_local_kolme_fork_bootstrap_readiness_contract_lane.sh [options]

Options:
  --output-json <path>         Bootstrap/readiness summary output.
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
  echo "expected local Kolme fork bootstrap/readiness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local Kolme fork bootstrap/readiness policy checker to be executable" >&2
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
            payload = {"first_block": 42, "last_block": 55}
            body = json.dumps(payload, sort_keys=True).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
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


HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
PY

CHECKOUT_PATH="$TMP_DIR/kolme_fork"
mkdir -p "$CHECKOUT_PATH"
git -C "$CHECKOUT_PATH" init -q
git -C "$CHECKOUT_PATH" checkout -q -b main
git -C "$CHECKOUT_PATH" config user.email "ci@example.com"
git -C "$CHECKOUT_PATH" config user.name "CI Runner"
cat >"$CHECKOUT_PATH/README.md" <<'EOF'
local fork bootstrap readiness fixture
EOF
git -C "$CHECKOUT_PATH" add README.md
git -C "$CHECKOUT_PATH" commit -q -m "init bootstrap readiness fixture"
git -C "$CHECKOUT_PATH" remote add origin "https://github.com/njfio/kolme_fork.git"

start_epoch="$(date +%s)"

bash "$RUNNER" \
  --mode dry-run \
  --checkout-path "$CHECKOUT_PATH" \
  --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
  --expected-ref "refs/heads/main" \
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
    --checkout-path "$CHECKOUT_PATH" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --base-url "http://127.0.0.1:${PORT}" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --max-seconds "$MAX_SECONDS" \
    --probe-max-seconds 20 \
    --output-json "$OUTPUT_JSON" \
    >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code bootstrap_readiness_passed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

if ! grep -q "run_local_kolme_fork_bootstrap_readiness_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork bootstrap/readiness runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_bootstrap_readiness_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork bootstrap/readiness policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_bootstrap_readiness_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork bootstrap/readiness contract lane" >&2
  exit 1
fi

if ! grep -q "Regression: #1488" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local fork bootstrap/readiness regression marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "local Kolme fork bootstrap/readiness contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "local Kolme fork bootstrap/readiness contract lane tests passed."
