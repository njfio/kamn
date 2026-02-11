#!/usr/bin/env python3
"""Contract lane runner for Kolme snapshot drift checks."""

from __future__ import annotations

import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
TEST_CHECKER = ROOT_DIR / "scripts/kolme/test_check_snapshot_drift.sh"
DOC_FILE = ROOT_DIR / "docs/research/kolme-upstream-compatibility.md"
MAX_SECONDS = 45


def main() -> int:
    if not TEST_CHECKER.is_file() or not TEST_CHECKER.stat().st_mode & 0o111:
        print(
            "expected Kolme snapshot drift checker test script to be executable",
            file=sys.stderr,
        )
        return 1

    if not DOC_FILE.is_file():
        print(
            "expected Kolme upstream compatibility research doc to exist",
            file=sys.stderr,
        )
        return 1

    start_epoch = time.monotonic()

    result = subprocess.run(
        ["bash", str(TEST_CHECKER)],
        cwd=ROOT_DIR,
        check=False,
    )
    if result.returncode != 0:
        return result.returncode

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    if "check_snapshot_drift.py" not in doc_text:
        print(
            "expected Kolme compatibility doc to reference drift checker command",
            file=sys.stderr,
        )
        return 1

    if "run_snapshot_drift_contract_lane.sh" not in doc_text:
        print(
            "expected Kolme compatibility doc to reference contract lane command",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > MAX_SECONDS:
        print(
            f"Kolme snapshot drift contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("Kolme snapshot drift contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
