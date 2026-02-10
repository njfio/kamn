#!/usr/bin/env python3
"""Run DID service-endpoint canonicalization vector matrix."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR))

from did_service_endpoint_canonicalization_contract import (  # noqa: E402
    _evaluate_vectors,
    _load_fixture,
)
from framework.contract_framework import write_json  # noqa: E402


def run_matrix(args: argparse.Namespace) -> int:
    fixture = Path(args.fixture)
    output_json = Path(args.output_json)

    vectors = _load_fixture(fixture)
    _, summary = _evaluate_vectors(vectors)
    final_decision = "GO" if summary["mismatch_vectors"] == 0 else "NO-GO"

    payload = {
        "schema_version": "kamn.did.service-endpoint-canonicalization-matrix-report.v1",
        "fixture_file": str(fixture.resolve()),
        "summary": summary,
        "final_decision": final_decision,
    }
    write_json(output_json, payload)

    print(f"output_json={output_json}")
    print(f"final_decision={final_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run DID service-endpoint canonicalization conformance matrix."
    )
    parser.add_argument(
        "--fixture",
        default=str(
            ROOT_DIR / "fixtures/did_core_conformance/service_endpoint_canonicalization_vectors.json"
        ),
    )
    parser.add_argument(
        "--output-json",
        default=str(ROOT_DIR / "did-service-endpoint-canonicalization-matrix-report.json"),
    )
    parser.set_defaults(handler=run_matrix)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    raise SystemExit(main())
