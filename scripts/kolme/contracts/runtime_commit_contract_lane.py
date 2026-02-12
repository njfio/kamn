#!/usr/bin/env python3
"""Contract lane runner for Kolme runtime commit checks."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_commit/runtime_commit_request_cases.txt"
PARITY_MATRIX_FILE = (
    ROOT_DIR / "fixtures/kolme_compatibility/runtime_commit_decomposition_parity_matrix.json"
)
PARITY_MATRIX_CHECKER = ROOT_DIR / "scripts/kolme/check_runtime_commit_decomposition_parity_matrix.py"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
FOUNDATION_DOC = ROOT_DIR / "docs/foundation/kolme-runtime-commit-client.md"
MAX_SECONDS = 60


def main() -> int:
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
    if elapsed_seconds > MAX_SECONDS:
        print(f"Kolme runtime commit contract lane exceeded runtime budget: {elapsed_seconds}s", file=sys.stderr)
        return 1

    print("Kolme runtime commit contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
