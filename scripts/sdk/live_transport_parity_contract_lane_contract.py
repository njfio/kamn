#!/usr/bin/env python3
"""Live transport parity fast-lane runner contract."""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys


SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
USAGE_LINE = (
    "usage: run_live_transport_parity_contract_lane.sh "
    "[--languages <rust,python,typescript>]"
)
ALLOWED_LANGUAGES = ("rust", "python", "typescript")


def usage() -> int:
    """Print usage to stderr and return shell-compatible usage exit code."""
    print(USAGE_LINE, file=sys.stderr)
    return 2


def normalize_languages(raw: str) -> str:
    """Normalize language selector while preserving deterministic order."""
    if raw == "" or raw == "all":
        return ",".join(ALLOWED_LANGUAGES)

    normalized: list[str] = []
    seen: set[str] = set()
    for token in raw.split(","):
        value = token.strip().lower()
        if value == "":
            continue
        if value not in ALLOWED_LANGUAGES:
            raise ValueError(f"unsupported language selector: {token}")
        if value not in seen:
            normalized.append(value)
            seen.add(value)

    if not normalized:
        raise ValueError("at least one language must be selected")

    return ",".join(normalized)


def parse_languages(argv: list[str]) -> str:
    """Parse --languages argument with shell-compatible errors."""
    languages = "all"
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--languages":
            if index + 1 >= len(argv):
                raise ValueError("")
            languages = argv[index + 1]
            index += 2
            continue
        raise ValueError("")
    return languages


def run_checked(command: list[str]) -> None:
    """Run command in repo root and fail fast on non-zero exit."""
    subprocess.run(command, cwd=ROOT_DIR, check=True)


def main(argv: list[str]) -> int:
    try:
        raw_languages = parse_languages(argv)
    except ValueError as error:
        if str(error):
            print(str(error), file=sys.stderr)
        return usage()

    try:
        selected_languages = normalize_languages(raw_languages)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return usage()

    run_rust = False
    run_python = False
    run_typescript = False
    for language in selected_languages.split(","):
        if language == "rust":
            run_rust = True
        elif language == "python":
            run_python = True
        elif language == "typescript":
            run_typescript = True

    try:
        if run_rust:
            print("running rust live transport contract lane tests")
            run_checked(["bash", "scripts/sdk/run_rust_live_transport_contract_lane.sh"])

        if run_python:
            print("running python live transport contract lane tests")
            run_checked(["python3", "-m", "unittest", "tests/python/test_sdk.py"])

        if run_typescript:
            print("running typescript live transport contract lane tests")
            run_checked(["npm", "--prefix", "packages/kamn-sdk", "test"])

        print("running transport profile parity drift matrix checks")
        run_checked(
            [
                "bash",
                "scripts/sdk/run_transport_profile_parity_matrix.sh",
                "--languages",
                selected_languages,
            ]
        )
    except subprocess.CalledProcessError as error:
        return error.returncode

    print(
        "live transport parity contract lane tests passed for languages: "
        f"{selected_languages}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
