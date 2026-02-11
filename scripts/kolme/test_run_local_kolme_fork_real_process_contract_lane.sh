#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_real_process_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -rf "$TMP_DIR"; rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_ERR"' EXIT

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
  echo "expected real-fork local process wrapper runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected real-fork local process wrapper policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_real_process_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference real-fork local process wrapper runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_real_process_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference real-fork local process wrapper policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_profile_preflight_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork profile preflight runner" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_self_test_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork self-test runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_self_test_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork self-test policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_real_process_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference real-fork local process wrapper runner" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run real-fork wrapper lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run lane mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "local_only_enforced")" "true" "expected local-only marker"

checker_dry_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code dry_run_no_commands_executed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_dry_run_output" "status")" "ok" "expected checker GO decision for dry-run report"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-fork-real-process-summary.v1":
    raise SystemExit("unexpected wrapper summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok dry-run status")
checks = report.get("checks")
if not isinstance(checks, list):
    raise SystemExit("expected checks list in summary")
for expected_id in (
    "real_fork_command_profile",
    "profile_preflight_lane",
    "profile_preflight_policy",
    "self_test_lane",
    "self_test_policy",
    "process_lifecycle_lane",
    "process_lifecycle_policy",
):
    entries = [entry for entry in checks if isinstance(entry, dict) and entry.get("id") == expected_id]
    if not entries:
        raise SystemExit(f"missing check id: {expected_id}")
    if entries[0].get("status") != "planned":
        raise SystemExit(f"expected planned status for check id: {expected_id}")
PY

set +e
bash "$RUNNER" --mode run --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for wrapper lane" >&2
  exit 1
fi

CHECKOUT_PATH="$TMP_DIR/kolme_fork"
mkdir -p "$CHECKOUT_PATH"
git -C "$CHECKOUT_PATH" init -q
git -C "$CHECKOUT_PATH" checkout -q -b main
git -C "$CHECKOUT_PATH" config user.email "ci@example.com"
git -C "$CHECKOUT_PATH" config user.name "CI Runner"
cat >"$CHECKOUT_PATH/README.md" <<'EOF'
real-fork wrapper fixture checkout
EOF
git -C "$CHECKOUT_PATH" add README.md
git -C "$CHECKOUT_PATH" commit -q -m "init wrapper fixture"
git -C "$CHECKOUT_PATH" remote add origin "https://github.com/njfio/kolme_fork.git"

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
            payload = {"first_block": 10, "last_block": 25}
            body = json.dumps(payload, sort_keys=True).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if parsed.path == "/get-next-nonce":
            payload = {"nonce": 9}
            body = json.dumps(payload, sort_keys=True).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if parsed.path == "/notifications":
            payload = {"items": [{"txhash": "tx-local", "status": "included", "height": 1}]}
            body = json.dumps(payload, sort_keys=True).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if parsed.path == "/block/1":
            payload = {"height": 1, "txs": [{"txhash": "tx-local"}]}
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

        content_length = int(self.headers.get("Content-Length", "0"))
        if content_length > 0:
            _ = self.rfile.read(content_length)
        payload = {"txhash": "tx-local"}
        body = json.dumps(payload, sort_keys=True).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self) -> None:  # noqa: N802
        parsed = urlparse(self.path)
        if parsed.path != "/broadcast/runtime-commit":
            body = b"not found"
            self.send_response(404)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        content_length = int(self.headers.get("Content-Length", "0"))
        if content_length > 0:
            _ = self.rfile.read(content_length)
        payload = {"status": "ok", "commit_id": "wrapper-runtime-commit"}
        body = json.dumps(payload, sort_keys=True).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
PY

PORT="$(pick_port)"
BASE_URL="http://127.0.0.1:${PORT}"
FORK_CHAIN_VERSION="v0.15.2"

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
      --expected-ref "refs/heads/main" \
      --base-url "$BASE_URL" \
      --fork-chain-version "$FORK_CHAIN_VERSION" \
      --serve-command "python3 $TMP_DIR/mock_kolme_api.py $PORT $FORK_CHAIN_VERSION" \
      --allow-non-fork-serve-command \
      --max-seconds 360 \
      --preflight-max-seconds 45 \
      --self-test-max-seconds 60 \
      --self-test-matrix-max-seconds 20 \
      --self-test-matrix-command "printf wrapper_self_test_ok_1" \
      --self-test-matrix-command "printf wrapper_self_test_ok_2" \
      --lifecycle-max-seconds 240 \
      --lifecycle-startup-max-seconds 45 \
      --lifecycle-integration-max-seconds 180 \
      --lifecycle-bootstrap-max-seconds 90 \
      --lifecycle-conformance-max-seconds 120 \
      --lifecycle-runtime-commit-max-seconds 30 \
      --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run mode wrapper lane to pass"
assert_eq "$(extract_value "$run_output" "lane_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "real_fork_process_wrapper_passed" "expected run-mode success reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

checker_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code real_fork_process_wrapper_passed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_run_output" "status")" "ok" "expected checker GO decision for run report"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("mode") != "run":
    raise SystemExit("expected run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok run status")
checks = report.get("checks")
if not isinstance(checks, list):
    raise SystemExit("expected checks list in run summary")
for expected_id in (
    "real_fork_command_profile",
    "profile_preflight_lane",
    "profile_preflight_policy",
    "self_test_lane",
    "self_test_policy",
    "process_lifecycle_lane",
    "process_lifecycle_policy",
):
    entries = [entry for entry in checks if isinstance(entry, dict) and entry.get("id") == expected_id]
    if not entries:
        raise SystemExit(f"missing check id: {expected_id}")
    if entries[0].get("status") != "pass":
        raise SystemExit(f"expected pass status for check id: {expected_id}")
PY

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --checkout-path "$CHECKOUT_PATH" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --base-url "$BASE_URL" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --serve-command "python3 $TMP_DIR/mock_kolme_api.py $PORT $FORK_CHAIN_VERSION" \
    --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_allow_non_fork_code=$?
set -e

if [ "$run_without_allow_non_fork_code" -eq 0 ]; then
  echo "expected non-fork serve-command policy to fail closed without explicit override" >&2
  exit 1
fi

if ! grep -q "must target checkout path and use cargo run" "$TMP_ERR"; then
  echo "expected deterministic non-fork serve-command policy failure message" >&2
  exit 1
fi

echo "real-fork local process wrapper contract lane tests passed."
