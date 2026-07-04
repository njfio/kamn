#!/usr/bin/env python3
"""Contract lane runner for Kolme runtime commit checks."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import time
import json
import os
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_commit/runtime_commit_request_cases.txt"
PARITY_MATRIX_FILE = (
    ROOT_DIR / "fixtures/kolme_compatibility/runtime_commit_decomposition_parity_matrix.json"
)
PARITY_MATRIX_CHECKER = ROOT_DIR / "scripts/kolme/check_runtime_commit_decomposition_parity_matrix.py"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
FOUNDATION_DOC = ROOT_DIR / "docs/foundation/kolme-runtime-commit-client.md"
MAX_SECONDS_ENV = "KAMN_KOLME_RUNTIME_COMMIT_MAX_SECONDS"
TARGET_DIR_ENV = "KAMN_KOLME_RUNTIME_COMMIT_TARGET_DIR"
DEFAULT_MAX_SECONDS = 360
DEFAULT_TARGET_DIR = ROOT_DIR / "target/kolme-runtime-commit-contract"
TEST_TARGETS = (
    "kolme_runtime_commit_client",
    "kolme_runtime_commit_finality",
)


def parse_max_seconds() -> int:
    raw_value = os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip()
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError(f"{MAX_SECONDS_ENV} must be a positive integer")
    return int(raw_value)


def cargo_target_env() -> dict[str, str]:
    raw_value = os.environ.get(TARGET_DIR_ENV, "").strip()
    target_dir = Path(raw_value) if raw_value else DEFAULT_TARGET_DIR
    target_dir.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    return env


def artifact_executables(cargo_stdout: str) -> dict[str, Path]:
    executables: dict[str, Path] = {}
    for line in cargo_stdout.splitlines():
        if not line.strip().startswith("{"):
            continue
        try:
            artifact = json.loads(line)
        except json.JSONDecodeError:
            continue
        if artifact.get("reason") != "compiler-artifact":
            continue
        target_name = artifact.get("target", {}).get("name")
        executable = artifact.get("executable")
        if target_name in TEST_TARGETS and executable:
            executable_path = Path(executable)
            if executable_path.is_file():
                executables[target_name] = executable_path
    return executables


def prebuild_test_executables() -> tuple[int, dict[str, Path]]:
    result = subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "kolme_runtime_commit_client",
            "--test",
            "kolme_runtime_commit_finality",
            "--no-run",
            "--message-format=json",
        ],
        cwd=ROOT_DIR,
        check=False,
        env=cargo_target_env(),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if result.returncode != 0:
        print(result.stdout, file=sys.stderr)
        return result.returncode, {}

    executables = artifact_executables(result.stdout)
    missing_targets = [target for target in TEST_TARGETS if target not in executables]
    if missing_targets:
        print(
            "expected Cargo to report runtime commit test executables: "
            + ",".join(missing_targets),
            file=sys.stderr,
        )
        return 1, {}
    return 0, executables


def remaining_seconds(start_epoch: float, max_seconds: int) -> int:
    return max_seconds - int(time.monotonic() - start_epoch)


def run_prebuilt_test_executables(
    executables: dict[str, Path],
    start_epoch: float,
    max_seconds: int,
) -> int:
    for target in TEST_TARGETS:
        timeout_seconds = remaining_seconds(start_epoch, max_seconds)
        if timeout_seconds <= 0:
            print("Kolme runtime commit contract lane exceeded runtime budget", file=sys.stderr)
            return 1
        try:
            result = subprocess.run(
                [str(executables[target]), "--test-threads=1"],
                cwd=ROOT_DIR,
                check=False,
                capture_output=True,
                text=True,
                timeout=timeout_seconds,
            )
        except subprocess.TimeoutExpired as error:
            output = (error.stdout or "") + (error.stderr or "")
            if output:
                print(output, file=sys.stderr)
            print(
                f"Kolme runtime commit test timed out after {timeout_seconds}s: {target}",
                file=sys.stderr,
            )
            return 1
        if result.returncode != 0:
            output = (result.stdout or "") + (result.stderr or "")
            print(output, file=sys.stderr)
            return result.returncode
    return 0


def main() -> int:
    try:
        max_seconds = parse_max_seconds()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    if not FIXTURE_FILE.is_file():
        print("expected Kolme runtime commit fixture file to exist", file=sys.stderr)
        return 1
    if not PARITY_MATRIX_FILE.is_file():
        print("expected runtime commit decomposition parity matrix fixture to exist", file=sys.stderr)
        return 1
    if not PARITY_MATRIX_CHECKER.is_file():
        print("expected runtime commit decomposition parity checker to exist", file=sys.stderr)
        return 1
    if not ROADMAP_DOC.is_file():
        print("expected Kolme integration roadmap doc to exist", file=sys.stderr)
        return 1
    if not FOUNDATION_DOC.is_file():
        print("expected Kolme runtime foundation doc to exist", file=sys.stderr)
        return 1

    compile_result, executables = prebuild_test_executables()
    if compile_result != 0:
        return compile_result

    start_epoch = time.monotonic()

    with tempfile.NamedTemporaryFile(prefix="runtime-commit-parity-policy-", suffix=".json") as policy_tmp:
        parity_result = subprocess.run(
            [
                "python3",
                str(PARITY_MATRIX_CHECKER),
                "check",
                "--matrix-file",
                str(PARITY_MATRIX_FILE),
                "--output-json",
                policy_tmp.name,
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
    if parity_result.returncode != 0:
        output = (parity_result.stdout or "") + (parity_result.stderr or "")
        print(output, file=sys.stderr)
        return parity_result.returncode

    test_result = run_prebuilt_test_executables(executables, start_epoch, max_seconds)
    if test_result != 0:
        return test_result

    roadmap_text = ROADMAP_DOC.read_text(encoding="utf-8")
    if "run_runtime_commit_contract_lane.sh" not in roadmap_text:
        print("expected Kolme integration roadmap to reference runtime commit contract lane command", file=sys.stderr)
        return 1
    if "fixtures/kolme_commit/runtime_commit_request_cases.txt" not in roadmap_text:
        print("expected Kolme integration roadmap to reference runtime commit fixture path", file=sys.stderr)
        return 1

    foundation_text = FOUNDATION_DOC.read_text(encoding="utf-8")
    if "runtime_commit_decomposition_parity_matrix.json" not in foundation_text:
        print(
            "expected Kolme runtime foundation doc to reference runtime commit decomposition parity matrix",
            file=sys.stderr,
        )
        return 1
    if "check_runtime_commit_decomposition_parity_matrix.py" not in foundation_text:
        print(
            "expected Kolme runtime foundation doc to reference parity matrix checker command",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(f"Kolme runtime commit contract lane exceeded runtime budget: {elapsed_seconds}s", file=sys.stderr)
        return 1

    print("Kolme runtime commit contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
