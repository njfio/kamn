#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_api_conformance_harness.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_live_api_conformance_policy.py"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
MATRIX_FILE="$ROOT_DIR/fixtures/kolme_commit/local_live_api_conformance_matrix.json"
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
  echo "expected local Kolme live API conformance harness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/assert_local_heavy_opt_in.sh" "$RUNNER"; then
  echo "expected local Kolme live API conformance runner to invoke shared local-heavy opt-in guard helper" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local Kolme live API conformance policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local Kolme live API conformance contract lane to be executable" >&2
  exit 1
fi

if [ ! -f "$MATRIX_FILE" ]; then
  echo "expected local live API conformance matrix fixture to exist" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_api_conformance_harness.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local Kolme live API conformance harness runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_api_conformance_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local Kolme live API conformance policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_api_conformance_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local Kolme live API conformance contract lane" >&2
  exit 1
fi

if ! grep -q "fixtures/kolme_commit/local_live_api_conformance_matrix.json" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local live API conformance matrix fixture" >&2
  exit 1
fi

FORK_CHAIN_VERSION="v0.15.2"

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --matrix-file "$MATRIX_FILE" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run live conformance harness to pass"
assert_eq "$(extract_value "$dry_run_output" "harness_mode")" "dry-run" "expected dry-run harness mode marker"
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
report["contracts"]["broadcast_method"] = "POST"
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
  echo "expected checker to fail when broadcast_method contract drifts from PUT" >&2
  exit 1
fi

if ! grep -q "broadcast_method_mismatch" "$TMP_ERR"; then
  echo "expected broadcast_method_mismatch reason marker for policy negative case" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-live-api-conformance-summary.v1":
    raise SystemExit("unexpected local live API conformance summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in summary")
contracts = report.get("contracts")
if not isinstance(contracts, dict):
    raise SystemExit("expected contracts object in summary")
if contracts.get("fork_info_query_key") != "chain_version":
    raise SystemExit("expected fork_info_query_key=chain_version")
if contracts.get("broadcast_method") != "PUT":
    raise SystemExit("expected broadcast_method=PUT")
checks = report.get("checks")
if not isinstance(checks, list) or len(checks) < 2:
    raise SystemExit("expected deterministic check list")
check_ids = [entry.get("id") for entry in checks if isinstance(entry, dict)]
if "api_probe" not in check_ids:
    raise SystemExit("expected api_probe check id")
if "native_api_parity" not in check_ids:
    raise SystemExit("expected native_api_parity check id")
matrix = report.get("conformance_matrix")
if not isinstance(matrix, dict):
    raise SystemExit("expected conformance_matrix object in summary")
if matrix.get("schema_version") != "kamn.kolme.local-live-api-conformance-matrix.v1":
    raise SystemExit("expected conformance_matrix schema marker")
matrix_checks = matrix.get("checks")
if not isinstance(matrix_checks, list) or len(matrix_checks) < 2:
    raise SystemExit("expected deterministic conformance matrix checks")
matrix_ids = [entry.get("id") for entry in matrix_checks if isinstance(entry, dict)]
if "api_probe" not in matrix_ids:
    raise SystemExit("expected api_probe matrix check id")
if "native_api_parity" not in matrix_ids:
    raise SystemExit("expected native_api_parity matrix check id")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --matrix-file "$MATRIX_FILE" \
  --fork-chain-version "$FORK_CHAIN_VERSION" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for live conformance harness" >&2
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
            payload = {"first_block": 5, "last_block": 8}
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
            body = b"9"
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
    --matrix-file "$MATRIX_FILE" \
    --base-url "http://127.0.0.1:${PORT}" \
    --fork-chain-version "$FORK_CHAIN_VERSION" \
    --max-seconds 60 \
      --probe-max-seconds 20 \
      --native-max-seconds 40 \
      --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run mode live conformance harness to pass"
assert_eq "$(extract_value "$run_output" "harness_mode")" "run" "expected run mode harness marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "live_api_conformance_passed" "expected run-mode success reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

checker_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code live_api_conformance_passed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_run_output" "status")" "ok" "expected checker GO decision for live conformance run report"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("mode") != "run":
    raise SystemExit("expected run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status for run summary")
if report.get("reason_code") != "live_api_conformance_passed":
    raise SystemExit("expected live_api_conformance_passed reason code")
if report.get("fork_chain_version") != "v0.15.2":
    raise SystemExit("expected deterministic fork_chain_version in run summary")
if report.get("max_seconds") != 60:
    raise SystemExit("expected max_seconds=60 in run summary")
checks = report.get("checks")
if not isinstance(checks, list):
    raise SystemExit("expected check list in run summary")
for expected_id in ("api_probe", "native_api_parity"):
    matching = [entry for entry in checks if isinstance(entry, dict) and entry.get("id") == expected_id]
    if not matching:
        raise SystemExit(f"missing check id: {expected_id}")
    if matching[0].get("status") != "pass":
        raise SystemExit(f"expected pass status for check id: {expected_id}")
matrix = report.get("conformance_matrix")
if not isinstance(matrix, dict):
    raise SystemExit("expected conformance_matrix object in run summary")
matrix_checks = matrix.get("checks")
if not isinstance(matrix_checks, list):
    raise SystemExit("expected conformance matrix checks in run summary")
for expected_id in ("api_probe", "native_api_parity"):
    matching = [
        entry for entry in matrix_checks if isinstance(entry, dict) and entry.get("id") == expected_id
    ]
    if not matching:
        raise SystemExit(f"missing conformance matrix check id: {expected_id}")
    if matching[0].get("ci_scope") != "local-only":
        raise SystemExit(f"expected local-only ci_scope for matrix check id: {expected_id}")
PY

contract_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$TMP_REPORT" \
    --policy-output-json "$TMP_POLICY" \
    --matrix-file "$MATRIX_FILE" \
    --max-seconds 120 \
    --fork-chain-version "$FORK_CHAIN_VERSION"
)"

if ! printf '%s\n' "$contract_output" | grep -q "local Kolme live API conformance contract lane tests passed."; then
  echo "expected local Kolme live API conformance contract lane success marker" >&2
  exit 1
fi

echo "local Kolme live API conformance contract lane tests passed."
