#!/usr/bin/env python3
"""Contract lane runner for Kolme runtime commit checks."""

from __future__ import annotations

import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_commit/runtime_commit_request_cases.txt"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
MAX_SECONDS = 60


def main() -> int:
    if not FIXTURE_FILE.is_file():
        print("expected Kolme runtime commit fixture file to exist", file=sys.stderr)
        return 1
    if not ROADMAP_DOC.is_file():
        print("expected Kolme integration roadmap doc to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

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
        ],
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        output = (result.stdout or "") + (result.stderr or "")
        print(output, file=sys.stderr)
        return result.returncode

    roadmap_text = ROADMAP_DOC.read_text(encoding="utf-8")
    if "run_runtime_commit_contract_lane.sh" not in roadmap_text:
        print("expected Kolme integration roadmap to reference runtime commit contract lane command", file=sys.stderr)
        return 1
    if "fixtures/kolme_commit/runtime_commit_request_cases.txt" not in roadmap_text:
        print("expected Kolme integration roadmap to reference runtime commit fixture path", file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > MAX_SECONDS:
        print(f"Kolme runtime commit contract lane exceeded runtime budget: {elapsed_seconds}s", file=sys.stderr)
        return 1

    print("Kolme runtime commit contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
