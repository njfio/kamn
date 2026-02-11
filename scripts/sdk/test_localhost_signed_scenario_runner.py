#!/usr/bin/env python3
"""Unit tests for localhost signed scenario runner helpers."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import sys
import unittest

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from localhost_signed_scenario_runner import (  # type: ignore[import-not-found]
    ScenarioRunnerError,
    extract_reason_code,
    run_harness_scenario_with_retry,
)


@dataclass
class _StubRunResult:
    returncode: int
    stdout: str
    stderr: str = ""


class LocalhostSignedScenarioRunnerTests(unittest.TestCase):
    def test_extract_reason_code_parses_expected_marker(self) -> None:
        output = "status=fail; scenario=timeout; reason_code=unexpected_listener_completion; elapsed=1;"
        self.assertEqual(extract_reason_code(output), "unexpected_listener_completion")

    def test_extract_reason_code_returns_none_when_missing(self) -> None:
        self.assertIsNone(extract_reason_code("status=ok; scenario=success;"))

    def test_retry_retries_once_for_expected_timeout_race(self) -> None:
        calls: list[list[str]] = []
        responses = [
            _StubRunResult(
                returncode=1,
                stdout=(
                    "status=fail; scenario=timeout; "
                    "reason_code=unexpected_listener_completion;"
                ),
            ),
            _StubRunResult(
                returncode=0,
                stdout="status=pass; scenario=timeout; reason_code=listener_timeout_detected;",
            ),
        ]

        def _runner(command: list[str]) -> _StubRunResult:
            calls.append(command)
            return responses[len(calls) - 1]

        sleeps: list[float] = []

        result = run_harness_scenario_with_retry(
            harness_runner=Path("/tmp/fake-harness.sh"),
            scenario="timeout",
            output_json=Path("/tmp/fake-timeout.json"),
            timeout_seconds=1,
            max_attempts=2,
            retry_reason_code="unexpected_listener_completion",
            run_command=_runner,
            sleep_fn=sleeps.append,
        )

        self.assertEqual(result.returncode, 0)
        self.assertIn("listener_timeout_detected", result.stdout)
        self.assertEqual(result.attempts, 2)
        self.assertEqual(len(calls), 2)
        self.assertEqual(sleeps, [0.2])

    def test_retry_fails_closed_when_race_persists(self) -> None:
        def _runner(_command: list[str]) -> _StubRunResult:
            return _StubRunResult(
                returncode=1,
                stdout=(
                    "status=fail; scenario=timeout; "
                    "reason_code=unexpected_listener_completion;"
                ),
            )

        with self.assertRaises(ScenarioRunnerError):
            run_harness_scenario_with_retry(
                harness_runner=Path("/tmp/fake-harness.sh"),
                scenario="timeout",
                output_json=Path("/tmp/fake-timeout.json"),
                timeout_seconds=1,
                max_attempts=2,
                retry_reason_code="unexpected_listener_completion",
                run_command=_runner,
                sleep_fn=lambda _seconds: None,
            )


if __name__ == "__main__":
    unittest.main()
