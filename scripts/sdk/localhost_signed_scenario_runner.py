#!/usr/bin/env python3
"""Shared localhost signed integration harness scenario runner helpers."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
import subprocess
import time
from typing import Callable


REASON_CODE_PATTERN = re.compile(r"reason_code=([a-z0-9_\\-]+);")


class ScenarioRunnerError(RuntimeError):
    """Raised when a localhost signed scenario run fails closed."""


@dataclass(frozen=True)
class ScenarioExecutionResult:
    """Result payload for scenario harness execution."""

    returncode: int
    stdout: str
    stderr: str
    attempts: int

    @property
    def combined_output(self) -> str:
        return f"{self.stdout}{self.stderr}"


def extract_reason_code(output: str) -> str | None:
    """Extract `reason_code=...;` marker from scenario output."""
    match = REASON_CODE_PATTERN.search(output)
    if match is None:
        return None
    return match.group(1)


def _default_run_command(command: list[str]) -> ScenarioExecutionResult:
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
    )
    return ScenarioExecutionResult(
        returncode=result.returncode,
        stdout=result.stdout or "",
        stderr=result.stderr or "",
        attempts=1,
    )


def run_harness_scenario_with_retry(
    *,
    harness_runner: Path,
    scenario: str,
    output_json: Path,
    timeout_seconds: int | None = None,
    max_attempts: int = 1,
    retry_reason_code: str | None = None,
    retry_delay_seconds: float = 0.2,
    run_command: Callable[[list[str]], ScenarioExecutionResult] | None = None,
    sleep_fn: Callable[[float], None] = time.sleep,
) -> ScenarioExecutionResult:
    """Run localhost integration harness scenario with bounded retry support."""
    if max_attempts <= 0:
        raise ScenarioRunnerError("max_attempts must be >= 1")

    command = [
        "bash",
        str(harness_runner),
        "--scenario",
        scenario,
        "--output-json",
        str(output_json),
    ]
    if timeout_seconds is not None:
        command.extend(["--timeout-seconds", str(timeout_seconds)])

    execute = run_command or _default_run_command

    for attempt in range(1, max_attempts + 1):
        raw_result = execute(command)
        result = ScenarioExecutionResult(
            returncode=raw_result.returncode,
            stdout=raw_result.stdout,
            stderr=raw_result.stderr,
            attempts=attempt,
        )
        if result.returncode == 0:
            return result

        combined_output = result.combined_output
        reason_code = extract_reason_code(combined_output)
        should_retry = (
            retry_reason_code is not None
            and reason_code == retry_reason_code
            and attempt < max_attempts
        )
        if should_retry:
            sleep_fn(retry_delay_seconds)
            continue

        failure_output = (result.stderr or result.stdout).strip()
        if not failure_output:
            failure_output = (
                f"scenario '{scenario}' failed with exit code {result.returncode}"
            )
        raise ScenarioRunnerError(failure_output)

    raise ScenarioRunnerError(f"scenario '{scenario}' exhausted retry budget")
