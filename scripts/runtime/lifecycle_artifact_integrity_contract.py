#!/usr/bin/env python3
"""Tamper-evident lifecycle artifact generator and integrity checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_enum,
    require_non_negative_int,
    require_pattern,
    write_json,
)

SCHEMA_VERSION = "kamn.runtime.lifecycle-artifact-integrity-evidence.v1"
ARTIFACT_SCHEMA_VERSION = "kamn.runtime.lifecycle-artifact-integrity-schema.v1"
REASON_TAXONOMY_VERSION = "kamn.runtime.lifecycle-artifact-integrity-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "lifecycle_artifact_required_field_missing,"
    "lifecycle_artifact_marker_mismatch,"
    "lifecycle_artifact_hash_mismatch,"
    "lifecycle_artifact_reason_taxonomy_mismatch,"
    "lifecycle_artifact_reason_codes_csv_mismatch,"
    "lifecycle_artifact_expected_decision_mismatch"
)


def _reason_fail(reason_code: str, detail: str) -> None:
    fail(f"reason_code={reason_code} detail={detail}")


def _sha256(value: str) -> str:
    return f"sha256:{hashlib.sha256(value.encode('utf-8')).hexdigest()}"


def _compute_hashes(
    *, artifact_id: str, lifecycle_stage: str, profile: str, record_count: int, ci_fast_gate: str
) -> tuple[str, str, str]:
    payload_material = "|".join(
        [
            artifact_id,
            lifecycle_stage,
            profile,
            str(record_count),
            ci_fast_gate,
            SCHEMA_VERSION,
            ARTIFACT_SCHEMA_VERSION,
            REASON_TAXONOMY_VERSION,
            REASON_CODES_CSV,
        ]
    )
    payload_hash = _sha256(payload_material)
    integrity_hash = _sha256(
        "|".join(
            [
                payload_hash,
                artifact_id,
                lifecycle_stage,
                profile,
                str(record_count),
                ci_fast_gate,
            ]
        )
    )
    provenance_hash = _sha256(
        "|".join(
            [
                integrity_hash,
                REASON_TAXONOMY_VERSION,
                REASON_CODES_CSV,
                "kamn.runtime.lifecycle-artifact-integrity",
            ]
        )
    )
    return payload_hash, integrity_hash, provenance_hash


def _require_string_field(payload: dict[str, object], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        _reason_fail(
            "lifecycle_artifact_required_field_missing", f"{key} must be a non-empty string"
        )
    return value


def _require_int_field(payload: dict[str, object], key: str) -> int:
    value = payload.get(key)
    if not isinstance(value, int):
        _reason_fail("lifecycle_artifact_required_field_missing", f"{key} must be an integer")
    if value < 0:
        _reason_fail(
            "lifecycle_artifact_required_field_missing", f"{key} must be >= 0"
        )
    return value


def _require_hash_field(payload: dict[str, object], key: str) -> str:
    value = _require_string_field(payload, key)
    if not value.startswith("sha256:") or len(value) != len("sha256:") + 64:
        _reason_fail(
            "lifecycle_artifact_marker_mismatch",
            f"{key} must be sha256:<64 lowercase hex characters>",
        )
    return value


def generate_bundle(args: argparse.Namespace) -> int:
    artifact_id = require_pattern(
        "artifact-id",
        args.artifact_id,
        r"[A-Za-z0-9._:-]+",
        "artifact-id must be URL-safe alphanumeric token",
    )
    lifecycle_stage = require_enum(
        "lifecycle-stage", args.lifecycle_stage, ("ingestion", "retention", "deletion")
    )
    profile = require_enum("profile", args.profile, ("baseline", "elevated-risk"))
    record_count = require_non_negative_int("record-count", str(args.record_count))
    ci_fast_gate = require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    payload_hash, integrity_hash, provenance_hash = _compute_hashes(
        artifact_id=artifact_id,
        lifecycle_stage=lifecycle_stage,
        profile=profile,
        record_count=record_count,
        ci_fast_gate=ci_fast_gate,
    )

    payload = {
        "schema_version": SCHEMA_VERSION,
        "artifact_schema_version": ARTIFACT_SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "artifact_id": artifact_id,
        "lifecycle_stage": lifecycle_stage,
        "profile": profile,
        "record_count": record_count,
        "ci_fast_gate": ci_fast_gate,
        "payload_hash_sha256": payload_hash,
        "integrity_hash_sha256": integrity_hash,
        "provenance_hash_sha256": provenance_hash,
        "reason_codes_value": "none",
        "final_decision": "GO",
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print("final_decision=GO")
    return 0


def check_bundle(args: argparse.Namespace) -> int:
    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        _reason_fail(
            "lifecycle_artifact_required_field_missing",
            f"bundle file not found: {bundle_path}",
        )

    payload_obj = load_json(bundle_path)
    payload: dict[str, object] = dict(payload_obj)

    for key in (
        "schema_version",
        "artifact_schema_version",
        "reason_taxonomy_version",
        "reason_codes_csv",
        "artifact_id",
        "lifecycle_stage",
        "profile",
        "record_count",
        "ci_fast_gate",
        "payload_hash_sha256",
        "integrity_hash_sha256",
        "provenance_hash_sha256",
        "final_decision",
    ):
        if key not in payload:
            _reason_fail("lifecycle_artifact_required_field_missing", f"missing field {key}")

    schema_version = _require_string_field(payload, "schema_version")
    if schema_version != SCHEMA_VERSION:
        _reason_fail(
            "lifecycle_artifact_marker_mismatch",
            f"unexpected schema_version={schema_version}",
        )

    artifact_schema_version = _require_string_field(payload, "artifact_schema_version")
    if artifact_schema_version != ARTIFACT_SCHEMA_VERSION:
        _reason_fail(
            "lifecycle_artifact_marker_mismatch",
            f"unexpected artifact_schema_version={artifact_schema_version}",
        )

    reason_taxonomy_version = _require_string_field(payload, "reason_taxonomy_version")
    if reason_taxonomy_version != REASON_TAXONOMY_VERSION:
        _reason_fail(
            "lifecycle_artifact_reason_taxonomy_mismatch",
            f"unexpected reason_taxonomy_version={reason_taxonomy_version}",
        )

    reason_codes_csv = _require_string_field(payload, "reason_codes_csv")
    if reason_codes_csv != REASON_CODES_CSV:
        _reason_fail(
            "lifecycle_artifact_reason_codes_csv_mismatch",
            f"unexpected reason_codes_csv={reason_codes_csv}",
        )

    artifact_id = require_pattern(
        "artifact_id",
        _require_string_field(payload, "artifact_id"),
        r"[A-Za-z0-9._:-]+",
        "artifact_id must be URL-safe alphanumeric token",
    )
    lifecycle_stage = require_enum(
        "lifecycle_stage",
        _require_string_field(payload, "lifecycle_stage"),
        ("ingestion", "retention", "deletion"),
    )
    profile = require_enum(
        "profile",
        _require_string_field(payload, "profile"),
        ("baseline", "elevated-risk"),
    )
    record_count = _require_int_field(payload, "record_count")
    ci_fast_gate = require_enum(
        "ci_fast_gate", _require_string_field(payload, "ci_fast_gate"), ("PASS", "FAIL")
    )

    payload_hash = _require_hash_field(payload, "payload_hash_sha256")
    integrity_hash = _require_hash_field(payload, "integrity_hash_sha256")
    provenance_hash = _require_hash_field(payload, "provenance_hash_sha256")

    expected_payload_hash, expected_integrity_hash, expected_provenance_hash = _compute_hashes(
        artifact_id=artifact_id,
        lifecycle_stage=lifecycle_stage,
        profile=profile,
        record_count=record_count,
        ci_fast_gate=ci_fast_gate,
    )

    if payload_hash != expected_payload_hash:
        _reason_fail(
            "lifecycle_artifact_hash_mismatch",
            "payload_hash_sha256 does not match deterministic recomputation",
        )
    if integrity_hash != expected_integrity_hash:
        _reason_fail(
            "lifecycle_artifact_hash_mismatch",
            "integrity_hash_sha256 does not match deterministic recomputation",
        )
    if provenance_hash != expected_provenance_hash:
        _reason_fail(
            "lifecycle_artifact_hash_mismatch",
            "provenance_hash_sha256 does not match deterministic recomputation",
        )

    final_decision = require_enum(
        "final_decision", _require_string_field(payload, "final_decision"), ("GO", "NO-GO")
    )
    if final_decision != "GO":
        _reason_fail(
            "lifecycle_artifact_expected_decision_mismatch",
            f"generated artifact expected final_decision=GO but found {final_decision}",
        )

    expected_final_decision = require_enum(
        "expected-final-decision", args.expected_final_decision, ("GO", "NO-GO")
    )
    if final_decision != expected_final_decision:
        _reason_fail(
            "lifecycle_artifact_expected_decision_mismatch",
            f"expected final_decision={expected_final_decision} but found {final_decision}",
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={final_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--artifact-id", required=True)
    generate.add_argument("--lifecycle-stage", required=True)
    generate.add_argument("--profile", required=True)
    generate.add_argument("--record-count", required=True)
    generate.add_argument("--ci-fast-gate", required=True)
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file", required=True)
    check.add_argument("--expected-final-decision", required=True)
    check.set_defaults(handler=check_bundle)

    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
