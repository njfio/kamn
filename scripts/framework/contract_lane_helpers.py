#!/usr/bin/env python3
"""Shared helpers for contract-lane runner scripts."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Iterable


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
