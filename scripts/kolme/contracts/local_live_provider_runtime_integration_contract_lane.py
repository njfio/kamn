#!/usr/bin/env python3
"""Bounded localhost integration contract lane for live-provider runtime checks."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT_DIR / "scripts"))

from framework.process_harness import (  # noqa: E402
    ProcessHarness,
    ProcessHarnessError,
    write_evidence_report,
)

RUNNER = ROOT_DIR / "scripts/kolme/run_local_runtime_commit_live_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
DOC_FILES = [
    ROOT_DIR / "docs/planning/kolme-devnet-ops.md",
    ROOT_DIR / "README.md",
]

DOC_MARKERS = [
    "run_local_live_provider_runtime_integration_contract_lane.sh",
    "run_local_runtime_commit_live_lane.sh",
    "check_local_runtime_commit_live_evidence_policy.py",
    "provider_client_contract=KolmeRuntimeCommitLiveProvider",
    "provider_client_contract_mismatch",
    "provider_in_memory_reference_detected",
    "live_preflight_failed",
    "live_preflight_timeout",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run bounded localhost live-provider runtime integration contract lane."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-live-provider-runtime-integration-contract-report.json",
    )
    parser.add_argument("--max-seconds", default="90")
    return parser.parse_args()


def _run(command: list[str], *, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )


def _ensure_docs_markers() -> None:
    missing: list[str] = []
    for doc_file in DOC_FILES:
        if not doc_file.is_file():
            raise RuntimeError(f"expected documentation file to exist: {doc_file}")
        doc_text = doc_file.read_text(encoding="utf-8")
        for marker in DOC_MARKERS:
            if marker not in doc_text:
                missing.append(f"{doc_file.relative_to(ROOT_DIR)}_missing_marker:{marker}")
    if missing:
        raise RuntimeError(",".join(missing))


def main() -> int:
    args = parse_args()
    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1

    if not RUNNER.is_file() or not os.access(RUNNER, os.X_OK):
        print("expected local runtime-commit live lane runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not os.access(CHECKER, os.X_OK):
        print("expected local runtime-commit live evidence policy checker to be executable", file=sys.stderr)
        return 1

    try:
        _ensure_docs_markers()
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    try:
        with tempfile.TemporaryDirectory(prefix="live-provider-runtime-integration-contract-") as temp_dir:
            temp_path = Path(temp_dir)
            mock_server_file = temp_path / "mock_kolme_api.py"
            mock_server_log = temp_path / "mock_kolme_api.log"
            go_summary_file = temp_path / "go_summary.json"
            go_policy_file = temp_path / "go_policy.json"
            provider_mismatch_summary_file = temp_path / "provider_mismatch_summary.json"
            provider_mismatch_policy_file = temp_path / "provider_mismatch_policy.json"
            unavailable_summary_file = temp_path / "unavailable_summary.json"
            unavailable_policy_file = temp_path / "unavailable_policy.json"
            go_live_output = temp_path / "go_live_output.txt"
            go_finality_output = temp_path / "go_finality_output.txt"
            unavailable_live_output = temp_path / "unavailable_live_output.txt"
            unavailable_finality_output = temp_path / "unavailable_finality_output.txt"
            process_harness_evidence_file = temp_path / "process_harness_evidence.json"

            mock_server_file.write_text(
                """#!/usr/bin/env python3
from __future__ import annotations

import http.server
import json
import socketserver
import sys
from urllib.parse import parse_qs, urlparse


class Handler(http.server.BaseHTTPRequestHandler):
    def _send(self, status: int, body: str, content_type: str = "text/plain") -> None:
        encoded = body.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def do_GET(self) -> None:  # noqa: N802
        parsed = urlparse(self.path)
        if parsed.path == "/healthz":
            self._send(200, "ok")
            return
        if parsed.path == "/fork-info":
            params = parse_qs(parsed.query)
            chain_version = params.get("chain_version", [""])[0]
            payload = {"chain_version": chain_version or "v0.15.2", "first_block": 1, "last_block": 1}
            self._send(200, json.dumps(payload, sort_keys=True), "application/json")
            return
        self._send(404, "not found")

    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        return


def main() -> int:
    port = int(sys.argv[1])
    with socketserver.TCPServer(("127.0.0.1", port), Handler) as server:
        server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
