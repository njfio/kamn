#!/usr/bin/env python3
"""Contract lane runner for Kolme block fallback reconciliation checks."""

from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNTIME_NETWORK_DOC = ROOT_DIR / "docs/foundation/runtime-network.md"
DEVNET_DOC = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
TEST_TARGET = "kolme_runtime_commit_block_fallback"
MAX_SECONDS_ENV = "KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 75


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

    if not DEVNET_DOC.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1

    if not ROADMAP_DOC.is_file():
        print("expected Kolme integration roadmap documentation to exist", file=sys.stderr)
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

    runtime_network_doc_text = RUNTIME_NETWORK_DOC.read_text(encoding="utf-8")
    devnet_doc_text = DEVNET_DOC.read_text(encoding="utf-8")
    roadmap_doc_text = ROADMAP_DOC.read_text(encoding="utf-8")

    if "run_block_fallback_reconciliation_contract_lane.sh" not in runtime_network_doc_text:
        print(
            "expected runtime network documentation to reference block fallback reconciliation lane command",
            file=sys.stderr,
        )
        return 1
    if "run_block_fallback_reconciliation_contract_lane.sh" not in devnet_doc_text:
        print(
            "expected Kolme devnet ops documentation to reference block fallback reconciliation lane command",
            file=sys.stderr,
        )
        return 1
    if "run_block_fallback_reconciliation_contract_lane.sh" not in roadmap_doc_text:
        print(
            "expected Kolme integration roadmap to reference block fallback reconciliation lane command",
            file=sys.stderr,
        )
        return 1

    if "Regression: #1464" not in runtime_network_doc_text:
        print(
            "expected runtime network documentation to include block fallback regression marker",
            file=sys.stderr,
        )
        return 1
    if "Regression: #1464" not in roadmap_doc_text:
        print(
            "expected Kolme integration roadmap to include block fallback regression marker",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"Kolme block fallback reconciliation contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("Kolme block fallback reconciliation contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
