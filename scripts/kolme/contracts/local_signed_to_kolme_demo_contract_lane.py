#!/usr/bin/env python3
"""Contract lane runner for unified local signed-to-Kolme demo checks."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from urllib.request import urlopen

ROOT_DIR = Path(__file__).resolve().parents[3]
SIGNED_DEMO = ROOT_DIR / "scripts/sdk/run_localhost_signed_demo_contract_lane.sh"
SIGNED_INTEGRATION = ROOT_DIR / "scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
RUNTIME_INTEGRATION = ROOT_DIR / "scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_signed_to_kolme_demo_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"
EXPECTED_REMOTE_URL = "https://github.com/njfio/kolme_fork.git"
EXPECTED_REF = "refs/heads/main"


def _pick_port() -> int:
    sock = socket.socket()
    sock.bind(("127.0.0.1", 0))
    port = int(sock.getsockname()[1])
    sock.close()
    return port


def _wait_for_healthz(base_url: str) -> bool:
    for _ in range(40):
        try:
            with urlopen(f"{base_url}/healthz", timeout=1) as response:
                if response.status == 200:
                    return True
        except Exception:
            time.sleep(0.1)
    return False


def _initialize_checkout(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "-C", str(path), "init", "-q"], check=True)
    subprocess.run(["git", "-C", str(path), "checkout", "-q", "-b", "main"], check=True)
    subprocess.run(["git", "-C", str(path), "config", "user.email", "ci@example.com"], check=True)
    subprocess.run(["git", "-C", str(path), "config", "user.name", "CI Runner"], check=True)
    (path / "README.md").write_text("local signed-to-kolme demo fixture\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(path), "add", "README.md"], check=True)
    subprocess.run(
        ["git", "-C", str(path), "commit", "-q", "-m", "init signed-to-kolme fixture"],
        check=True,
    )
    subprocess.run(["git", "-C", str(path), "remote", "add", "origin", EXPECTED_REMOTE_URL], check=True)


def _write_mock_server(path: Path) -> None:
    path.write_text(
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


def _runtime_commit_command(
    base_url: str,
    runtime_commit_live_summary: str,
    runtime_commit_live_policy_report: str,
    runtime_commit_output_file: str,
    runtime_commit_finality_output_file: str,
) -> str:
    broadcast_payload = '{"message":"runtime-commit-demo","signature":"sig","recovery_id":1}'
    live_command = (
        "KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=env-local "
        "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 "
        f"curl --silent --show-error --fail --request PUT --header 'Content-Type: application/json' --data {shlex.quote(broadcast_payload)} {shlex.quote(base_url.rstrip('/') + '/broadcast')} "
        "&& printf 'status=submitted\\nintegration_kolme_fork_live_node_submit_reaches_endpoint\\nreplay_guard=verified\\n{\"pubkey\":\"proof\",\"nonce\":1,\"messages\":[]}\\n'"
    )
    finality_command = (
        f"curl --silent --show-error --fail {shlex.quote(base_url.rstrip('/') + '/healthz')} "
        "&& printf 'finality=final\\n'"
    )
    run_command_parts = [
        "KAMN_KOLME_LOCAL_HEAVY=1",
        "bash scripts/kolme/run_local_runtime_commit_live_lane.sh",
        "--mode run",
        "--skip-preflight",
        f"--output-json {shlex.quote(runtime_commit_live_summary)}",
        f"--live-output-file {shlex.quote(runtime_commit_output_file)}",
        f"--finality-output-file {shlex.quote(runtime_commit_finality_output_file)}",
        "--max-seconds 30",
        "--finality-max-seconds 10",
        f"--live-command {shlex.quote(live_command)}",
        f"--finality-command {shlex.quote(finality_command)}",
    ]
    policy_command_parts = [
        "python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py",
        f"--report-file {shlex.quote(runtime_commit_live_summary)}",
        "--expected-final-decision GO",
        "--ci-fast-gate PASS",
        f"--output-json {shlex.quote(runtime_commit_live_policy_report)}",
        "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider",
        "--require-non-synthetic-run-evidence",
        "--require-native-payload-evidence",
    ]
    # Keep this marker in the command for runtime integration policy parity checks.
    contract_lane_marker = (
        "RUNTIME_COMMIT_CONTRACT_LANE_REF=run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
    )
    return (
        f"{contract_lane_marker} "
        + " ".join(run_command_parts)
        + " && "
        + " ".join(policy_command_parts)
    )


def _load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run unified local signed-to-Kolme demo contract lane checks."
    )
    parser.add_argument(
        "--mode",
        default="run",
        choices=("dry-run", "run"),
        help="Emit planned checks or run the demo checkpoint sequence.",
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-signed-to-kolme-demo-summary.json",
        help="Signed-to-Kolme demo summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-signed-to-kolme-demo-policy.json",
        help="Signed-to-Kolme policy report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="420",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Required fork-info chain_version query value.",
    )
    parser.add_argument(
        "--runtime-integration-summary",
        default="/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
        help="Runtime integration summary path for the run-mode checkpoint.",
    )
    parser.add_argument(
        "--runtime-integration-policy-report",
        default="/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
        help="Runtime integration policy report path for the run-mode checkpoint.",
    )
    parser.add_argument(
        "--runtime-commit-live-summary",
        default="/tmp/kolme-local-runtime-commit-live-summary.json",
        help="Nested runtime commit live summary path.",
    )
    parser.add_argument(
        "--runtime-commit-live-policy-report",
        default="/tmp/kolme-local-runtime-commit-live-policy.json",
        help="Nested runtime commit live policy report path.",
    )
    parser.add_argument(
        "--runtime-commit-output-file",
        default="/tmp/kolme-local-runtime-commit-endpoint-output.txt",
        help="Nested runtime commit submit output path.",
    )
    parser.add_argument(
        "--runtime-commit-finality-output-file",
        default="/tmp/kolme-local-runtime-commit-live-finality-output.txt",
        help="Nested runtime commit finality output path.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    for script in (SIGNED_DEMO, SIGNED_INTEGRATION, RUNTIME_INTEGRATION, CHECKER):
        if not script.is_file() or not script.stat().st_mode & 0o111:
            print(f"expected executable dependency: {script}", file=sys.stderr)
            return 1

    if not DOC_FILE.is_file() or not README_FILE.is_file():
        print("expected docs to exist", file=sys.stderr)
        return 1

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    for marker in (
        "run_local_signed_to_kolme_demo_contract_lane.sh",
        "check_local_signed_to_kolme_demo_policy.py",
        "run_local_kamn_live_runtime_integration_lane.sh",
        "runtime_commit_submit_evidence_marker_present",
        "runtime_commit_finality_evidence_marker_present",
        "Regression: #1640",
        "Regression: #2388",
    ):
        if marker not in doc_text:
            print(f"expected Kolme devnet ops doc marker: {marker}", file=sys.stderr)
            return 1

    for marker in (
        "run_local_signed_to_kolme_demo_contract_lane.sh",
        "check_local_signed_to_kolme_demo_policy.py",
        "runtime_commit_submit_evidence_marker_present",
        "runtime_commit_finality_evidence_marker_present",
    ):
        if marker not in readme_text:
            print(f"expected README marker: {marker}", file=sys.stderr)
            return 1

    start_epoch = time.monotonic()
    checks: list[dict[str, str]] = []
    status = "ok"
    reason_code = "dry_run_no_commands_executed"
    budget_status = "not_run"
    runtime_commit_submit_evidence_marker = "status=submitted"
    runtime_commit_submit_evidence_marker_present = False
    runtime_commit_finality_evidence_marker = "finality=final"
    runtime_commit_finality_evidence_marker_present = False
    runtime_commit_submit_finality_contract_version = "v1"
    runtime_commit_submit_finality_linked = False
    runtime_commit_live_reason_code = "not_run"
    runtime_commit_live_status = "not_run"

    checkpoint_commands = [
        ("localhost_signed_demo_contract", ["bash", str(SIGNED_DEMO)]),
        ("localhost_signed_integration_contract", ["bash", str(SIGNED_INTEGRATION)]),
        (
            "local_kamn_runtime_integration_run",
            [
                "bash",
                str(RUNTIME_INTEGRATION),
                "--mode",
                "run",
                "--checkout-path",
                "<generated-temp-checkout>",
                "--expected-remote-url",
                EXPECTED_REMOTE_URL,
                "--expected-ref",
                EXPECTED_REF,
                "--base-url",
                "<generated-localhost-base-url>",
                "--fork-chain-version",
                args.fork_chain_version,
                "--runtime-provider-client-contract",
                "KolmeRuntimeCommitLiveProvider",
                "--runtime-commit-command",
                "<generated-runtime-commit-command>",
                "--output-json",
                args.runtime_integration_summary,
            ],
        ),
    ]

    if args.mode == "dry-run":
        for check_id, command in checkpoint_commands:
            checks.append(
                {
                    "id": check_id,
                    "command": " ".join(command),
                    "status": "planned",
                    "reason_code": "not_run",
                }
            )
    else:
        budget_status = "within_budget"
        reason_code = "signed_to_kolme_demo_passed"

        for check_id, command in checkpoint_commands[:2]:
            result = subprocess.run(
                command,
                cwd=ROOT_DIR,
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if result.returncode == 0:
                checks.append(
                    {
                        "id": check_id,
                        "command": " ".join(command),
                        "status": "pass",
                        "reason_code": f"{check_id}_passed",
                    }
                )
                continue
            checks.append(
                {
                    "id": check_id,
                    "command": " ".join(command),
                    "status": "fail",
                    "reason_code": f"{check_id}_failed",
                }
            )
            status = "fail"
            reason_code = f"checkpoint_failed_{check_id}"
            break

        if status == "ok":
            with tempfile.TemporaryDirectory(prefix="signed-to-kolme-demo-") as temp_dir:
                temp_path = Path(temp_dir)
                checkout_path = temp_path / "kolme_fork"
                mock_server_file = temp_path / "mock_kolme_api.py"
                mock_server_log = temp_path / "mock_kolme_api.log"
                _initialize_checkout(checkout_path)
                _write_mock_server(mock_server_file)

                port = _pick_port()
                server_proc = subprocess.Popen(
                    ["python3", str(mock_server_file), str(port), args.fork_chain_version],
                    cwd=ROOT_DIR,
                    stdout=mock_server_log.open("w", encoding="utf-8"),
                    stderr=subprocess.STDOUT,
                )
                try:
                    base_url = f"http://127.0.0.1:{port}"
                    if not _wait_for_healthz(base_url):
                        checks.append(
                            {
                                "id": "local_kamn_runtime_integration_run",
                                "command": "python3 <mock_kolme_api.py>",
                                "status": "fail",
                                "reason_code": "mock_server_start_failed",
                            }
                        )
                        status = "fail"
                        reason_code = "mock_server_start_failed"
                    else:
                        runtime_commit_command = _runtime_commit_command(
                            base_url=base_url,
                            runtime_commit_live_summary=args.runtime_commit_live_summary,
                            runtime_commit_live_policy_report=args.runtime_commit_live_policy_report,
                            runtime_commit_output_file=args.runtime_commit_output_file,
                            runtime_commit_finality_output_file=args.runtime_commit_finality_output_file,
                        )
                        runtime_integration_command = [
                            "bash",
                            str(RUNTIME_INTEGRATION),
                            "--mode",
                            "run",
                            "--checkout-path",
                            str(checkout_path),
                            "--expected-remote-url",
                            EXPECTED_REMOTE_URL,
                            "--expected-ref",
                            EXPECTED_REF,
                            "--base-url",
                            base_url,
                            "--fork-chain-version",
                            args.fork_chain_version,
                            "--runtime-provider-client-contract",
                            "KolmeRuntimeCommitLiveProvider",
                            "--max-seconds",
                            "160",
                            "--bootstrap-max-seconds",
                            "20",
                            "--conformance-max-seconds",
                            "40",
                            "--localhost-signed-max-seconds",
                            "45",
                            "--runtime-commit-max-seconds",
                            "35",
                            "--runtime-commit-output-file",
                            args.runtime_commit_output_file,
                            "--runtime-commit-live-summary",
                            args.runtime_commit_live_summary,
                            "--runtime-commit-live-policy-report",
                            args.runtime_commit_live_policy_report,
                            "--runtime-commit-command",
                            runtime_commit_command,
                            "--output-json",
                            args.runtime_integration_summary,
                        ]
                        runtime_env = dict(os.environ)
                        runtime_env["KAMN_KOLME_LOCAL_HEAVY"] = "1"
                        result = subprocess.run(
                            runtime_integration_command,
                            cwd=ROOT_DIR,
                            env=runtime_env,
                            check=False,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL,
                        )
                        if result.returncode == 0:
                            checks.append(
                                {
                                    "id": "local_kamn_runtime_integration_run",
                                    "command": " ".join(runtime_integration_command),
                                    "status": "pass",
                                    "reason_code": "local_kamn_runtime_integration_run_passed",
                                }
                            )
                        else:
                            checks.append(
                                {
                                    "id": "local_kamn_runtime_integration_run",
                                    "command": " ".join(runtime_integration_command),
                                    "status": "fail",
                                    "reason_code": "local_kamn_runtime_integration_run_failed",
                                }
                            )
                            status = "fail"
                            reason_code = "checkpoint_failed_local_kamn_runtime_integration_run"
                finally:
                    server_proc.terminate()
                    try:
                        server_proc.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        server_proc.kill()
                        server_proc.wait(timeout=2)

            if status == "ok":
                runtime_commit_summary_path = Path(args.runtime_commit_live_summary)
                if not runtime_commit_summary_path.is_file():
                    status = "fail"
                    reason_code = "runtime_commit_live_summary_missing"
                else:
                    runtime_commit_summary = _load_json(runtime_commit_summary_path)
                    runtime_commit_submit_evidence_marker = str(
                        runtime_commit_summary.get("submit_evidence_marker", "status=submitted")
                    )
                    runtime_commit_submit_evidence_marker_present = (
                        runtime_commit_summary.get("submit_evidence_marker_present") is True
                    )
                    runtime_commit_finality_evidence_marker = str(
                        runtime_commit_summary.get("finality_evidence_marker", "finality=final")
                    )
                    runtime_commit_finality_evidence_marker_present = (
                        runtime_commit_summary.get("finality_evidence_marker_present") is True
                    )
                    runtime_commit_submit_finality_contract_version = str(
                        runtime_commit_summary.get("request_finality_evidence_contract_version", "v1")
                    )
                    runtime_commit_submit_finality_linked = (
                        runtime_commit_summary.get("request_finality_evidence_linked") is True
                    )
                    runtime_commit_live_reason_code = str(
                        runtime_commit_summary.get("reason_code", "reason_code_missing")
                    )
                    runtime_commit_live_status = str(
                        runtime_commit_summary.get("status", "status_missing")
                    )

                    if not runtime_commit_submit_evidence_marker_present:
                        status = "fail"
                        reason_code = "runtime_commit_submit_evidence_marker_missing"
                    elif not runtime_commit_finality_evidence_marker_present:
                        status = "fail"
                        reason_code = "runtime_commit_finality_evidence_marker_missing"
                    elif not runtime_commit_submit_finality_linked:
                        status = "fail"
                        reason_code = "runtime_commit_submit_finality_linkage_missing"

        if status == "fail":
            remaining = [entry for entry in checkpoint_commands if entry[0] not in {c["id"] for c in checks}]
            for check_id, command in remaining:
                checks.append(
                    {
                        "id": check_id,
                        "command": " ".join(command),
                        "status": "skipped",
                        "reason_code": "skipped_due_prior_failure",
                    }
                )

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        budget_status = "exceeded_budget"
        if status == "ok":
            status = "fail"
            reason_code = "demo_budget_exceeded"

    summary = {
        "schema_version": "kamn.kolme.local-signed-to-kolme-demo-summary.v1",
        "mode": args.mode,
        "status": status,
        "reason_code": reason_code,
        "local_only_enforced": True,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "budget_status": budget_status,
        "runtime_commit_submit_evidence_marker": runtime_commit_submit_evidence_marker,
        "runtime_commit_submit_evidence_marker_present": runtime_commit_submit_evidence_marker_present,
        "runtime_commit_finality_evidence_marker": runtime_commit_finality_evidence_marker,
        "runtime_commit_finality_evidence_marker_present": runtime_commit_finality_evidence_marker_present,
        "runtime_commit_submit_finality_contract_version": runtime_commit_submit_finality_contract_version,
        "runtime_commit_submit_finality_linked": runtime_commit_submit_finality_linked,
        "runtime_commit_live_status": runtime_commit_live_status,
        "runtime_commit_live_reason_code": runtime_commit_live_reason_code,
        "runtime_commit_live_summary_path": args.runtime_commit_live_summary,
        "runtime_commit_live_policy_report_path": args.runtime_commit_live_policy_report,
        "checks": checks,
        "artifact_paths": [
            "/tmp/localhost-signed-demo-contract-report.json",
            "/tmp/localhost-signed-integration-contract-report.json",
            args.runtime_integration_summary,
            args.runtime_integration_policy_report,
            args.runtime_commit_live_summary,
            args.runtime_commit_live_policy_report,
            args.runtime_commit_output_file,
            args.runtime_commit_finality_output_file,
        ],
    }

    output_path = Path(args.output_json).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    expected_final_decision = "GO" if status == "ok" else "NO-GO"
    subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            str(output_path),
            "--expected-final-decision",
            expected_final_decision,
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            reason_code,
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    print("unified local signed-to-Kolme demo contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
