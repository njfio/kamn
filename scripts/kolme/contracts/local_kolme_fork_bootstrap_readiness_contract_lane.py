#!/usr/bin/env python3
"""Contract lane runner for local Kolme fork bootstrap/readiness checks."""

from __future__ import annotations

import argparse
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from urllib.request import urlopen

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_bootstrap_readiness_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local Kolme fork bootstrap/readiness contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-fork-bootstrap-readiness-summary.json",
        help="Bootstrap/readiness summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-fork-bootstrap-readiness-policy.json",
        help="Policy checker report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="120",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Required fork-info chain_version query value.",
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

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local Kolme fork bootstrap/readiness runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local Kolme fork bootstrap/readiness policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="kolme-bootstrap-readiness-") as temp_dir:
        temp_path = Path(temp_dir)
        mock_server_file = temp_path / "mock_kolme_api.py"
        checkout_path = temp_path / "kolme_fork"
        server_log = temp_path / "server.log"
        checkout_path.mkdir(parents=True, exist_ok=True)

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
""",
            encoding="utf-8",
        )

        subprocess.run(["git", "-C", str(checkout_path), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.name", "CI Runner"], check=True)
        (checkout_path / "README.md").write_text(
            "local fork bootstrap readiness fixture\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "-C", str(checkout_path), "add", "README.md"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "commit", "-q", "-m", "init bootstrap readiness fixture"], check=True)
        subprocess.run(
            ["git", "-C", str(checkout_path), "remote", "add", "origin", "https://github.com/njfio/kolme_fork.git"],
            check=True,
        )

        subprocess.run(
            [
                "bash",
                str(RUNNER),
                "--mode",
                "dry-run",
                "--checkout-path",
                str(checkout_path),
                "--expected-remote-url",
                "https://github.com/njfio/kolme_fork.git",
                "--expected-ref",
                "refs/heads/main",
                "--fork-chain-version",
                args.fork_chain_version,
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
                    "--checkout-path",
                    str(checkout_path),
                    "--expected-remote-url",
                    "https://github.com/njfio/kolme_fork.git",
                    "--expected-ref",
                    "refs/heads/main",
                    "--base-url",
                    base_url,
                    "--fork-chain-version",
                    args.fork_chain_version,
                    "--max-seconds",
                    str(max_seconds),
                    "--probe-max-seconds",
                    "20",
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
                "bootstrap_readiness_passed",
                "--output-json",
                args.policy_output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    if "run_local_kolme_fork_bootstrap_readiness_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork bootstrap/readiness runner", file=sys.stderr)
        return 1
    if "check_local_kolme_fork_bootstrap_readiness_policy.py" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork bootstrap/readiness policy checker", file=sys.stderr)
        return 1
    if "run_local_kolme_fork_bootstrap_readiness_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork bootstrap/readiness contract lane", file=sys.stderr)
        return 1
    if "Regression: #1488" not in doc_text:
        print("expected Kolme devnet ops doc to include local fork bootstrap/readiness regression marker", file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(f"local Kolme fork bootstrap/readiness contract lane exceeded runtime budget: {elapsed_seconds}s", file=sys.stderr)
        return 1

    print("local Kolme fork bootstrap/readiness contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
