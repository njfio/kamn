#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def _parse_args() -> argparse.Namespace:
    root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(
        description="Validate deterministic triadic devnet smoke output markers."
    )
    parser.add_argument(
        "--fixture",
        default=str(root / "fixtures/kolme_compatibility/devnet_smoke_markers.json"),
    )
    parser.add_argument("--marker-file", required=True)
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def _load_required_markers(fixture_file: Path) -> list[str]:
    fixture = json.loads(fixture_file.read_text(encoding="utf-8"))
    if fixture.get("schema_version") != "kamn.kolme.triadic-devnet-smoke-markers.v1":
        raise SystemExit("unexpected triadic devnet smoke marker fixture schema")

    required_markers = fixture.get("required_markers")
    if not isinstance(required_markers, list) or not required_markers:
        raise SystemExit("required_markers must be a non-empty array")

    for marker in required_markers:
        if not isinstance(marker, str) or not marker.strip():
            raise SystemExit("required_markers entries must be non-empty strings")
    return required_markers


def _load_observed_markers(marker_file: Path) -> list[str]:
    return [
        line.strip()
        for line in marker_file.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def main() -> int:
    args = _parse_args()
    fixture_file = Path(args.fixture).resolve()
    marker_file = Path(args.marker_file).resolve()

    required_markers = _load_required_markers(fixture_file)
    observed_markers = _load_observed_markers(marker_file)
    observed_set = set(observed_markers)

    missing_markers = [marker for marker in required_markers if marker not in observed_set]
    final_decision = "PASS" if not missing_markers else "FAIL"

    report = {
        "schema_version": "kamn.kolme.triadic-devnet-smoke-validation-report.v1",
        "fixture": str(fixture_file),
        "marker_file": str(marker_file),
        "required_markers": required_markers,
        "observed_markers": observed_markers,
        "missing_markers": missing_markers,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "ok" if final_decision == "PASS" else "fail"
    missing = "none" if not missing_markers else ",".join(missing_markers)
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"missing_markers={missing}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
