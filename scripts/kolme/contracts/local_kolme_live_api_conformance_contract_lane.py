#!/usr/bin/env python3
"""Contract lane runner for local Kolme live API conformance checks."""

from __future__ import annotations

import argparse
import os
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from urllib.request import urlopen

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_live_api_conformance_harness.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_live_api_conformance_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local Kolme live API conformance contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-live-api-conformance-summary.json",
        help="Conformance harness summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-live-api-conformance-policy.json",
        help="Policy checker report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="180",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Required fork-info chain_version query value.",
    )
    parser.add_argument(
        "--matrix-file",
        default=str(ROOT_DIR / "fixtures/kolme_commit/local_live_api_conformance_matrix.json"),
        help="Conformance matrix fixture path.",
    )
    return parser


def pick_port() -> int:
    sock = socket.socket()
    sock.bind(("127.0.0.1", 0))
    port = int(sock.getsockname()[1])
    sock.close()
    return port


def wait_for_healthz(base_url: str) -> bool:
    for _ in range(40):
        try:
            with urlopen(f"{base_url}/healthz", timeout=1) as response:
                if response.status == 200:
                    return True
        except Exception:
            time.sleep(0.1)
    return False


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    matrix_file = Path(args.matrix_file)

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local Kolme live API conformance harness runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local Kolme live API conformance policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not matrix_file.is_file():
        print("expected local live API conformance matrix fixture to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="kolme-live-api-conformance-") as temp_dir:
        temp_path = Path(temp_dir)
        mock_server_file = temp_path / "mock_kolme_api.py"
        server_log = temp_path / "server.log"

        mock_server_file.write_text(
            """from __future__ import annotations

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
""",
            encoding="utf-8",
        )

        subprocess.run(
            [
                "bash",
                str(RUNNER),
                "--mode",
                "dry-run",
                "--fork-chain-version",
                args.fork_chain_version,
                "--matrix-file",
                str(matrix_file),
                "--output-json",
                args.output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                args.output_json,
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "dry_run_no_commands_executed",
                "--output-json",
                args.policy_output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        port = pick_port()
        server_proc = subprocess.Popen(
            ["python3", str(mock_server_file), str(port), args.fork_chain_version],
            cwd=ROOT_DIR,
            stdout=server_log.open("w", encoding="utf-8"),
            stderr=subprocess.STDOUT,
        )
        try:
            base_url = f"http://127.0.0.1:{port}"
            if not wait_for_healthz(base_url):
                print("mock Kolme API server failed to start", file=sys.stderr)
                return 1

            run_env = dict(os.environ)
            run_env["KAMN_KOLME_LOCAL_HEAVY"] = "1"
            subprocess.run(
                [
                    "bash",
                    str(RUNNER),
                    "--mode",
                    "run",
                    "--base-url",
                    base_url,
                    "--fork-chain-version",
                    args.fork_chain_version,
                    "--matrix-file",
                    str(matrix_file),
                    "--max-seconds",
                    str(max_seconds),
                    "--probe-max-seconds",
                    "20",
                    "--native-max-seconds",
                    "40",
                    "--output-json",
                    args.output_json,
                ],
                cwd=ROOT_DIR,
                env=run_env,
                check=True,
                stdout=subprocess.DEVNULL,
            )
        finally:
            server_proc.terminate()
            try:
                server_proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                server_proc.kill()
                server_proc.wait(timeout=2)

        subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                args.output_json,
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "live_api_conformance_passed",
                "--output-json",
                args.policy_output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    if "run_local_kolme_live_api_conformance_harness.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local live API conformance harness runner", file=sys.stderr)
        return 1
    if "check_local_kolme_live_api_conformance_policy.py" not in doc_text:
        print("expected Kolme devnet ops doc to reference local live API conformance policy checker", file=sys.stderr)
        return 1
    if "run_local_kolme_live_api_conformance_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local live API conformance contract lane", file=sys.stderr)
        return 1
    if "fixtures/kolme_commit/local_live_api_conformance_matrix.json" not in doc_text:
        print("expected Kolme devnet ops doc to reference local live API conformance matrix fixture", file=sys.stderr)
        return 1
    if "Regression: #1483" not in doc_text:
        print("expected Kolme devnet ops doc to include local live API conformance regression marker", file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(f"local Kolme live API conformance contract lane exceeded runtime budget: {elapsed_seconds}s", file=sys.stderr)
        return 1

    print("local Kolme live API conformance contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
