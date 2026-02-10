#!/usr/bin/env python3
"""Localhost signed integration contract lane runner."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402


def _is_executable(path: Path) -> bool:
    return path.is_file() and os.access(path, os.X_OK)


def _require_contains(text: str, marker: str, message: str) -> None:
    if marker not in text:
        fail(message)


def _require_file_contains(path: Path, marker: str, message: str) -> None:
    _require_contains(path.read_text(encoding="utf-8"), marker, message)


def run_localhost_signed_integration_contract_lane(args: argparse.Namespace) -> int:
    output_json = args.output_json

    harness_runner = ROOT_DIR / "scripts/sdk/run_localhost_signed_integration_harness.sh"
    policy_checker = ROOT_DIR / "scripts/sdk/check_localhost_signed_integration_evidence_policy.sh"
    live_network_doc = ROOT_DIR / "docs/planning/live-network-wave.md"
    runtime_network_doc = ROOT_DIR / "docs/foundation/runtime-network.md"
    readme_file = ROOT_DIR / "README.md"

    if not _is_executable(harness_runner):
        fail("expected localhost signed integration harness runner to be executable")
    if not _is_executable(policy_checker):
        fail("expected localhost signed integration evidence policy checker to be executable")
    if not live_network_doc.is_file():
        fail("expected live-network planning doc to exist")
    if not runtime_network_doc.is_file():
        fail("expected runtime-network foundation doc to exist")
    if not readme_file.is_file():
        fail("expected README.md to exist")

    max_seconds_raw = os.environ.get("KAMN_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_MAX_SECONDS", "120")
    if not max_seconds_raw.isdigit() or int(max_seconds_raw) <= 0:
        fail("contract max seconds must be a positive integer")
    max_seconds = int(max_seconds_raw)

    start_epoch = int(time.time())
    tmp_dir = Path(tempfile.mkdtemp())
    try:
        success_report = tmp_dir / "success.json"
        signature_report = tmp_dir / "signature-mismatch.json"
        timeout_report = tmp_dir / "timeout.json"
        replay_report = tmp_dir / "replay-nonce.json"
        admission_report = tmp_dir / "admission-guards.json"
        summary_report = tmp_dir / "localhost-signed-integration-contract.json"

        success_run = subprocess.run(
            [
                "bash",
                str(harness_runner),
                "--scenario",
                "success",
                "--output-json",
                str(success_report),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if success_run.returncode != 0:
            fail((success_run.stderr or success_run.stdout or "success scenario failed").strip())
        success_output = success_run.stdout
        _require_contains(
            success_output,
            "status=pass; scenario=success;",
            "expected localhost signed integration success scenario status marker",
        )
        _require_contains(
            success_output,
            "reason_code=none;",
            "expected localhost signed integration success scenario reason code marker",
        )
        _require_contains(
            success_output,
            "evidence_key=localhost_signed_integration:success:v1;",
            "expected localhost signed integration success scenario evidence key",
        )

        signature_run = subprocess.run(
            [
                "bash",
                str(harness_runner),
                "--scenario",
                "signature-mismatch",
                "--output-json",
                str(signature_report),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if signature_run.returncode != 0:
            fail(
                (
                    signature_run.stderr
                    or signature_run.stdout
                    or "signature-mismatch scenario failed"
                ).strip()
            )
        signature_output = signature_run.stdout
        _require_contains(
            signature_output,
            "status=pass; scenario=signature-mismatch;",
            "expected localhost signed integration signature mismatch scenario status marker",
        )
        _require_contains(
            signature_output,
            "reason_code=signature_mismatch_detected;",
            "expected localhost signed integration signature mismatch scenario reason code marker",
        )
        _require_contains(
            signature_output,
            "evidence_key=localhost_signed_integration:signature-mismatch:v1;",
            "expected localhost signed integration signature mismatch scenario evidence key",
        )

        timeout_run = subprocess.run(
            [
                "bash",
                str(harness_runner),
                "--scenario",
                "timeout",
                "--timeout-seconds",
                "1",
                "--output-json",
                str(timeout_report),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if timeout_run.returncode != 0:
            fail((timeout_run.stderr or timeout_run.stdout or "timeout scenario failed").strip())
        timeout_output = timeout_run.stdout
        _require_contains(
            timeout_output,
            "status=pass; scenario=timeout;",
            "expected localhost signed integration timeout scenario status marker",
        )
        _require_contains(
            timeout_output,
            "reason_code=listener_timeout_detected;",
            "expected localhost signed integration timeout scenario reason code marker",
        )
        _require_contains(
            timeout_output,
            "evidence_key=localhost_signed_integration:timeout:v1;",
            "expected localhost signed integration timeout scenario evidence key",
        )

        replay_run = subprocess.run(
            [
                "bash",
                str(harness_runner),
                "--scenario",
                "replay-nonce",
                "--output-json",
                str(replay_report),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if replay_run.returncode != 0:
            fail((replay_run.stderr or replay_run.stdout or "replay-nonce scenario failed").strip())
        replay_output = replay_run.stdout
        _require_contains(
            replay_output,
            "status=pass; scenario=replay-nonce;",
            "expected localhost signed integration replay nonce scenario status marker",
        )
        _require_contains(
            replay_output,
            "reason_code=replay_nonce_detected;",
            "expected localhost signed integration replay nonce scenario reason code marker",
        )
        _require_contains(
            replay_output,
            "evidence_key=localhost_signed_integration:replay-nonce:v1;",
            "expected localhost signed integration replay nonce scenario evidence key",
        )

        admission_run = subprocess.run(
            [
                "bash",
                str(harness_runner),
                "--scenario",
                "admission-guards",
                "--output-json",
                str(admission_report),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if admission_run.returncode != 0:
            fail(
                (
                    admission_run.stderr
                    or admission_run.stdout
                    or "admission-guards scenario failed"
                ).strip()
            )
        admission_output = admission_run.stdout
        _require_contains(
            admission_output,
            "status=pass; scenario=admission-guards;",
            "expected localhost signed integration admission guards scenario status marker",
        )
        _require_contains(
            admission_output,
            "reason_code=session_admission_guards_detected;",
            "expected localhost signed integration admission guards scenario reason code marker",
        )
        _require_contains(
            admission_output,
            "evidence_key=localhost_signed_integration:admission-guards:v1;",
            "expected localhost signed integration admission guards scenario evidence key",
        )

        success_payload = load_json(success_report)
        signature_payload = load_json(signature_report)
        timeout_payload = load_json(timeout_report)
        replay_payload = load_json(replay_report)
        admission_payload = load_json(admission_report)
        summary = {
            "schema_version": "kamn.sdk.localhost-signed.integration-contract.v1",
            "status": "pass",
            "contract_key": "localhost_signed_integration_contract:v1",
            "success_scenario_status": success_payload["status"],
            "signature_mismatch_scenario_status": signature_payload["status"],
            "timeout_scenario_status": timeout_payload["status"],
            "replay_nonce_scenario_status": replay_payload["status"],
            "admission_guards_scenario_status": admission_payload["status"],
            "success_evidence_key": success_payload["evidence_key"],
            "signature_mismatch_evidence_key": signature_payload["evidence_key"],
            "timeout_evidence_key": timeout_payload["evidence_key"],
            "replay_nonce_evidence_key": replay_payload["evidence_key"],
            "admission_guards_evidence_key": admission_payload["evidence_key"],
            "signature_mismatch_reason_code": signature_payload["reason_code"],
            "timeout_reason_code": timeout_payload["reason_code"],
            "replay_nonce_reason_code": replay_payload["reason_code"],
            "admission_guards_reason_code": admission_payload["reason_code"],
            "success_reason_key": success_payload["reason_key"],
            "signature_mismatch_reason_key": signature_payload["reason_key"],
            "timeout_reason_key": timeout_payload["reason_key"],
            "replay_nonce_reason_key": replay_payload["reason_key"],
            "admission_guards_reason_key": admission_payload["reason_key"],
            "success_elapsed_seconds": success_payload["elapsed_seconds"],
            "signature_mismatch_elapsed_seconds": signature_payload["elapsed_seconds"],
            "timeout_elapsed_seconds": timeout_payload["elapsed_seconds"],
            "replay_nonce_elapsed_seconds": replay_payload["elapsed_seconds"],
            "admission_guards_elapsed_seconds": admission_payload["elapsed_seconds"],
            "replay_guard_status": replay_payload["replay_guard_status"],
            "replay_rejected_nonce": replay_payload["replay_rejected_nonce"],
            "admission_guard_status": admission_payload["admission_guard_status"],
            "admission_reason_codes": admission_payload["admission_reason_codes"],
        }
        summary_report.write_text(
            json.dumps(summary, separators=(",", ":")),
            encoding="utf-8",
        )

        policy_run = subprocess.run(
            ["bash", str(policy_checker), "--report-file", str(summary_report)],
            capture_output=True,
            text=True,
            check=False,
        )
        if policy_run.returncode != 0:
            fail((policy_run.stderr or policy_run.stdout or "policy checker failed").strip())
        _require_contains(
            policy_run.stdout,
            "status=ok",
            "expected localhost signed integration evidence policy check to pass",
        )

        _require_file_contains(
            live_network_doc,
            "run_localhost_signed_integration_contract_lane.sh",
            "expected live-network planning doc to reference localhost signed integration contract lane",
        )
        _require_file_contains(
            live_network_doc,
            "check_localhost_signed_integration_evidence_policy.sh",
            "expected live-network planning doc to reference localhost signed integration evidence policy checker",
        )
        _require_file_contains(
            live_network_doc,
            "/tmp/localhost-signed-integration-contract-report.json",
            "expected live-network planning doc to reference localhost signed integration report artifact path",
        )
        _require_file_contains(
            readme_file,
            "run_localhost_signed_integration_contract_lane.sh",
            "expected README to reference localhost signed integration contract lane command",
        )
        _require_file_contains(
            readme_file,
            "check_localhost_signed_integration_evidence_policy.sh",
            "expected README to reference localhost signed integration evidence policy checker command",
        )
        _require_file_contains(
            readme_file,
            "/tmp/localhost-signed-integration-contract-report.json",
            "expected README to reference localhost signed integration report artifact path",
        )
        _require_file_contains(
            runtime_network_doc,
            "Localhost Signed Integration Evidence Key Contract Rules",
            "expected runtime-network doc to define localhost signed integration evidence key contract rules",
        )
        _require_file_contains(
            runtime_network_doc,
            "localhost_signed_integration_contract:v1",
            "expected runtime-network doc to reference localhost signed integration contract key",
        )

        if output_json:
            output_path = Path(output_json)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(summary_report, output_path)

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(
                "localhost signed integration contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s"
            )

        print("localhost_signed_integration_success=pass")
        print("localhost_signed_integration_signature_mismatch=pass")
        print("localhost_signed_integration_timeout=pass")
        print("localhost_signed_integration_replay_nonce=pass")
        print("localhost_signed_integration_admission_guards=pass")
        print("localhost_signed_integration_policy=ok")
        print("localhost signed integration contract lane tests passed.")
        return 0
    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run localhost signed integration contract lane checks."
    )
    parser.add_argument("--output-json", default="")
    parser.set_defaults(handler=run_localhost_signed_integration_contract_lane)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
