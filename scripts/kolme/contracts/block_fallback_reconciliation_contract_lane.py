#!/usr/bin/env python3
"""Contract lane runner for Kolme block fallback reconciliation checks."""

from __future__ import annotations

import os
import json
import subprocess
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNTIME_NETWORK_DOC = ROOT_DIR / "docs/foundation/runtime-network.md"
DEVNET_DOC = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
TEST_TARGET = "kolme_runtime_commit_block_fallback"
MAX_SECONDS_ENV = "KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 300
CARGO_TARGET_DIR = ROOT_DIR / "target" / "contract-lanes" / TEST_TARGET
COMPILE_COMMAND = ["cargo", "test", "-p", "kamn-core", "--test"]
COMPILE_ARGS = [TEST_TARGET, "--no-run", "--message-format=json"]
TEST_ARGS = ["--test-threads=1", "--nocapture"]


def parse_max_seconds() -> int:
    raw_value = os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip()
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError(f"{MAX_SECONDS_ENV} must be a positive integer")
    return int(raw_value)


def validate_doc_files() -> int:
    if not RUNTIME_NETWORK_DOC.is_file():
        print("expected runtime network documentation to exist", file=sys.stderr)
        return 1

    if not DEVNET_DOC.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1

    if not ROADMAP_DOC.is_file():
        print("expected Kolme integration roadmap documentation to exist", file=sys.stderr)
        return 1

    return 0


def validate_docs_reference_lane() -> int:
    runtime_network_doc_text = RUNTIME_NETWORK_DOC.read_text(encoding="utf-8")
    devnet_doc_text = DEVNET_DOC.read_text(encoding="utf-8")
    roadmap_doc_text = ROADMAP_DOC.read_text(encoding="utf-8")

    if validate_runtime_network_doc(runtime_network_doc_text) != 0:
        return 1
    if validate_devnet_doc(devnet_doc_text) != 0:
        return 1
    return validate_roadmap_doc(roadmap_doc_text)


def validate_runtime_network_doc(doc_text: str) -> int:
    manifest_command = (
        "run_manifest_lane.sh --manifest "
        "scripts/framework/manifests/kolme_block_fallback_reconciliation_contract_lane.json "
        "--phase contract"
    )
    if "run_block_fallback_reconciliation_contract_lane.sh" not in doc_text and (
        manifest_command not in doc_text
    ):
        print(
            "expected runtime network documentation to reference block fallback reconciliation lane command",
            file=sys.stderr,
        )
        return 1
    if "Regression: #1464" not in doc_text:
        print(
            "expected runtime network documentation to include block fallback regression marker",
            file=sys.stderr,
        )
        return 1
    return 0


def validate_devnet_doc(doc_text: str) -> int:
    if "run_block_fallback_reconciliation_contract_lane.sh" not in doc_text:
        print(
            "expected Kolme devnet ops documentation to reference block fallback reconciliation lane command",
            file=sys.stderr,
        )
        return 1
    return 0


def validate_roadmap_doc(doc_text: str) -> int:
    if "run_block_fallback_reconciliation_contract_lane.sh" not in doc_text:
        print(
            "expected Kolme integration roadmap to reference block fallback reconciliation lane command",
            file=sys.stderr,
        )
        return 1
    if "Regression: #1464" not in doc_text:
        print(
            "expected Kolme integration roadmap to include block fallback regression marker",
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
        [*COMPILE_COMMAND, *COMPILE_ARGS],
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
        print(
            "expected Cargo to report block fallback test executable",
            file=sys.stderr,
        )
        return 1, None
    return 0, executable


def run_timed_block_fallback_tests(max_seconds: int, executable: Path) -> int:
    try:
        result = subprocess.run(
            [str(executable), *TEST_ARGS],
            cwd=ROOT_DIR,
            check=False,
            timeout=max_seconds,
        )
    except subprocess.TimeoutExpired:
        print(
            f"Kolme block fallback reconciliation contract lane exceeded runtime budget: {max_seconds}s",
            file=sys.stderr,
        )
        return 1
    if result.returncode != 0:
        return result.returncode
    return 0


def main() -> int:
    try:
        max_seconds = parse_max_seconds()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1
    if validate_doc_files() != 0:
        return 1
    compile_result, executable = prebuild_test_executable()
    if compile_result != 0 or executable is None:
        return compile_result or 1
    if run_timed_block_fallback_tests(max_seconds, executable) != 0:
        return 1
    if validate_docs_reference_lane() != 0:
        return 1
    print("Kolme block fallback reconciliation contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
