#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_api_probe_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$(mktemp)"
TMP_ERR="$(mktemp)"
SERVER_PID=""
trap 'rm -rf "$TMP_DIR"; rm -f "$TMP_REPORT" "$TMP_ERR"; if [ -n "$SERVER_PID" ]; then kill "$SERVER_PID" >/dev/null 2>&1 || true; wait "$SERVER_PID" 2>/dev/null || true; fi' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

pick_port() {
  python3 - <<'PY'
import socket

sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

if [ ! -x "$RUNNER" ]; then
  echo "expected local Kolme API probe runner to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_api_probe_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local Kolme API probe runner" >&2
  exit 1
fi

FORK_CHAIN_VERSION="v0.15.2"

cat >"$TMP_DIR/mock_kolme_api.py" <<'PY'
from __future__ import annotations

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

PORT = int(sys.argv[1])


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        return

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/healthz":
            body = b"Healthy!"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path.startswith("/fork-info?chain_version="):
            payload = {"first_block": 1, "last_block": 2}
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

PORT="$(pick_port)"
python3 "$TMP_DIR/mock_kolme_api.py" "$PORT" >"$TMP_DIR/server.log" 2>&1 &
SERVER_PID="$!"

for _ in $(seq 1 40); do
  if curl --silent --show-error "http://127.0.0.1:${PORT}/healthz" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --base-url "http://127.0.0.1:${PORT}" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run probe lane to pass"
assert_eq "$(extract_value "$dry_run_output" "probe_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget marker"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-api-probe-summary.v1":
    raise SystemExit("unexpected local api probe summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true")
if report.get("fork_chain_version") != "v0.15.2":
    raise SystemExit("expected deterministic fork_chain_version in summary")
checks = report.get("checks")
if not isinstance(checks, list) or len(checks) < 2:
    raise SystemExit("expected deterministic probe checks")
if not any(entry.get("id") == "healthz_endpoint" for entry in checks if isinstance(entry, dict)):
    raise SystemExit("expected healthz_endpoint check in summary")
PY

run_output="$(
  bash "$RUNNER" \
    --mode run \
    --base-url "http://127.0.0.1:${PORT}" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --max-seconds 15 \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run probe lane to pass"
assert_eq "$(extract_value "$run_output" "probe_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "probe_checks_passed" "expected success reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("mode") != "run":
    raise SystemExit("expected run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status for run mode")
if report.get("reason_code") != "probe_checks_passed":
    raise SystemExit("expected probe_checks_passed reason code")
if report.get("fork_chain_version") != "v0.15.2":
    raise SystemExit("expected deterministic fork_chain_version in run summary")
fork_info = report.get("fork_info")
if not isinstance(fork_info, dict):
    raise SystemExit("expected fork_info object in summary")
if fork_info.get("first_block") != 1 or fork_info.get("last_block") != 2:
    raise SystemExit("unexpected fork_info values in summary")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --base-url "http://127.0.0.1:1" \
  --fork-chain-version "$FORK_CHAIN_VERSION" \
  --max-seconds 5 \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
unreachable_code=$?
set -e

if [ "$unreachable_code" -eq 0 ]; then
  echo "expected local API probe run mode to fail for unreachable base URL" >&2
  exit 1
fi

if ! grep -q "reason_code=healthz_request_failed" "$TMP_ERR"; then
  echo "expected healthz_request_failed reason marker for unreachable base URL" >&2
  exit 1
fi

echo "local Kolme API probe lane tests passed."