""",
                encoding="utf-8",
            )
            mock_server_file.chmod(0o755)

            with ProcessHarness(root_dir=ROOT_DIR) as harness:
                port = harness.reserve_port("mock_kolme_api", start_port=30000, end_port=45000)
                base_url = f"http://127.0.0.1:{port}"
                mock_process = harness.start_process(
                    "mock_kolme_api",
                    ["python3", str(mock_server_file), str(port)],
                    log_file=mock_server_log,
                    release_port_labels=("mock_kolme_api",),
                )

                if not harness.wait_for_http_ready(f"{base_url}/healthz", timeout_seconds=10):
                    raise RuntimeError("mock Kolme API server failed readiness probe")

                go_live_command = (
                    "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 "
                    f"curl --silent --show-error --fail {base_url}/healthz >/dev/null && "
                    "printf 'status=submitted\\nintegration_kolme_fork_live_node_submit_reaches_endpoint\\n"
                    "replay_guard=verified\\n{\"pubkey\":\"proof\",\"nonce\":1,\"messages\":[]}\\n'"
                )
                go_result = _run(
                    [
                        "bash",
                        str(RUNNER),
                        "--mode",
                        "run",
                        "--base-url",
                        base_url,
                        "--provider-hint",
                        "kolme-fork-local",
                        "--max-seconds",
                        "30",
                        "--preflight-max-seconds",
                        "5",
                        "--live-command",
                        go_live_command,
                        "--finality-command",
                        f"curl --silent --show-error --fail {base_url}/healthz >/dev/null && printf 'finality=final\\n'",
                        "--finality-max-seconds",
                        "5",
                        "--finality-retry-max-attempts",
                        "2",
                        "--finality-retry-backoff-seconds",
                        "0",
                        "--output-json",
                        str(go_summary_file),
                        "--live-output-file",
                        str(go_live_output),
                        "--finality-output-file",
                        str(go_finality_output),
                    ],
                    env={**os.environ, "KAMN_KOLME_LOCAL_HEAVY": "1"},
                )
                if go_result.returncode != 0:
                    raise RuntimeError(
                        "expected GO live-provider integration run to succeed: "
                        f"{go_result.stdout}{go_result.stderr}"
                    )

                go_policy_result = _run(
                    [
                        "python3",
                        str(CHECKER),
                        "--report-file",
                        str(go_summary_file),
                        "--expected-final-decision",
                        "GO",
                        "--ci-fast-gate",
                        "PASS",
                        "--expected-provider-client-contract",
                        "KolmeRuntimeCommitLiveProvider",
                        "--require-non-synthetic-run-evidence",
                        "--require-native-payload-evidence",
                        "--output-json",
                        str(go_policy_file),
                    ]
                )
                if go_policy_result.returncode != 0:
                    raise RuntimeError(
                        "expected GO live-provider integration policy check to pass: "
                        f"{go_policy_result.stdout}{go_policy_result.stderr}"
                    )

                go_summary = json.loads(go_summary_file.read_text(encoding="utf-8"))
                if go_summary.get("provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
                    raise RuntimeError("expected provider_client_contract=KolmeRuntimeCommitLiveProvider in GO summary")
                if go_summary.get("provider_in_memory_reference_detected") is not False:
                    raise RuntimeError("expected provider_in_memory_reference_detected=false in GO summary")

                provider_mismatch_summary = dict(go_summary)
                provider_mismatch_summary["provider_client_contract"] = "InMemoryKolmeRuntimeCommitClient"
                provider_mismatch_summary["provider_in_memory_reference_detected"] = True
                provider_mismatch_summary["provider_live_contract_marker"] = (
                    "provider_client_contract=InMemoryKolmeRuntimeCommitClient"
                )
                provider_mismatch_summary["provider_live_contract_marker_present"] = False
                provider_mismatch_summary_file.write_text(
                    json.dumps(provider_mismatch_summary, sort_keys=True, indent=2) + "\n",
                    encoding="utf-8",
                )

                provider_mismatch_result = _run(
                    [
                        "python3",
                        str(CHECKER),
                        "--report-file",
                        str(provider_mismatch_summary_file),
                        "--expected-final-decision",
                        "NO-GO",
                        "--ci-fast-gate",
                        "PASS",
                        "--expected-provider-client-contract",
                        "KolmeRuntimeCommitLiveProvider",
                        "--output-json",
                        str(provider_mismatch_policy_file),
                    ]
                )
                if provider_mismatch_result.returncode == 0:
                    raise RuntimeError("expected provider mismatch policy check to fail closed")
                provider_mismatch_policy = json.loads(
                    provider_mismatch_policy_file.read_text(encoding="utf-8")
                )
                provider_mismatch_reasons = provider_mismatch_policy.get("reason_codes")
                if not isinstance(provider_mismatch_reasons, list):
                    raise RuntimeError("expected reason_codes list in provider mismatch policy output")
                required_provider_reasons = {
                    "provider_client_contract_mismatch",
                    "provider_in_memory_reference_detected",
                }
                if not required_provider_reasons.issubset(set(provider_mismatch_reasons)):
                    raise RuntimeError("missing required provider mismatch fail-closed reason markers")

                unavailable_base_url = f"http://127.0.0.1:{port + 17}"
                unavailable_result = _run(
                    [
                        "bash",
                        str(RUNNER),
                        "--mode",
                        "run",
                        "--base-url",
                        unavailable_base_url,
                        "--provider-hint",
                        "kolme-fork-local",
                        "--max-seconds",
                        "20",
                        "--preflight-max-seconds",
                        "2",
                        "--live-command",
                        go_live_command,
                        "--output-json",
                        str(unavailable_summary_file),
                        "--live-output-file",
                        str(unavailable_live_output),
                        "--finality-output-file",
                        str(unavailable_finality_output),
                    ],
                    env={**os.environ, "KAMN_KOLME_LOCAL_HEAVY": "1"},
                )
                if unavailable_result.returncode == 0:
                    raise RuntimeError("expected unavailable-node integration run to fail closed")
                unavailable_summary = json.loads(unavailable_summary_file.read_text(encoding="utf-8"))
                unavailable_reason_code = unavailable_summary.get("reason_code")
                if unavailable_reason_code not in {"live_preflight_failed", "live_preflight_timeout"}:
                    raise RuntimeError("expected deterministic preflight failure reason for unavailable-node path")

                unavailable_policy_result = _run(
                    [
                        "python3",
                        str(CHECKER),
                        "--report-file",
                        str(unavailable_summary_file),
                        "--expected-final-decision",
                        "NO-GO",
                        "--ci-fast-gate",
                        "PASS",
                        "--expected-provider-client-contract",
                        "KolmeRuntimeCommitLiveProvider",
                        "--require-reason-code",
                        str(unavailable_reason_code),
                        "--output-json",
                        str(unavailable_policy_file),
                    ]
                )
                if unavailable_policy_result.returncode != 0:
                    raise RuntimeError(
                        "expected unavailable-node NO-GO policy check to pass with deterministic reason: "
                        f"{unavailable_policy_result.stdout}{unavailable_policy_result.stderr}"
                    )

                write_evidence_report(
                    process_harness_evidence_file,
                    {
                        "schema_version": "kamn.runtime.process-harness-evidence.v1",
                        "status": "pass",
                        "final_decision": "GO",
                        "reason_code": "local_live_provider_process_harness_verified",
                        "ports": {"mock_kolme_api": port},
                        "processes": [
                            {
                                "name": "mock_kolme_api",
                                "status": "running",
                                "pid": mock_process.process.pid,
                            }
                        ],
                        "artifacts": {
                            "mock_kolme_api_log": str(mock_server_log),
                        },
                    },
                )

                contract_report = {
                    "schema_version": "kamn.kolme.local-live-provider-runtime-integration-contract-report.v1",
                    "go_summary_file": str(go_summary_file),
                    "go_policy_file": str(go_policy_file),
                    "provider_mismatch_summary_file": str(provider_mismatch_summary_file),
                    "provider_mismatch_policy_file": str(provider_mismatch_policy_file),
                    "unavailable_summary_file": str(unavailable_summary_file),
                    "unavailable_policy_file": str(unavailable_policy_file),
                    "process_harness_evidence_file": str(process_harness_evidence_file),
                    "final_decision": "GO",
                }
                output_path = Path(args.output_json).resolve()
                output_path.parent.mkdir(parents=True, exist_ok=True)
                output_path.write_text(
                    json.dumps(contract_report, sort_keys=True, indent=2) + "\n",
                    encoding="utf-8",
                )
    except (RuntimeError, ProcessHarnessError) as exc:
        print(str(exc), file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > int(args.max_seconds):
        print(
            f"live-provider runtime integration contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local live-provider runtime integration contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
