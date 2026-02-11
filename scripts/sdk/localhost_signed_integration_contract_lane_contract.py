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
from localhost_signed_report_composer import (  # noqa: E402
    compose_localhost_signed_integration_contract_report,
)


def _is_executable(path: Path) -> bool:
    return path.is_file() and os.access(path, os.X_OK)


def _require_contains(text: str, marker: str, message: str) -> None:
    if marker not in text:
        fail(message)


def _require_file_contains(path: Path, marker: str, message: str) -> None:
    _require_contains(path.read_text(encoding="utf-8"), marker, message)


def _load_fixture_contract(
    fixture_file: Path,
) -> tuple[str, list[str], dict[str, dict[str, str]]]:
    payload = load_json(fixture_file)
    schema_version = payload.get("schema_version")
    if schema_version != "kamn.sdk.localhost-signed.integration-fixtures.v1":
        fail("unexpected localhost signed integration fixture schema_version")

    scenario_ids = payload.get("scenario_ids")
    if scenario_ids != ["success-v1", "signature-mismatch-v1", "timeout-v1"]:
        fail("scenario_ids must match deterministic localhost fixture contract")

    scenarios = payload.get("scenarios")
    if not isinstance(scenarios, list):
        fail("fixture scenarios must be an array")

    fixture_by_scenario: dict[str, dict[str, str]] = {}
    for scenario_entry in scenarios:
        if not isinstance(scenario_entry, dict):
            fail("each fixture scenario entry must be an object")
        scenario_name = scenario_entry.get("scenario")
        if not isinstance(scenario_name, str) or not scenario_name:
            fail("fixture scenario entry must include non-empty scenario")
        normalized_entry: dict[str, str] = {}
        for field_name in (
            "id",
            "expected_status",
            "expected_reason_code",
            "expected_evidence_key",
            "expected_reason_key",
        ):
            value = scenario_entry.get(field_name)
            if not isinstance(value, str) or not value:
                fail(f"fixture scenario '{scenario_name}' missing field '{field_name}'")
            normalized_entry[field_name] = value
        fixture_by_scenario[scenario_name] = normalized_entry

    for required_scenario in ("success", "signature-mismatch", "timeout"):
        if required_scenario not in fixture_by_scenario:
            fail(f"fixture scenarios missing required scenario '{required_scenario}'")

    return schema_version, scenario_ids, fixture_by_scenario


def run_localhost_signed_integration_contract_lane(args: argparse.Namespace) -> int:
    output_json = args.output_json

    harness_runner = ROOT_DIR / "scripts/sdk/run_localhost_signed_integration_harness.sh"
    policy_checker = ROOT_DIR / "scripts/sdk/check_localhost_signed_integration_evidence_policy.sh"
    fixture_file = ROOT_DIR / "fixtures/runtime/localhost_signed_integration_cases.json"
    live_network_doc = ROOT_DIR / "docs/planning/live-network-wave.md"
    runtime_network_doc = ROOT_DIR / "docs/foundation/runtime-network.md"
    readme_file = ROOT_DIR / "README.md"

    if not _is_executable(harness_runner):
        fail("expected localhost signed integration harness runner to be executable")
    if not _is_executable(policy_checker):
        fail("expected localhost signed integration evidence policy checker to be executable")
    if not fixture_file.is_file():
        fail("expected localhost signed integration fixture corpus file to exist")
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
    fixture_schema_version, scenario_fixture_ids, fixture_by_scenario = _load_fixture_contract(
        fixture_file
    )

    start_epoch = int(time.time())
    tmp_dir = Path(tempfile.mkdtemp())
    try:
        success_report = tmp_dir / "success.json"
        signature_report = tmp_dir / "signature-mismatch.json"
        malformed_signature_report = tmp_dir / "malformed-signature.json"
        timeout_report = tmp_dir / "timeout.json"
        session_expired_report = tmp_dir / "session-expired.json"
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
            f"reason_code={fixture_by_scenario['success']['expected_reason_code']};",
            "expected localhost signed integration success scenario reason code marker",
        )
        _require_contains(
            success_output,
            f"evidence_key={fixture_by_scenario['success']['expected_evidence_key']};",
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
            f"reason_code={fixture_by_scenario['signature-mismatch']['expected_reason_code']};",
            "expected localhost signed integration signature mismatch scenario reason code marker",
        )
        _require_contains(
            signature_output,
            f"evidence_key={fixture_by_scenario['signature-mismatch']['expected_evidence_key']};",
            "expected localhost signed integration signature mismatch scenario evidence key",
        )

        malformed_signature_run = subprocess.run(
            [
                "bash",
                str(harness_runner),
                "--scenario",
                "malformed-signature",
                "--output-json",
                str(malformed_signature_report),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if malformed_signature_run.returncode != 0:
            fail(
                (
                    malformed_signature_run.stderr
                    or malformed_signature_run.stdout
                    or "malformed-signature scenario failed"
                ).strip()
            )
        malformed_signature_output = malformed_signature_run.stdout
        _require_contains(
            malformed_signature_output,
            "status=pass; scenario=malformed-signature;",
            "expected localhost signed integration malformed signature scenario status marker",
        )
        _require_contains(
            malformed_signature_output,
            "reason_code=malformed_signature_detected;",
            "expected localhost signed integration malformed signature scenario reason code marker",
        )
        _require_contains(
            malformed_signature_output,
            "evidence_key=localhost_signed_integration:malformed-signature:v1;",
            "expected localhost signed integration malformed signature scenario evidence key",
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
            f"reason_code={fixture_by_scenario['timeout']['expected_reason_code']};",
            "expected localhost signed integration timeout scenario reason code marker",
        )
        _require_contains(
            timeout_output,
            f"evidence_key={fixture_by_scenario['timeout']['expected_evidence_key']};",
            "expected localhost signed integration timeout scenario evidence key",
        )

        session_expired_run = subprocess.run(
            [
                "bash",
                str(harness_runner),
                "--scenario",
                "session-expired",
                "--output-json",
                str(session_expired_report),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if session_expired_run.returncode != 0:
            fail(
                (
                    session_expired_run.stderr
                    or session_expired_run.stdout
                    or "session-expired scenario failed"
                ).strip()
            )
        session_expired_output = session_expired_run.stdout
        _require_contains(
            session_expired_output,
            "status=pass; scenario=session-expired;",
            "expected localhost signed integration session-expired scenario status marker",
        )
        _require_contains(
            session_expired_output,
            "reason_code=session_expired_detected;",
            "expected localhost signed integration session-expired scenario reason code marker",
        )
        _require_contains(
            session_expired_output,
            "evidence_key=localhost_signed_integration:session-expired:v1;",
            "expected localhost signed integration session-expired scenario evidence key",
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
        malformed_signature_payload = load_json(malformed_signature_report)
        timeout_payload = load_json(timeout_report)
        session_expired_payload = load_json(session_expired_report)
        replay_payload = load_json(replay_report)
        admission_payload = load_json(admission_report)
        summary = compose_localhost_signed_integration_contract_report(
            fixture_schema_version=fixture_schema_version,
            scenario_fixture_ids=scenario_fixture_ids,
            success_payload=success_payload,
            signature_payload=signature_payload,
            malformed_signature_payload=malformed_signature_payload,
            timeout_payload=timeout_payload,
            session_expired_payload=session_expired_payload,
            replay_payload=replay_payload,
            admission_payload=admission_payload,
        )
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
        print("localhost_signed_integration_malformed_signature=pass")
        print("localhost_signed_integration_timeout=pass")
        print("localhost_signed_integration_session_expired=pass")
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
