#!/usr/bin/env python3
"""Compatibility shim for the Rust SBOM/provenance generator harness."""

from __future__ import annotations

import os
import subprocess
import sys


def main() -> int:
    command = [
        "cargo",
        "run",
        "-p",
        "kamn-core",
        "--bin",
        "sbom_provenance_artifact_generator_contract",
        "--",
        *sys.argv[1:],
    ]
    result = subprocess.run(command, env=os.environ.copy(), check=False)
    return int(result.returncode)


if __name__ == "__main__":
    raise SystemExit(main())
