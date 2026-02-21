#!/usr/bin/env python3
"""Release go/no-go checker for SBOM/provenance artifact and docs parity contracts."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import time
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent

CHECKER_SCHEMA_VERSION = "kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1"
CHECKER_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.sbom-provenance-release-gonogo-checker-reason-taxonomy.v1"
)
CHECKER_REASON_CODES_CSV = (
    "sbom_provenance_artifact_marker_missing,"
    "sbom_provenance_artifact_marker_invalid,"
    "sbom_provenance_artifact_decision_not_go,"
    "sbom_provenance_docs_parity_marker_missing,"
    "sbom_provenance_runtime_budget_exceeded"
)

DEFAULT_ARTIFACT_PATH = ROOT_DIR / "artifacts" / "sbom-provenance-baseline.json"
DEFAULT_CI_STRATEGY_DOC = ROOT_DIR / "docs/ci/strategy.md"
DEFAULT_OPS_CONFIGURATION_DOC = ROOT_DIR / "docs/ops/configuration.md"

EXPECTED_ARTIFACT_MARKERS = {
    "schema_version": "kamn.runtime.sbom-provenance-artifact-report.v1",
    "artifact_schema_version": "kamn.runtime.sbom-provenance-artifact-schema.v1",
    "fixture_schema_version": "kamn.ci.sbom-provenance-artifact-fixture-matrix.v1",
    "reason_taxonomy_version": "kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1",
    "release_manifest_required_artifact_id": "sbom_provenance",
    "status": "pass",
    "final_decision": "GO",
    "reason_code": "none",
}
EXPECTED_ARTIFACT_MARKERS_CSV = (
    "schema_version,artifact_schema_version,fixture_schema_version,reason_taxonomy_version,"
    "release_manifest_required_artifact_id,status,final_decision,reason_code"
)

DOCS_PARITY_REQUIRED_MARKERS = (
    "sbom_provenance_release_gonogo_checker_schema_version=" + CHECKER_SCHEMA_VERSION,
    "sbom_provenance_release_gonogo_checker_reason_taxonomy_version="
    + CHECKER_REASON_TAXONOMY_VERSION,
    "sbom_provenance_release_gonogo_checker_reason_codes_csv=" + CHECKER_REASON_CODES_CSV,
    "sbom_provenance_release_gonogo_required_artifact_markers_csv="
    + EXPECTED_ARTIFACT_MARKERS_CSV,
)
DOCS_PARITY_REQUIRED_MARKERS_CSV = (
    "sbom_provenance_release_gonogo_checker_schema_version,"
    "sbom_provenance_release_gonogo_checker_reason_taxonomy_version,"
    "sbom_provenance_release_gonogo_checker_reason_codes_csv,"
    "sbom_provenance_release_gonogo_required_artifact_markers_csv"
)
DOCS_PARITY_REQUIRED_COMMAND = (
    "python3 scripts/deploy/sbom_provenance_release_gonogo_checker_contract.py "
    "--artifact-json /tmp/sbom-provenance-baseline.json --ci-strategy-doc docs/ci/strategy.md "
    "--ops-configuration-doc docs/ops/configuration.md --max-seconds 120 "
    "--output-json /tmp/sbom-provenance-release-gonogo-checker.json"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact-json", default=str(DEFAULT_ARTIFACT_PATH))
    parser.add_argument("--ci-strategy-doc", default=str(DEFAULT_CI_STRATEGY_DOC))
    parser.add_argument("--ops-configuration-doc", default=str(DEFAULT_OPS_CONFIGURATION_DOC))
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_SBOM_PROVENANCE_RELEASE_GONOGO_MAX_SECONDS", "120"),
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def parse_required_positive_int(raw_value: str, field: str) -> int:
    if not raw_value.isdigit():
        raise ValueError(f"{field} must be an integer")
    parsed = int(raw_value)
    if parsed <= 0:
        raise ValueError(f"{field} must be greater than zero")
    return parsed


def load_json_object(path: Path) -> tuple[dict[str, Any] | None, str]:
    if not path.is_file():
        return None, "file_not_found"
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None, "json_decode_error"
    if not isinstance(payload, dict):
        return None, "json_object_required"
    return payload, ""


def load_text_file(path: Path) -> tuple[str, str]:
    if not path.is_file():
        return "", "file_not_found"
    try:
        return path.read_text(encoding="utf-8"), ""
    except OSError:
        return "", "read_error"


def add_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def run_lane() -> int:
    started = time.monotonic()
    args = parse_args()

    try:
        max_seconds = parse_required_positive_int(args.max_seconds, "max-seconds")
    except ValueError as error:
        print(str(error))
        return 1

    artifact_path = Path(args.artifact_json)
    ci_doc_path = Path(args.ci_strategy_doc)
    ops_doc_path = Path(args.ops_configuration_doc)

    reasons: list[str] = []

    artifact_marker_contract_status = "verified"
    docs_parity_status = "verified"
    strategy_doc_marker_status = "verified"
    ops_configuration_doc_marker_status = "verified"
    performance_budget_status = "verified"

    artifact_missing_markers: list[str] = []
    artifact_invalid_markers: list[str] = []
    artifact_decision_mismatch_markers: list[str] = []
    strategy_doc_missing_markers: list[str] = []
    ops_configuration_doc_missing_markers: list[str] = []

    artifact_payload, artifact_load_error = load_json_object(artifact_path)
    if artifact_load_error:
        artifact_marker_contract_status = "violation"
        artifact_missing_markers.append(f"artifact_json:{artifact_load_error}")
        add_reason(reasons, "sbom_provenance_artifact_marker_missing")
    elif artifact_payload is not None:
        for marker, expected_value in EXPECTED_ARTIFACT_MARKERS.items():
            value = artifact_payload.get(marker)
            if value is None:
                artifact_missing_markers.append(marker)
                continue
            if not isinstance(value, str):
                artifact_invalid_markers.append(f"{marker}:type")
                continue
            if not value.strip():
                artifact_missing_markers.append(marker)
                continue
            if marker in {"status", "final_decision", "reason_code"} and value != expected_value:
                artifact_decision_mismatch_markers.append(marker)
            elif marker not in {"status", "final_decision", "reason_code"} and value != expected_value:
                artifact_invalid_markers.append(marker)

        if artifact_missing_markers:
            artifact_marker_contract_status = "violation"
            add_reason(reasons, "sbom_provenance_artifact_marker_missing")
        if artifact_invalid_markers:
            artifact_marker_contract_status = "violation"
            add_reason(reasons, "sbom_provenance_artifact_marker_invalid")
        if artifact_decision_mismatch_markers:
            artifact_marker_contract_status = "violation"
            add_reason(reasons, "sbom_provenance_artifact_decision_not_go")

    ci_strategy_doc, ci_strategy_doc_error = load_text_file(ci_doc_path)
    if ci_strategy_doc_error:
        docs_parity_status = "violation"
        strategy_doc_marker_status = "violation"
        strategy_doc_missing_markers.append(f"ci_strategy_doc:{ci_strategy_doc_error}")
    else:
        strategy_doc_missing_markers = [
            marker
            for marker in [*DOCS_PARITY_REQUIRED_MARKERS, DOCS_PARITY_REQUIRED_COMMAND]
            if marker not in ci_strategy_doc
        ]
        if strategy_doc_missing_markers:
            strategy_doc_marker_status = "violation"
            docs_parity_status = "violation"

    ops_configuration_doc, ops_configuration_doc_error = load_text_file(ops_doc_path)
    if ops_configuration_doc_error:
        docs_parity_status = "violation"
        ops_configuration_doc_marker_status = "violation"
        ops_configuration_doc_missing_markers.append(
            f"ops_configuration_doc:{ops_configuration_doc_error}"
        )
    else:
        ops_configuration_doc_missing_markers = [
            marker
            for marker in [*DOCS_PARITY_REQUIRED_MARKERS, DOCS_PARITY_REQUIRED_COMMAND]
            if marker not in ops_configuration_doc
        ]
        if ops_configuration_doc_missing_markers:
            ops_configuration_doc_marker_status = "violation"
            docs_parity_status = "violation"

    if docs_parity_status == "violation":
        add_reason(reasons, "sbom_provenance_docs_parity_marker_missing")

    elapsed_seconds = int(time.monotonic() - started)
    if elapsed_seconds > max_seconds:
        performance_budget_status = "violation"
        add_reason(reasons, "sbom_provenance_runtime_budget_exceeded")

    status = "pass" if not reasons else "fail"
    final_decision = "GO" if not reasons else "NO-GO"
    reason_code = "none" if not reasons else reasons[0]
    reason_codes_value = "none" if not reasons else ",".join(reasons)

    payload = {
        "schema_version": CHECKER_SCHEMA_VERSION,
        "reason_taxonomy_version": CHECKER_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": CHECKER_REASON_CODES_CSV,
        "status": status,
        "final_decision": final_decision,
        "reason_code": reason_code,
        "reason_codes_value": reason_codes_value,
        "artifact_marker_contract_status": artifact_marker_contract_status,
        "docs_parity_status": docs_parity_status,
        "strategy_doc_marker_status": strategy_doc_marker_status,
        "ops_configuration_doc_marker_status": ops_configuration_doc_marker_status,
        "performance_budget_status": performance_budget_status,
        "artifact_missing_markers": artifact_missing_markers,
        "artifact_invalid_markers": artifact_invalid_markers,
        "artifact_decision_mismatch_markers": artifact_decision_mismatch_markers,
        "strategy_doc_missing_markers": strategy_doc_missing_markers,
        "ops_configuration_doc_missing_markers": ops_configuration_doc_missing_markers,
        "required_artifact_markers_csv": EXPECTED_ARTIFACT_MARKERS_CSV,
        "docs_parity_required_markers_csv": DOCS_PARITY_REQUIRED_MARKERS_CSV,
        "docs_parity_required_command": DOCS_PARITY_REQUIRED_COMMAND,
        "artifact_json": str(artifact_path),
        "ci_strategy_doc": str(ci_doc_path),
        "ops_configuration_doc": str(ops_doc_path),
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(
            json.dumps(payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_code={reason_code}")
    print(f"reason_codes_value={reason_codes_value}")
    print(f"checker_schema_version={CHECKER_SCHEMA_VERSION}")
    print(f"checker_reason_taxonomy_version={CHECKER_REASON_TAXONOMY_VERSION}")
    print(f"checker_reason_codes_csv={CHECKER_REASON_CODES_CSV}")
    print(f"artifact_marker_contract_status={artifact_marker_contract_status}")
    print(f"docs_parity_status={docs_parity_status}")
    print(f"strategy_doc_marker_status={strategy_doc_marker_status}")
    print(f"ops_configuration_doc_marker_status={ops_configuration_doc_marker_status}")
    print(f"performance_budget_status={performance_budget_status}")
    print(f"required_artifact_markers_csv={EXPECTED_ARTIFACT_MARKERS_CSV}")
    print(f"docs_parity_required_markers_csv={DOCS_PARITY_REQUIRED_MARKERS_CSV}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"max_seconds={max_seconds}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(run_lane())
