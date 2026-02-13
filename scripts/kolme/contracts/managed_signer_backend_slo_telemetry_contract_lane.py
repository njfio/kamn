#!/usr/bin/env python3
"""Contract lane for managed-signer backend SLO telemetry bundle generation."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
GENERATOR = ROOT_DIR / "scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh"
DOC_FILES = [
    ROOT_DIR / "docs/ci/ci-cost-and-lane-framework.md",
    ROOT_DIR / "docs/planning/kolme-devnet-ops.md",
    ROOT_DIR / "README.md",
]


def run_generator(bundle_path: Path, **kwargs: str) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(GENERATOR),
        "--output-file",
        str(bundle_path),
    ]
    for key, value in kwargs.items():
        option_name = key.replace("_", "-")
        command.extend([f"--{option_name}", value])
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )


def ensure_markers_present(text: str, markers: list[str], source_name: str) -> list[str]:
    missing: list[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(f"{source_name}_missing_marker:{marker}")
    return missing


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run managed-signer backend SLO telemetry bundle contract lane."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/managed-signer-backend-slo-contract-report.json",
    )
    args = parser.parse_args()

    if not GENERATOR.is_file() or not GENERATOR.stat().st_mode & 0o111:
        print("expected managed-signer backend SLO telemetry generator to be executable", file=sys.stderr)
        return 1

    required_markers = [
        "generate_managed_signer_backend_slo_telemetry_bundle.sh",
        "run_managed_signer_backend_slo_telemetry_contract_lane.sh",
        "kamn.kolme.managed-signer-backend-slo-telemetry.v1",
        "managed_signer_backend_timeout_rate_threshold_exceeded",
        "managed_signer_backend_unavailable_rate_threshold_exceeded",
        "managed_signer_backend_error_rate_threshold_exceeded",
        "managed_signer_backend_ci_fast_gate_failed",
        "signer_key_source=managed-external",
        "contracts.required_signer_key_source=managed-external",
    ]

    missing_doc_markers: list[str] = []
    for doc_file in DOC_FILES:
        if not doc_file.is_file():
            print(f"expected documentation file to exist: {doc_file}", file=sys.stderr)
            return 1
        missing_doc_markers.extend(
            ensure_markers_present(
                doc_file.read_text(encoding="utf-8"),
                required_markers,
                str(doc_file.relative_to(ROOT_DIR)),
            )
        )
    if missing_doc_markers:
        print(",".join(missing_doc_markers), file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="managed-signer-slo-contract-") as temp_dir:
        temp_path = Path(temp_dir)
        go_bundle = temp_path / "go.json"
        no_go_bundle = temp_path / "no-go.json"

        go_result = run_generator(
            go_bundle,
            window_start_utc="2026-02-13T00:00:00Z",
            window_end_utc="2026-02-13T00:15:00Z",
            backend_name="kolme-managed-signer-primary",
            signer_profile="ops-primary",
            signer_key_source="managed-external",
            sample_count="100",
            timeout_events="0",
            unavailable_events="0",
            error_events="1",
            max_timeout_rate_bps="100",
            max_unavailable_rate_bps="100",
            max_error_rate_bps="200",
            ci_fast_gate="PASS",
        )
        if go_result.returncode != 0:
            print("expected managed-signer SLO GO fixture generation to succeed", file=sys.stderr)
            stderr = go_result.stderr.strip()
            if stderr:
                print(stderr, file=sys.stderr)
            return 1
        if "final_decision=GO" not in go_result.stdout:
            print("expected managed-signer SLO GO fixture decision marker", file=sys.stderr)
            return 1
        go_payload = json.loads(go_bundle.read_text(encoding="utf-8"))
        if go_payload.get("schema_version") != "kamn.kolme.managed-signer-backend-slo-telemetry.v1":
            print("unexpected managed-signer SLO GO fixture schema", file=sys.stderr)
            return 1
        if go_payload.get("final_decision") != "GO":
            print("expected managed-signer SLO GO fixture final decision marker", file=sys.stderr)
            return 1
        if go_payload.get("threshold_breaches") != []:
            print("expected managed-signer SLO GO fixture threshold breaches to be empty", file=sys.stderr)
            return 1

        no_go_result = run_generator(
            no_go_bundle,
            window_start_utc="2026-02-13T00:15:00Z",
            window_end_utc="2026-02-13T00:30:00Z",
            backend_name="kolme-managed-signer-primary",
            signer_profile="ops-primary",
            signer_key_source="managed-external",
            sample_count="100",
            timeout_events="8",
            unavailable_events="7",
            error_events="9",
            max_timeout_rate_bps="500",
            max_unavailable_rate_bps="500",
            max_error_rate_bps="500",
            ci_fast_gate="PASS",
        )
        if no_go_result.returncode != 0:
            print("expected managed-signer SLO NO-GO fixture generation to succeed", file=sys.stderr)
            stderr = no_go_result.stderr.strip()
            if stderr:
                print(stderr, file=sys.stderr)
            return 1
        if "final_decision=NO-GO" not in no_go_result.stdout:
            print("expected managed-signer SLO NO-GO fixture decision marker", file=sys.stderr)
            return 1
        no_go_payload = json.loads(no_go_bundle.read_text(encoding="utf-8"))
        if no_go_payload.get("final_decision") != "NO-GO":
            print("expected managed-signer SLO NO-GO fixture final decision marker", file=sys.stderr)
            return 1
        no_go_breaches = no_go_payload.get("threshold_breaches")
        if not isinstance(no_go_breaches, list):
            print("expected managed-signer SLO NO-GO fixture threshold_breaches list", file=sys.stderr)
            return 1
        required_breaches = {
            "managed_signer_backend_timeout_rate_threshold_exceeded",
            "managed_signer_backend_unavailable_rate_threshold_exceeded",
            "managed_signer_backend_error_rate_threshold_exceeded",
        }
        if set(no_go_breaches) != required_breaches:
            print("expected managed-signer SLO NO-GO fixture threshold breach set", file=sys.stderr)
            return 1

        contract_report = {
            "schema_version": "kamn.kolme.managed-signer-backend-slo-contract-report.v1",
            "go_fixture_bundle": str(go_bundle),
            "no_go_fixture_bundle": str(no_go_bundle),
            "final_decision": "GO",
        }
        output_path = Path(args.output_json).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(contract_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    print("managed-signer backend SLO telemetry contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
