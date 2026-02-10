#!/usr/bin/env python3
"""Live-network smoke lane contract runner."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402


def _require_non_negative_int(variable_name: str, raw_value: str) -> int:
    if not raw_value.isdigit():
        fail(f"{variable_name} must be a non-negative integer")
    return int(raw_value)


def _require_bool(variable_name: str, raw_value: str) -> bool:
    if raw_value not in {"true", "false"}:
        fail(f"{variable_name} must be true or false")
    return raw_value == "true"


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

    start_epoch = int(time.time())
    if fake_delay_seconds > 0:
        time.sleep(fake_delay_seconds)

    commands: list[str] = []
    try:
        if not skip_commands:
            subprocess.run(
                ["bash", str(ROOT_DIR / "scripts/sdk/run_localhost_signed_demo.sh")],
                cwd=ROOT_DIR,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=True,
                text=True,
            )
            commands.append("scripts/sdk/run_localhost_signed_demo.sh")

            subprocess.run(
                [
                    "cargo",
                    "test",
                    "-p",
                    "kamn-core",
                    "--test",
                    "role_smoke_network",
                    "functional_roles_complete_smoke_roundtrip_with_gossip",
                    "--",
                    "--exact",
                ],
                cwd=ROOT_DIR,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=True,
                text=True,
            )
            commands.append("cargo_test_role_smoke_network_functional_roundtrip")
    except subprocess.CalledProcessError as error:
        fail(f"live-network smoke lane command failed with exit code {error.returncode}")

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
