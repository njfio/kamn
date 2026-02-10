#!/usr/bin/env python3
"""Shared helpers for contract-lane runner scripts."""

from __future__ import annotations

import subprocess
import time
from pathlib import Path
from typing import Iterable, Sequence


class ContractLaneError(RuntimeError):
    """Raised when a contract-lane helper detects a failed invariant."""


def run_capture(command: list[str], *, cwd: Path | None = None) -> tuple[int, str]:
    """Run command and return exit code with merged stdout/stderr output."""
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        cwd=str(cwd) if cwd is not None else None,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def build_default_bundle_args(*, output_file: str, pairs: Iterable[tuple[str, str]]) -> list[str]:
    """Build stable argument list with required output file and ordered option pairs."""
    args: list[str] = ["--output-file", output_file]
    for key, value in pairs:
        args.extend([key, value])
    return args


def require_output_contains(output: str, *, expected: str, context: str) -> None:
    """Require output to include expected marker, otherwise raise ContractLaneError."""
    if expected not in output:
        raise ContractLaneError(f"{context} missing expected marker: {expected}")


def run_go_bundle_policy_pair(
    *,
    root_dir: Path,
    generator: Path,
    generator_args: Sequence[str],
    policy_checker: Path,
    bundle_file: Path,
    decision_marker: str = "final_decision=GO",
) -> tuple[str, str]:
    """Run generator + policy checker pair and require GO decision markers."""
    generator_command = ["bash", str(generator), *generator_args]
    generator_code, generator_output = run_capture(generator_command, cwd=root_dir)
    if generator_code != 0:
        raise ContractLaneError(
            f"generator command failed with exit code {generator_code}: {' '.join(generator_command)}"
        )
    require_output_contains(
        generator_output,
        expected=decision_marker,
        context="generator output",
    )

    policy_command = ["bash", str(policy_checker), "--bundle-file", str(bundle_file)]
    policy_code, policy_output = run_capture(policy_command, cwd=root_dir)
    if policy_code != 0:
        raise ContractLaneError(
            f"policy checker failed with exit code {policy_code}: {' '.join(policy_command)}"
        )
    require_output_contains(
        policy_output,
        expected=decision_marker,
        context="policy checker output",
    )
    return generator_output, policy_output


def enforce_runtime_budget(*, lane_name: str, started_at: float, max_runtime_seconds: int) -> int:
    """Enforce runtime budget and return elapsed seconds."""
    elapsed_seconds = int(time.monotonic() - started_at)
    if elapsed_seconds > max_runtime_seconds:
        raise ContractLaneError(
            f"{lane_name} exceeded runtime budget: {elapsed_seconds}s > {max_runtime_seconds}s"
        )
    return elapsed_seconds
