#!/usr/bin/env python3
"""Contract lane runner for Kolme notifications consumer checks."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNTIME_NETWORK_DOC = ROOT_DIR / "docs/foundation/runtime-network.md"
TEST_TARGET = "kolme_runtime_commit_notifications"
MAX_SECONDS_ENV = "KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 60
CARGO_TARGET_DIR = ROOT_DIR / "target" / "contract-lanes" / TEST_TARGET
COMPILE_COMMAND = [
    "cargo",
    "test",
    "-p",
    "kamn-core",
    "--test",
    TEST_TARGET,
    "--no-run",
    "--message-format=json",
]
TEST_ARGS = [
    "--test-threads=1",
    "--nocapture",
]


def parse_max_seconds() -> int:
    raw_value = os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip()
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError(f"{MAX_SECONDS_ENV} must be a positive integer")
    return int(raw_value)


def validate_runtime_doc() -> int:
    if not RUNTIME_NETWORK_DOC.is_file():
        print("expected runtime network documentation to exist", file=sys.stderr)
        return 1

    doc_text = RUNTIME_NETWORK_DOC.read_text(encoding="utf-8")
    if (
        "run_notifications_consumer_contract_lane.sh" not in doc_text
        and "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_notifications_consumer_contract_lane.json --phase contract"
        not in doc_text
    ):
        print(
            "expected runtime network documentation to reference notifications consumer lane command",
            file=sys.stderr,
        )
        return 1

    if "Regression: #1463" not in doc_text:
        print(
            "expected runtime network documentation to include notifications consumer regression marker",
            file=sys.stderr,
        )
        return 1

    return 0


def cargo_target_env() -> dict[str, str]:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(CARGO_TARGET_DIR)
    return env


def artifact_executable(cargo_stdout: str) -> Path | None:
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
        if artifact.get("target", {}).get("name") != TEST_TARGET:
            continue
        if artifact.get("executable"):
            executable = Path(artifact["executable"])

    if executable is not None and executable.is_file():
        return executable
    return None


def prebuild_test_executable() -> tuple[int, Path | None]:
    result = subprocess.run(
        COMPILE_COMMAND,
        cwd=ROOT_DIR,
        check=False,
        env=cargo_target_env(),
        stdout=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        return result.returncode, None

    executable = artifact_executable(result.stdout)
    if executable is None:
        print("expected Cargo to report notifications consumer test executable", file=sys.stderr)
        return 1, None
    return 0, executable


def run_timed_notifications_tests(max_seconds: int, executable: Path) -> int:
    start_epoch = time.monotonic()
    try:
        result = subprocess.run(
            [str(executable), *TEST_ARGS],
            cwd=ROOT_DIR,
            check=False,
            timeout=max_seconds,
        )
    except subprocess.TimeoutExpired:
        print(
            f"Kolme notifications consumer contract lane exceeded runtime budget: {max_seconds}s",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if result.returncode != 0:
        return result.returncode

    if elapsed_seconds > max_seconds:
        print(
            f"Kolme notifications consumer contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("Kolme notifications consumer contract lane tests passed.")
    return 0


def main() -> int:
    try:
        max_seconds = parse_max_seconds()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    doc_result = validate_runtime_doc()
    if doc_result != 0:
        return doc_result

    compile_result, executable = prebuild_test_executable()
    if compile_result != 0:
        return compile_result
    if executable is None:
        return 1

    return run_timed_notifications_tests(max_seconds, executable)


if __name__ == "__main__":
    raise SystemExit(main())
