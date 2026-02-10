#!/usr/bin/env python3
"""Live transport smoke parity lane runner contract."""

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


ALLOWED_LANGUAGES = ("rust", "python", "typescript")
REPORT_SCHEMA_VERSION = "kamn.sdk.live-transport-smoke-parity-report.v1"


def normalize_languages(raw: str) -> str:
    """Normalize language selector and preserve deterministic ordering."""
    if raw == "" or raw == "all":
        return ",".join(ALLOWED_LANGUAGES)

    normalized: list[str] = []
    seen: set[str] = set()

    for token in raw.split(","):
        value = token.strip().lower()
        if value == "":
            continue
        if value not in ALLOWED_LANGUAGES:
            fail(f"unsupported language selector: {token}")
        if value not in seen:
            normalized.append(value)
            seen.add(value)

    if not normalized:
        fail("at least one language must be selected")

    return ",".join(normalized)


def require_non_negative_int(name: str, raw_value: str) -> int:
    """Require a non-negative integer with stable shell-compatible errors."""
    try:
        value = int(raw_value)
    except (TypeError, ValueError):
        fail(f"{name} must be a non-negative integer")
    if value < 0:
        fail(f"{name} must be a non-negative integer")
    return value


def require_retry_budget(raw_value: str) -> int:
    """Require retry budget to remain in the [0, 2] policy window."""
    try:
        value = int(raw_value)
    except (TypeError, ValueError):
        fail("KAMN_SDK_SMOKE_PARITY_MAX_RETRIES must be an integer between 0 and 2")
    if value < 0 or value > 2:
        fail("KAMN_SDK_SMOKE_PARITY_MAX_RETRIES must be an integer between 0 and 2")
    return value


def require_bool_flag(name: str, raw_value: str) -> bool:
    """Parse true/false flag with stable shell-compatible errors."""
    if raw_value not in {"true", "false"}:
        fail(f"{name} must be true or false")
    return raw_value == "true"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run SDK live transport smoke parity lane contract."
    )
    parser.add_argument("--output-json")
    parser.add_argument("--languages")
    return parser


def run_smoke_attempt(
    selected_languages: str,
    skip_commands: bool,
    force_failure: bool,
    commands: list[str],
) -> bool:
    """Run one smoke attempt and return success."""
    if force_failure:
        return False

    if skip_commands:
        return True

    command = (
        "bash scripts/sdk/run_live_transport_parity_contract_lane.sh "
        f"--languages {selected_languages}"
    )
    commands.append(command)
    completed = subprocess.run(
        [
            "bash",
            "scripts/sdk/run_live_transport_parity_contract_lane.sh",
            "--languages",
            selected_languages,
        ],
        cwd=ROOT_DIR,
        stdout=subprocess.DEVNULL,
        check=False,
    )
    return completed.returncode == 0


def main(argv: list[str]) -> int:
    args = build_parser().parse_args(argv)

    parity_runner = ROOT_DIR / "scripts/sdk/run_live_transport_parity_contract_lane.sh"
    if not parity_runner.is_file() or not os.access(parity_runner, os.X_OK):
        fail("expected live transport parity contract lane runner to be executable")

    raw_languages = args.languages
    if raw_languages is None:
        raw_languages = os.getenv(
            "KAMN_SDK_SMOKE_PARITY_LANGUAGES", "rust,python,typescript"
        )
    selected_languages = normalize_languages(raw_languages)

    max_seconds = require_non_negative_int(
        "KAMN_SDK_SMOKE_PARITY_MAX_SECONDS",
        os.getenv("KAMN_SDK_SMOKE_PARITY_MAX_SECONDS", "180"),
    )
    max_retries = require_retry_budget(
        os.getenv("KAMN_SDK_SMOKE_PARITY_MAX_RETRIES", "1")
    )
    fake_delay_seconds = require_non_negative_int(
        "KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS",
        os.getenv("KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS", "0"),
    )
    skip_commands = require_bool_flag(
        "KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS",
        os.getenv("KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS", "false"),
    )
    force_failure = require_bool_flag(
        "KAMN_SDK_SMOKE_PARITY_FORCE_FAILURE",
        os.getenv("KAMN_SDK_SMOKE_PARITY_FORCE_FAILURE", "false"),
    )

    start_epoch = int(time.time())
    if fake_delay_seconds > 0:
        time.sleep(fake_delay_seconds)

    commands: list[str] = []
    max_attempts = max_retries + 1
    attempt = 1
    retry_used = False
    retry_final_status = "failed"

    while True:
        if run_smoke_attempt(selected_languages, skip_commands, force_failure, commands):
            retry_final_status = "passed"
            break

        if attempt >= max_attempts:
            break

        retry_used = True
        attempt += 1

    retry_attempts = attempt
    if retry_attempts > 1:
        retry_used = True

    elapsed_seconds = int(time.time()) - start_epoch

    reason_codes: list[str] = []
    if retry_final_status != "passed":
        reason_codes.append("smoke_lane_failed")
    if retry_final_status != "passed" and retry_used:
        reason_codes.append("retry_budget_exceeded")
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")
    reason_codes = sorted(reason_codes)

    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    output_path: Path | None = None
    if args.output_json:
        output_path = Path(args.output_json)
        payload = {
            "schema_version": REPORT_SCHEMA_VERSION,
            "status": status,
            "final_decision": final_decision,
            "elapsed_seconds": elapsed_seconds,
            "max_seconds": max_seconds,
            "max_retries": max_retries,
            "retry_attempts": retry_attempts,
            "retry_used": retry_used,
            "retry_final_status": retry_final_status,
            "languages": [item for item in selected_languages.split(",") if item],
            "skip_commands": skip_commands,
            "force_failure": force_failure,
            "command_count": len(commands),
            "commands": commands,
            "reason_codes": reason_codes,
        }
        write_json(output_path, payload)

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"max_seconds={max_seconds}")
    print(f"retry_attempts={retry_attempts}")
    print(f"max_retries={max_retries}")
    print(f"retry_used={'true' if retry_used else 'false'}")
    print(f"retry_final_status={retry_final_status}")
    print(f"failed_checks={reason_codes_csv}")
    if output_path is not None:
        print(f"report_file={output_path.resolve()}")

    if status != "pass":
        fail(f"sdk live transport smoke parity lane failed closed: {reason_codes_csv}")

    print("sdk live transport smoke parity lane tests passed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
