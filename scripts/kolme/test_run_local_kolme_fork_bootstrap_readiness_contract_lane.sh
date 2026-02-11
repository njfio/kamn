#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_bootstrap_readiness_policy.py"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_bootstrap_readiness_contract_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_ERR="$(mktemp)"
SERVER_PID=""
trap 'rm -rf "$TMP_DIR"; rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_ERR"; if [ -n "$SERVER_PID" ]; then kill "$SERVER_PID" >/dev/null 2>&1 || true; wait "$SERVER_PID" 2>/dev/null || true; fi' EXIT

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
  echo "expected local Kolme fork bootstrap/readiness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local Kolme fork bootstrap/readiness policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local Kolme fork bootstrap/readiness contract lane to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_bootstrap_readiness_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local Kolme fork bootstrap/readiness runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_bootstrap_readiness_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local Kolme fork bootstrap/readiness policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_bootstrap_readiness_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local Kolme fork bootstrap/readiness contract lane" >&2
  exit 1
fi

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

FORK_CHAIN_VERSION="v0.15.2"

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --checkout-path "$CHECKOUT_PATH" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run bootstrap/readiness lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget marker"

checker_dry_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code dry_run_no_commands_executed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_dry_run_output" "status")" "ok" "expected checker GO decision for dry-run report"

python3 - "$TMP_REPORT" "$TMP_DIR/policy-negative-report.json" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

source_path = pathlib.Path(sys.argv[1])
target_path = pathlib.Path(sys.argv[2])
report = json.loads(source_path.read_text(encoding="utf-8"))
report["schema_version"] = "kamn.kolme.invalid-schema.v1"
target_path.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_DIR/policy-negative-report.json" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY" >"$TMP_ERR" 2>&1
checker_negative_code=$?
set -e

if [ "$checker_negative_code" -eq 0 ]; then
  echo "expected checker to fail when schema version drifts" >&2
  exit 1
fi

if ! grep -q "schema_version_mismatch" "$TMP_ERR"; then
  echo "expected schema_version_mismatch reason marker for policy negative case" >&2
  exit 1
fi

set +e
bash "$RUNNER" \
  --mode run \
  --checkout-path "$CHECKOUT_PATH" \
  --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
  --expected-ref "refs/heads/main" \
  --fork-chain-version "$FORK_CHAIN_VERSION" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for bootstrap/readiness lane" >&2
  exit 1
fi

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
            payload = {"first_block": 1, "last_block": 3}
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
python3 "$TMP_DIR/mock_kolme_api.py" "$PORT" "$FORK_CHAIN_VERSION" >"$TMP_DIR/server.log" 2>&1 &
SERVER_PID="$!"

for _ in $(seq 1 40); do
  if curl --silent --show-error --fail "http://127.0.0.1:${PORT}/healthz" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
      --expected-ref "refs/heads/main" \
      --base-url "http://127.0.0.1:${PORT}" \
      --fork-chain-version "$FORK_CHAIN_VERSION" \
      --max-seconds 60 \
      --probe-max-seconds 20 \
      --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run mode bootstrap/readiness lane to pass"
assert_eq "$(extract_value "$run_output" "lane_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "bootstrap_readiness_passed" "expected run-mode success reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

checker_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code bootstrap_readiness_passed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_run_output" "status")" "ok" "expected checker GO decision for run report"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-fork-bootstrap-readiness-summary.v1":
    raise SystemExit("unexpected local fork bootstrap/readiness summary schema")
if report.get("mode") != "run":
    raise SystemExit("expected run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status in run summary")
checks = report.get("checks")
if not isinstance(checks, list):
    raise SystemExit("expected check list in run summary")
for expected_id in ("fork_metadata_sync", "api_probe"):
    matching = [entry for entry in checks if isinstance(entry, dict) and entry.get("id") == expected_id]
    if not matching:
        raise SystemExit(f"missing check id: {expected_id}")
    if matching[0].get("status") != "pass":
        raise SystemExit(f"expected pass status for check id: {expected_id}")
PY

contract_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$TMP_REPORT" \
    --policy-output-json "$TMP_POLICY" \
    --max-seconds 120 \
    --fork-chain-version "$FORK_CHAIN_VERSION"
)"

if ! printf '%s\n' "$contract_output" | grep -q "local Kolme fork bootstrap/readiness contract lane tests passed."; then
  echo "expected local fork bootstrap/readiness contract lane success marker" >&2
  exit 1
fi

echo "local Kolme fork bootstrap/readiness contract lane tests passed."
