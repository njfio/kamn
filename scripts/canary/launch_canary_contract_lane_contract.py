#!/usr/bin/env python3
"""Launch canary contract-lane runner."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/canary/run_launch_canary_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def run_capture(command: list[str], *, cwd: Path) -> tuple[int, str]:
    """Run command and return exit code + merged output."""
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        cwd=str(cwd),
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def main(argv: list[str]) -> int:
    """Execute launch canary contract-lane checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    root_dir = Path(__file__).resolve().parents[2]
    matrix_script = root_dir / "scripts/canary/run_launch_canary_matrix.py"
    fixture_file = root_dir / "fixtures/launch_canary/critical_path_probe_cases.json"

    for required_file in (matrix_script, fixture_file):
        if not required_file.exists():
            return fail(f"missing required launch canary artifact: {required_file}")

    if not matrix_script.is_file() or not os.access(matrix_script, os.X_OK):
        return fail("launch canary matrix runner must be executable")

    with tempfile.NamedTemporaryFile(prefix="launch-canary-", suffix=".json") as report:
        report_path = Path(report.name)
        matrix_code, _matrix_output = run_capture(
            [
                "python3",
                str(matrix_script),
                "--fixture",
                str(fixture_file),
                "--output-json",
                str(report_path),
            ],
            cwd=root_dir,
        )
        if matrix_code != 0:
            return fail("expected launch canary matrix runner to pass")

        payload = json.loads(report_path.read_text(encoding="utf-8"))
        if payload.get("schema_version") != "kamn.launch-canary.probe-report.v1":
            return fail("unexpected canary report schema")
        if payload.get("failed_count") != 0:
            return fail("expected failed_count=0 for launch canary contract lane")
        cases = payload.get("cases", [])
        if not any(
            isinstance(case, dict)
            and case.get("name") == "missing_probe_evidence"
            and case.get("derived_decision") == "NO-GO"
            for case in cases
        ):
            return fail(
                "expected missing_probe_evidence regression case to derive NO-GO"
            )

    print("launch canary contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
