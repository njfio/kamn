#!/usr/bin/env python3
"""Contract lane runner for Kolme notifications consumer checks."""

from __future__ import annotations

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


def parse_max_seconds() -> int:
    raw_value = os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip()
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError(f"{MAX_SECONDS_ENV} must be a positive integer")
    return int(raw_value)


def main() -> int:
    try:
        max_seconds = parse_max_seconds()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    if not RUNTIME_NETWORK_DOC.is_file():
        print("expected runtime network documentation to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    result = subprocess.run(
        ["cargo", "test", "-p", "kamn-core", "--test", TEST_TARGET],
        cwd=ROOT_DIR,
        check=False,
        stdout=subprocess.DEVNULL,
    )
    if result.returncode != 0:
        return result.returncode

    doc_text = RUNTIME_NETWORK_DOC.read_text(encoding="utf-8")
    if "run_notifications_consumer_contract_lane.sh" not in doc_text:
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

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"Kolme notifications consumer contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("Kolme notifications consumer contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
