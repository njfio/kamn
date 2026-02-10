#!/usr/bin/env python3
"""Localhost signed integration harness runner."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

FROM_DID = "kamn:did:agent:sender-1"
TO_DID = "kamn:did:agent:listener-1"
STATE_HASH = "state:localhost-demo"
BODY = "hello-from-localhost-demo"


class ScenarioFailure(Exception):
    """Raised when a harness scenario must fail with a reason code."""

    def __init__(self, reason_code: str) -> None:
        super().__init__(reason_code)
        self.reason_code = reason_code


class HarnessContext:
    """Runtime context for scenario execution."""

    def __init__(self, scenario: str, addr: str, timeout_seconds: int, output_json: str) -> None:
        self.scenario = scenario
        self.addr = addr
        self.timeout_seconds = timeout_seconds
        self.output_json = output_json
        self.start_epoch = int(time.time())
        self.tmp_dir = Path(tempfile.mkdtemp())
        self.listener_out = self.tmp_dir / "listener.out"
        self.sender_out = self.tmp_dir / "sender.out"
        self.listener_proc: subprocess.Popen[str] | None = None

    def elapsed_seconds(self) -> int:
        return int(time.time()) - self.start_epoch

    def cleanup(self) -> None:
        if self.listener_proc is not None and self.listener_proc.poll() is None:
            self.listener_proc.kill()
            try:
                self.listener_proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
        shutil.rmtree(self.tmp_dir, ignore_errors=True)

    def emit_report(self, status: str, reason_code: str) -> None:
        elapsed = self.elapsed_seconds()
        evidence_key = f"localhost_signed_integration:{self.scenario}:v1"
        reason_key = f"localhost_signed_integration_reason:{reason_code}:v1"

        print(
            f"status={status}; scenario={self.scenario}; evidence_key={evidence_key}; "
            f"reason_code={reason_code}; reason_key={reason_key}; elapsed_seconds={elapsed};"
        )

        if self.output_json:
            write_json(
                Path(self.output_json),
                {
                    "schema_version": "kamn.sdk.localhost-signed.integration-harness.v1",
                    "status": status,
                    "scenario": self.scenario,
                    "evidence_key": evidence_key,
                    "reason_code": reason_code,
                    "reason_key": reason_key,
                    "addr": self.addr,
                    "timeout_seconds": self.timeout_seconds,
                    "elapsed_seconds": elapsed,
                },
            )

    def fail_with_reason(self, reason_code: str) -> None:
        raise ScenarioFailure(reason_code)

    def start_listener(self) -> None:
        with self.listener_out.open("w", encoding="utf-8") as listener_stream:
            self.listener_proc = subprocess.Popen(
                [
                    "cargo",
                    "run",
                    "--quiet",
                    "-p",
                    "kamn-sdk",
                    "--example",
                    "localhost_signed_listener",
                    "--",
                    "--addr",
                    self.addr,
                    "--expected-from",
                    FROM_DID,
                    "--expected-to",
                    TO_DID,
                    "--state-hash",
                    STATE_HASH,
                ],
                cwd=ROOT_DIR,
                stdout=listener_stream,
                stderr=subprocess.STDOUT,
                text=True,
            )

    def wait_for_listener(self) -> int:
        if self.listener_proc is None:
            return 1

        elapsed = 0
        while self.listener_proc.poll() is None:
            if elapsed >= self.timeout_seconds:
                return 124
            time.sleep(1)
            elapsed += 1

        return self.listener_proc.returncode or 0

    def send_invalid_signature_payload(self) -> bool:
        host, port_raw = self.addr.rsplit(":", 1)
        port = int(port_raw)
        payload = (
            f"from={FROM_DID}\n"
            f"to={TO_DID}\n"
            "nonce=1\n"
            f"state_hash={STATE_HASH}\n"
            f"body={BODY}\n"
            "signature=sig:ed25519:baseline-v1:invalid\n"
        )

        for _ in range(20):
            try:
                with socket.create_connection((host, port), timeout=1.0) as connection:
                    connection.sendall(payload.encode("utf-8"))
                    return True
            except OSError:
                time.sleep(0.1)
        return False

    def run_success_scenario(self) -> None:
        self.start_listener()
        with self.sender_out.open("w", encoding="utf-8") as sender_stream:
            sender_result = subprocess.run(
                [
                    "cargo",
                    "run",
                    "--quiet",
                    "-p",
                    "kamn-sdk",
                    "--example",
                    "localhost_signed_sender",
                    "--",
                    "--addr",
                    self.addr,
                    "--from",
                    FROM_DID,
                    "--to",
                    TO_DID,
                    "--nonce",
                    "1",
                    "--state-hash",
                    STATE_HASH,
                    "--body",
                    BODY,
                ],
                cwd=ROOT_DIR,
                stdout=sender_stream,
                stderr=subprocess.STDOUT,
                check=False,
                text=True,
            )
        if sender_result.returncode != 0:
            self.fail_with_reason("sender_failed")

        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")
        if listener_status != 0:
            self.fail_with_reason("listener_failed")

        sender_text = self.sender_out.read_text(encoding="utf-8")
        listener_text = self.listener_out.read_text(encoding="utf-8")
        if "status=ok" not in sender_text:
            self.fail_with_reason("sender_status_missing")
        if "status=ok" not in listener_text:
            self.fail_with_reason("listener_status_missing")
        if "verified=true" not in listener_text:
            self.fail_with_reason("listener_verification_missing")

        self.emit_report("pass", "none")

    def run_signature_mismatch_scenario(self) -> None:
        self.start_listener()
        if not self.send_invalid_signature_payload():
            self.fail_with_reason("payload_send_failed")

        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")

        listener_text = self.listener_out.read_text(encoding="utf-8")
        if "status=error" not in listener_text:
            self.fail_with_reason("listener_error_status_missing")
        if "signature verification failed" not in listener_text:
            if "status=ok" in listener_text:
                self.fail_with_reason("mismatch_not_detected")
            self.fail_with_reason("signature_mismatch_not_reported")

        self.emit_report("pass", "signature_mismatch_detected")

    def run_timeout_scenario(self) -> None:
        self.start_listener()
        listener_status = self.wait_for_listener()
        if listener_status != 124:
            self.fail_with_reason("unexpected_listener_completion")

        if self.listener_proc is not None and self.listener_proc.poll() is None:
            self.listener_proc.kill()
            self.listener_proc.wait(timeout=2)

        self.emit_report("pass", "listener_timeout_detected")


def _require_positive_int(name: str, raw_value: str) -> int:
    if not raw_value.isdigit() or int(raw_value) <= 0:
        fail(f"{name} must be a positive integer")
    return int(raw_value)


def run_harness(args: argparse.Namespace) -> int:
    scenario = args.scenario
    if scenario not in {"success", "signature-mismatch", "timeout"}:
        fail("scenario must be one of: success, signature-mismatch, timeout")

    addr = args.addr
    if not addr or ":" not in addr:
        fail("addr must be in host:port form")

    timeout_seconds = _require_positive_int("timeout-seconds", str(args.timeout_seconds))

    context = HarnessContext(
        scenario=scenario,
        addr=addr,
        timeout_seconds=timeout_seconds,
        output_json=args.output_json,
    )
    try:
        try:
            if scenario == "success":
                context.run_success_scenario()
            elif scenario == "signature-mismatch":
                context.run_signature_mismatch_scenario()
            else:
                context.run_timeout_scenario()
            return 0
        except ScenarioFailure as error:
            context.emit_report("fail", error.reason_code)
            return 1
    finally:
        context.cleanup()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run localhost signed integration harness scenarios."
    )
    parser.add_argument(
        "--scenario",
        default=os.environ.get("KAMN_LOCALHOST_SIGNED_INTEGRATION_SCENARIO", "success"),
    )
    parser.add_argument(
        "--addr",
        default=os.environ.get("KAMN_LOCALHOST_SIGNED_INTEGRATION_ADDR", "127.0.0.1:17883"),
    )
    parser.add_argument(
        "--timeout-seconds",
        default=os.environ.get("KAMN_LOCALHOST_SIGNED_INTEGRATION_TIMEOUT_SECONDS", "5"),
    )
    parser.add_argument("--output-json", default="")
    parser.set_defaults(handler=run_harness)
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
