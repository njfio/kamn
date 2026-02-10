#!/usr/bin/env python3
"""Live transport replay/tamper contract lane runner."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail  # noqa: E402


def _is_executable(path: Path) -> bool:
    return path.is_file() and os.access(path, os.X_OK)


def _require_contains_line(output: str, marker: str, message: str) -> None:
    if marker not in output.splitlines():
        fail(message)


def _require_contains_text(output: str, marker: str, message: str) -> None:
    if marker not in output:
        fail(message)


def run_lane(args: argparse.Namespace) -> int:
    mode = args.mode
    generator = ROOT_DIR / "scripts/sdk/generate_live_transport_replay_tamper_evidence_bundle.sh"
    checker = ROOT_DIR / "scripts/sdk/check_live_transport_replay_tamper_policy.sh"
    runtime_doc = ROOT_DIR / "docs/foundation/runtime-network.md"
    invariant_doc = ROOT_DIR / "docs/testing/invariant-and-fuzz-strategy.md"

    if not _is_executable(generator):
        fail("expected replay/tamper evidence generator to be executable")
    if not _is_executable(checker):
        fail("expected replay/tamper evidence policy checker to be executable")
    if not runtime_doc.is_file():
        fail("expected runtime-network doc to exist")
    if not invariant_doc.is_file():
        fail("expected invariant-and-fuzz-strategy doc to exist")

    max_seconds_raw = os.environ.get("KAMN_SDK_REPLAY_TAMPER_CONTRACT_MAX_SECONDS", "90")
    if not max_seconds_raw.isdigit() or int(max_seconds_raw) <= 0:
        fail("KAMN_SDK_REPLAY_TAMPER_CONTRACT_MAX_SECONDS must be a positive integer")
    max_seconds = int(max_seconds_raw)

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_path = Path(tmp_dir)
        go_bundle = (
            Path(args.output_report)
            if args.output_report
            else tmp_path / "live-transport-replay-tamper-go.json"
        )

        go_generate = subprocess.run(
            [
                "bash",
                str(generator),
                "--output-file",
                str(go_bundle),
                "--transport-lane-id",
                "localhost-signed-integration",
                "--message-id",
                "msg-go-001",
                "--from-did",
                "kamn:did:agent:sender-1",
                "--to-did",
                "kamn:did:agent:listener-1",
                "--nonce",
                "41",
                "--signature-status",
                "valid",
                "--replay-detected",
                "false",
                "--tamper-detected",
                "false",
                "--ci-fast-gate",
                "PASS",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if go_generate.returncode != 0:
            fail((go_generate.stderr or go_generate.stdout or "GO bundle generation failed").strip())
        _require_contains_line(
            go_generate.stdout,
            "status=generated",
            "expected GO replay/tamper evidence generation status marker",
        )
        _require_contains_line(
            go_generate.stdout,
            "final_decision=GO",
            "expected GO replay/tamper evidence final decision marker",
        )

        go_check = subprocess.run(
            ["bash", str(checker), "--bundle-file", str(go_bundle)],
            capture_output=True,
            text=True,
            check=False,
        )
        if go_check.returncode != 0:
            fail((go_check.stderr or go_check.stdout or "GO policy check failed").strip())
        _require_contains_line(
            go_check.stdout,
            "status=ok",
            "expected GO replay/tamper policy checker status marker",
        )
        _require_contains_line(
            go_check.stdout,
            "final_decision=GO",
            "expected GO replay/tamper policy checker final decision marker",
        )

        no_go_bundle = tmp_path / "live-transport-replay-tamper-no-go.json"
        no_go_generate = subprocess.run(
            [
                "bash",
                str(generator),
                "--output-file",
                str(no_go_bundle),
                "--transport-lane-id",
                "localhost-signed-integration",
                "--message-id",
                "msg-no-go-001",
                "--from-did",
                "kamn:did:agent:sender-1",
                "--to-did",
                "kamn:did:agent:listener-1",
                "--nonce",
                "41",
                "--signature-status",
                "malformed",
                "--replay-detected",
                "true",
                "--tamper-detected",
                "true",
                "--ci-fast-gate",
                "PASS",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if no_go_generate.returncode != 0:
            fail(
                (no_go_generate.stderr or no_go_generate.stdout or "NO-GO generation failed").strip()
            )
        _require_contains_line(
            no_go_generate.stdout,
            "final_decision=NO-GO",
            "expected NO-GO replay/tamper final decision marker",
        )

        no_go_check = subprocess.run(
            ["bash", str(checker), "--bundle-file", str(no_go_bundle)],
            capture_output=True,
            text=True,
            check=False,
        )
        if no_go_check.returncode != 0:
            fail((no_go_check.stderr or no_go_check.stdout or "NO-GO policy check failed").strip())
        _require_contains_line(
            no_go_check.stdout,
            "final_decision=NO-GO",
            "expected NO-GO replay/tamper policy checker final decision marker",
        )

        tampered_bundle = tmp_path / "live-transport-replay-tamper-tampered.json"
        payload = json.loads(no_go_bundle.read_text(encoding="utf-8"))
        payload["final_decision"] = "GO"
        tampered_bundle.write_text(
            json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8"
        )

        tampered_check = subprocess.run(
            ["bash", str(checker), "--bundle-file", str(tampered_bundle)],
            capture_output=True,
            text=True,
            check=False,
        )
        if tampered_check.returncode == 0:
            fail("expected tampered replay/tamper evidence bundle to fail policy checker")
        _require_contains_text(
            tampered_check.stderr or tampered_check.stdout,
            "policy decision mismatch",
            "expected policy decision mismatch marker for tampered replay/tamper bundle",
        )

        runtime_doc_text = runtime_doc.read_text(encoding="utf-8")
        if "run_live_transport_replay_tamper_contract_lane.sh" not in runtime_doc_text:
            fail("expected runtime-network doc to reference replay/tamper contract lane command")
        if "run_live_transport_replay_tamper_fast_lane.sh" not in runtime_doc_text:
            fail("expected runtime-network doc to reference replay/tamper fast lane command")
        if "run_live_transport_replay_tamper_deep_lane.sh" not in runtime_doc_text:
            fail("expected runtime-network doc to reference replay/tamper deep lane command")
        if "generate_live_transport_replay_tamper_evidence_bundle.sh" not in runtime_doc_text:
            fail("expected runtime-network doc to reference replay/tamper generator command")
        if "check_live_transport_replay_tamper_policy.sh" not in runtime_doc_text:
            fail("expected runtime-network doc to reference replay/tamper policy checker command")
        if "kamn.sdk.live-transport-replay-tamper-evidence.v1" not in runtime_doc_text:
            fail("expected runtime-network doc to reference replay/tamper schema marker")

        invariant_doc_text = invariant_doc.read_text(encoding="utf-8")
        if "run_live_transport_replay_tamper_contract_lane.sh" not in invariant_doc_text:
            fail("expected invariant-and-fuzz strategy doc to reference replay/tamper contract lane")
        if "run_live_transport_replay_tamper_fast_lane.sh" not in invariant_doc_text:
            fail("expected invariant-and-fuzz strategy doc to reference replay/tamper fast lane")
        if "run_live_transport_replay_tamper_deep_lane.sh" not in invariant_doc_text:
            fail("expected invariant-and-fuzz strategy doc to reference replay/tamper deep lane")
        if "check_live_transport_replay_tamper_policy.sh" not in invariant_doc_text:
            fail("expected invariant-and-fuzz strategy doc to reference replay/tamper policy checker")

        if mode == "deep":
            deep_no_go_bundle = tmp_path / "live-transport-replay-tamper-deep-no-go.json"
            deep_no_go_generate = subprocess.run(
                [
                    "bash",
                    str(generator),
                    "--output-file",
                    str(deep_no_go_bundle),
                    "--transport-lane-id",
                    "localhost-signed-integration",
                    "--message-id",
                    "msg-deep-no-go-001",
                    "--from-did",
                    "kamn:did:agent:sender-1",
                    "--to-did",
                    "kamn:did:agent:listener-1",
                    "--nonce",
                    "42",
                    "--signature-status",
                    "mismatch",
                    "--replay-detected",
                    "false",
                    "--tamper-detected",
                    "false",
                    "--ci-fast-gate",
                    "FAIL",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            if deep_no_go_generate.returncode != 0:
                fail(
                    (
                        deep_no_go_generate.stderr
                        or deep_no_go_generate.stdout
                        or "deep NO-GO generation failed"
                    ).strip()
                )
            _require_contains_line(
                deep_no_go_generate.stdout,
                "final_decision=NO-GO",
                "expected deep replay/tamper generation to produce NO-GO",
            )

            deep_no_go_check = subprocess.run(
                ["bash", str(checker), "--bundle-file", str(deep_no_go_bundle)],
                capture_output=True,
                text=True,
                check=False,
            )
            if deep_no_go_check.returncode != 0:
                fail(
                    (
                        deep_no_go_check.stderr
                        or deep_no_go_check.stdout
                        or "deep NO-GO policy check failed"
                    ).strip()
                )
            _require_contains_line(
                deep_no_go_check.stdout,
                "final_decision=NO-GO",
                "expected deep replay/tamper policy checker to keep NO-GO decision",
            )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(f"replay/tamper contract lane exceeded runtime budget: {elapsed_seconds}s")

        print("status=ok")
        print(f"report_file={go_bundle}")
        print(f"lane_mode={mode}")
        if mode == "deep":
            print("deep_no_go_status=verified")
        print("final_decision=GO")
        print("live transport replay/tamper contract lane tests passed.")
        return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run live transport replay/tamper evidence contract lane."
    )
    parser.add_argument("--output-report", default="")
    parser.add_argument("--mode", default="fast", choices=("fast", "deep"))
    parser.set_defaults(handler=run_lane)
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
