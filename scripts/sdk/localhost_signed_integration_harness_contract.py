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
SESSION_ID = "session:localhost-demo:v1"
STATE_HASH = "state:localhost-demo"
BODY = "hello-from-localhost-demo"
NONCE_REPLAY_TEST_VALUE = 7
UNAUTHORIZED_FROM_DID = "kamn:did:agent:rogue-1"
STALE_STATE_HASH = "state:localhost-stale"


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

    def emit_report(
        self,
        status: str,
        reason_code: str,
        extra_fields: dict[str, object] | None = None,
    ) -> None:
        elapsed = self.elapsed_seconds()
        evidence_key = f"localhost_signed_integration:{self.scenario}:v1"
        reason_key = f"localhost_signed_integration_reason:{reason_code}:v1"
        report: dict[str, object] = {
            "schema_version": "kamn.sdk.localhost-signed.integration-harness.v1",
            "status": status,
            "scenario": self.scenario,
            "evidence_key": evidence_key,
            "reason_code": reason_code,
            "reason_key": reason_key,
            "addr": self.addr,
            "timeout_seconds": self.timeout_seconds,
            "elapsed_seconds": elapsed,
        }
        if extra_fields:
            report.update(extra_fields)

        print(
            f"status={status}; scenario={self.scenario}; evidence_key={evidence_key}; "
            f"reason_code={reason_code}; reason_key={reason_key}; elapsed_seconds={elapsed};"
        )

        if self.output_json:
            write_json(Path(self.output_json), report)

    def fail_with_reason(self, reason_code: str) -> None:
        raise ScenarioFailure(reason_code)

    def start_listener(
        self,
        *,
        expected_from: str = FROM_DID,
        expected_to: str = TO_DID,
        expected_session_id: str = SESSION_ID,
        expected_state_hash: str = STATE_HASH,
        nonce_state_file: Path | None = None,
    ) -> None:
        args = [
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
            expected_from,
            "--expected-to",
            expected_to,
            "--expected-session-id",
            expected_session_id,
            "--state-hash",
            expected_state_hash,
        ]
        if nonce_state_file is not None:
            args.extend(["--nonce-state-file", str(nonce_state_file)])

        with self.listener_out.open("w", encoding="utf-8") as listener_stream:
            self.listener_proc = subprocess.Popen(
                args,
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

    def _send_payload(self, payload: str) -> bool:
        host, port_raw = self.addr.rsplit(":", 1)
        port = int(port_raw)
        for _ in range(20):
            try:
                with socket.create_connection((host, port), timeout=1.0) as connection:
                    connection.sendall(payload.encode("utf-8"))
                    return True
            except OSError:
                time.sleep(0.1)
        return False

    def _signature_for_fields(
        self,
        *,
        from_did: str,
        session_id: str,
        nonce: int,
        state_hash: str,
        body: str,
    ) -> str:
        return (
            "sig:ed25519:baseline-v1:"
            f"{from_did}:{session_id}:{nonce}:{state_hash}:{len(body)}"
        )

    def _build_wire_payload(
        self,
        *,
        from_did: str,
        to_did: str,
        session_id: str,
        nonce: int,
        state_hash: str,
        body: str,
        signature: str | None = None,
    ) -> str:
        signature_value = signature or self._signature_for_fields(
            from_did=from_did,
            session_id=session_id,
            nonce=nonce,
            state_hash=state_hash,
            body=body,
        )
        payload = (
            f"from={from_did}\n"
            f"to={to_did}\n"
            f"session_id={session_id}\n"
            f"nonce={nonce}\n"
            f"state_hash={state_hash}\n"
            f"body={body}\n"
            f"signature={signature_value}\n"
        )
        return payload

    def send_invalid_signature_payload(self) -> bool:
        payload = self._build_wire_payload(
            from_did=FROM_DID,
            to_did=TO_DID,
            session_id=SESSION_ID,
            nonce=1,
            state_hash=STATE_HASH,
            body=BODY,
            signature="sig:ed25519:baseline-v1:invalid",
        )
        return self._send_payload(payload)

    def run_sender(
        self,
        *,
        from_did: str = FROM_DID,
        to_did: str = TO_DID,
        session_id: str = SESSION_ID,
        nonce: int = 1,
        state_hash: str = STATE_HASH,
        body: str = BODY,
    ) -> int:
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
                    from_did,
                    "--to",
                    to_did,
                    "--session-id",
                    session_id,
                    "--nonce",
                    str(nonce),
                    "--state-hash",
                    state_hash,
                    "--body",
                    body,
                ],
                cwd=ROOT_DIR,
                stdout=sender_stream,
                stderr=subprocess.STDOUT,
                check=False,
                text=True,
            )
        return sender_result.returncode

    def _require_listener_failure(self, marker: str, failure_reason: str) -> None:
        listener_text = self.listener_out.read_text(encoding="utf-8")
        if "status=error" not in listener_text:
            self.fail_with_reason("listener_error_status_missing")
        if marker not in listener_text:
            if "status=ok" in listener_text:
                self.fail_with_reason(failure_reason)
            self.fail_with_reason(f"{failure_reason}_not_reported")

    def _wait_listener_success(self) -> None:
        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")
        if listener_status != 0:
            self.fail_with_reason("listener_failed")

    def run_success_scenario(self) -> None:
        self.start_listener()
        if self.run_sender(nonce=1) != 0:
            self.fail_with_reason("sender_failed")

        self._wait_listener_success()

        sender_text = self.sender_out.read_text(encoding="utf-8")
        listener_text = self.listener_out.read_text(encoding="utf-8")
        if "status=ok" not in sender_text:
            self.fail_with_reason("sender_status_missing")
        if "status=ok" not in listener_text:
            self.fail_with_reason("listener_status_missing")
        if "verified=true" not in listener_text:
            self.fail_with_reason("listener_verification_missing")
        if f"session_id={SESSION_ID}" not in sender_text:
            self.fail_with_reason("sender_session_id_missing")
        if f"session_id={SESSION_ID}" not in listener_text:
            self.fail_with_reason("listener_session_id_missing")

        self.emit_report(
            "pass",
            "none",
            extra_fields={
                "signature_guard_status": "pass",
                "admission_guard_status": "pass",
                "session_id": SESSION_ID,
            },
        )

    def run_signature_mismatch_scenario(self) -> None:
        self.start_listener()
        if not self.send_invalid_signature_payload():
            self.fail_with_reason("payload_send_failed")

        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")
        self._require_listener_failure("signature verification failed", "mismatch_not_detected")
        self.emit_report(
            "pass",
            "signature_mismatch_detected",
            extra_fields={"signature_guard_status": "pass"},
        )

    def run_timeout_scenario(self) -> None:
        self.start_listener()
        listener_status = self.wait_for_listener()
        if listener_status != 124:
            self.fail_with_reason("unexpected_listener_completion")

        if self.listener_proc is not None and self.listener_proc.poll() is None:
            self.listener_proc.kill()
            self.listener_proc.wait(timeout=2)

        self.emit_report("pass", "listener_timeout_detected")

    def run_replay_nonce_scenario(self) -> None:
        nonce_state_file = self.tmp_dir / "replay-nonce.state"
        self.start_listener(nonce_state_file=nonce_state_file)
        if self.run_sender(nonce=NONCE_REPLAY_TEST_VALUE) != 0:
            self.fail_with_reason("sender_failed")
        self._wait_listener_success()

        first_listener_text = self.listener_out.read_text(encoding="utf-8")
        if "status=ok" not in first_listener_text:
            self.fail_with_reason("listener_status_missing")

        self.start_listener(nonce_state_file=nonce_state_file)
        if self.run_sender(nonce=NONCE_REPLAY_TEST_VALUE) != 0:
            self.fail_with_reason("sender_failed")

        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")
        self._require_listener_failure("replay nonce detected", "replay_not_detected")

        self.emit_report(
            "pass",
            "replay_nonce_detected",
            extra_fields={
                "replay_guard_status": "pass",
                "replay_rejected_nonce": NONCE_REPLAY_TEST_VALUE,
            },
        )

    def run_admission_guards_scenario(self) -> None:
        expected_reason_codes = [
            "stale_session_detected",
            "unauthorized_sender_detected",
            "malformed_payload_detected",
        ]
        observed_reason_codes: list[str] = []

        self.start_listener(expected_state_hash=STATE_HASH)
        if self.run_sender(nonce=3, state_hash=STALE_STATE_HASH) != 0:
            self.fail_with_reason("sender_failed")
        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")
        self._require_listener_failure("unexpected state hash", "stale_session")
        observed_reason_codes.append("stale_session_detected")

        self.start_listener(expected_from=FROM_DID)
        if self.run_sender(from_did=UNAUTHORIZED_FROM_DID, nonce=4) != 0:
            self.fail_with_reason("sender_failed")
        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")
        self._require_listener_failure("unexpected sender did", "unauthorized_sender")
        observed_reason_codes.append("unauthorized_sender_detected")

        self.start_listener()
        malformed_payload = (
            f"from={FROM_DID}\n"
            f"to={TO_DID}\n"
            f"session_id={SESSION_ID}\n"
            f"state_hash={STATE_HASH}\n"
            f"body={BODY}\n"
            "signature=sig:ed25519:baseline-v1:malformed\n"
        )
        if not self._send_payload(malformed_payload):
            self.fail_with_reason("payload_send_failed")
        listener_status = self.wait_for_listener()
        if listener_status == 124:
            self.fail_with_reason("listener_timeout")
        self._require_listener_failure("wire message missing nonce", "malformed_payload")
        observed_reason_codes.append("malformed_payload_detected")

        if observed_reason_codes != expected_reason_codes:
            self.fail_with_reason("admission_reason_codes_mismatch")

        self.emit_report(
            "pass",
            "session_admission_guards_detected",
            extra_fields={
                "admission_guard_status": "pass",
                "admission_reason_codes": observed_reason_codes,
            },
        )


def _require_positive_int(name: str, raw_value: str) -> int:
    if not raw_value.isdigit() or int(raw_value) <= 0:
        fail(f"{name} must be a positive integer")
    return int(raw_value)


def run_harness(args: argparse.Namespace) -> int:
    scenario = args.scenario
    if scenario not in {
        "success",
        "signature-mismatch",
        "timeout",
        "replay-nonce",
        "admission-guards",
    }:
        fail(
            "scenario must be one of: success, signature-mismatch, timeout, "
            "replay-nonce, admission-guards"
        )

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
            elif scenario == "timeout":
                context.run_timeout_scenario()
            elif scenario == "replay-nonce":
                context.run_replay_nonce_scenario()
            else:
                context.run_admission_guards_scenario()
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
