#!/usr/bin/env python3
"""Live-network smoke lane contract runner."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

ROLE_SMOKE_TEST_TARGET = "role_smoke_network"
ROLE_SMOKE_TEST_NAME = "functional_roles_complete_smoke_roundtrip_with_gossip"
TARGET_DIR = Path(
    os.environ.get(
        "KAMN_LIVE_NETWORK_SMOKE_TARGET_DIR",
        str(ROOT_DIR / "target" / "contract-lanes" / "live-network-smoke"),
    )
)


def _require_non_negative_int(variable_name: str, raw_value: str) -> int:
    if not raw_value.isdigit():
        fail(f"{variable_name} must be a non-negative integer")
    return int(raw_value)


def _require_bool(variable_name: str, raw_value: str) -> bool:
    if raw_value not in {"true", "false"}:
        fail(f"{variable_name} must be true or false")
    return raw_value == "true"


def _cargo_target_env() -> dict[str, str]:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(TARGET_DIR)
    return env


def _run_checked_command(
    command: list[str],
    label: str,
    env_overrides: dict[str, str] | None = None,
    timeout_seconds: int | None = None,
) -> None:
    env = os.environ.copy()
    if env_overrides:
        env.update(env_overrides)
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT_DIR,
            capture_output=True,
            check=False,
            env=env,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        output = f"{error.stdout or ''}{error.stderr or ''}".strip()
        detail = f"live-network smoke command timed out: {label}"
        if timeout_seconds is not None:
            detail = f"{detail} after {timeout_seconds}s"
        if output:
            detail = f"{detail}: {output}"
        fail(detail)
    if completed.returncode == 0:
        return

    output = f"{completed.stdout}{completed.stderr}".strip()
    detail = f"{label} failed with exit code {completed.returncode}"
    if output:
        detail = f"{detail}: {output}"
    fail(detail)


def _artifact_executable(cargo_stdout: str, target_name: str) -> Path:
    executable = None
    for line in cargo_stdout.splitlines():
        if not line.strip().startswith("{"):
            continue
        try:
            artifact = json.loads(line)
        except json.JSONDecodeError:
            continue
        if artifact.get("reason") != "compiler-artifact":
            continue
        if artifact.get("target", {}).get("name") != target_name:
            continue
        if artifact.get("executable"):
            executable = Path(artifact["executable"])

    if executable is not None and executable.is_file():
        return executable
    fail(f"expected Cargo to report executable for {target_name}")


def _prebuild_role_smoke_network(timeout_seconds: int) -> Path:
    TARGET_DIR.mkdir(parents=True, exist_ok=True)
    try:
        completed = subprocess.run(
            [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                ROLE_SMOKE_TEST_TARGET,
                "--no-run",
                "--message-format=json",
            ],
            cwd=ROOT_DIR,
            check=False,
            env=_cargo_target_env(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        output = f"{error.stdout or ''}{error.stderr or ''}".strip()
        detail = (
            "live-network smoke command timed out: "
            f"role smoke network prebuild after {timeout_seconds}s"
        )
        if output:
            detail = f"{detail}: {output}"
        fail(detail)
    if completed.returncode != 0:
        output = f"{completed.stdout}{completed.stderr}".strip()
        detail = f"role smoke network prebuild failed with exit code {completed.returncode}"
        if output:
            detail = f"{detail}: {output}"
        fail(detail)
    return _artifact_executable(completed.stdout, ROLE_SMOKE_TEST_TARGET)


def _prebuild_smoke_artifacts(timeout_seconds: int) -> Path:
    _run_checked_command(
        [
            "cargo",
            "build",
            "--quiet",
            "-p",
            "kamn-sdk",
            "--example",
            "localhost_signed_listener",
            "--example",
            "localhost_signed_sender",
        ],
        "localhost signed demo example prebuild",
        {"CARGO_TARGET_DIR": str(TARGET_DIR)},
        timeout_seconds=timeout_seconds,
    )
    return _prebuild_role_smoke_network(timeout_seconds)


def run_live_network_smoke_lane(args: argparse.Namespace) -> int:
    output_json = args.output_json or ""

    max_seconds = _require_non_negative_int(
        "KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS",
        os.environ.get("KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS", "120"),
    )
    fake_delay_seconds = _require_non_negative_int(
        "KAMN_LIVE_NETWORK_SMOKE_FAKE_DELAY_SECONDS",
        os.environ.get("KAMN_LIVE_NETWORK_SMOKE_FAKE_DELAY_SECONDS", "0"),
    )
    skip_commands = _require_bool(
        "KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS",
        os.environ.get("KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS", "false"),
    )

    role_smoke_executable: Path | None = None
    command_timeout_seconds = max(max_seconds, 1)
    if not skip_commands:
        role_smoke_executable = _prebuild_smoke_artifacts(command_timeout_seconds)

    start_epoch = int(time.time())
    if fake_delay_seconds > 0:
        time.sleep(fake_delay_seconds)

    commands: list[str] = []
    if not skip_commands:
        _run_checked_command(
            ["bash", str(ROOT_DIR / "scripts/sdk/run_localhost_signed_demo.sh")],
            "localhost signed demo",
            {
                "KAMN_LOCALHOST_SIGNED_DEMO_TIMEOUT_SECONDS": str(
                    command_timeout_seconds
                ),
                "KAMN_LOCALHOST_SIGNED_DEMO_SKIP_BUILD": "true",
            },
            timeout_seconds=command_timeout_seconds,
        )
        commands.append("scripts/sdk/run_localhost_signed_demo.sh")

        if role_smoke_executable is None:
            fail("expected prebuilt role smoke executable")
        _run_checked_command(
            [
                str(role_smoke_executable),
                ROLE_SMOKE_TEST_NAME,
                "--exact",
            ],
            "role smoke network contract",
            timeout_seconds=command_timeout_seconds,
        )
        commands.append("cargo_test_role_smoke_network_functional_roundtrip")

    elapsed_seconds = int(time.time()) - start_epoch
    reason_codes: list[str] = []
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    failed_checks = "none" if not reason_codes else ",".join(reason_codes)

    report_path: Path | None = None
    if output_json:
        report_path = Path(output_json)
        write_json(
            report_path,
            {
                "schema_version": "kamn.runtime.live-network-smoke-report.v1",
                "status": status,
                "final_decision": final_decision,
                "elapsed_seconds": elapsed_seconds,
                "max_seconds": max_seconds,
                "skip_commands": skip_commands,
                "command_count": len(commands),
                "commands": commands,
                "reason_codes": reason_codes,
            },
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"failed_checks={failed_checks}")
    if report_path is not None:
        print(f"report_file={report_path.resolve()}")

    if status != "pass":
        fail(
            "live-network smoke lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    print("live-network smoke lane tests passed.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run the live-network smoke lane.")
    parser.add_argument("--output-json", default="")
    parser.set_defaults(handler=run_live_network_smoke_lane)
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
