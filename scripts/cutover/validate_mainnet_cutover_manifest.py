#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

MANIFEST_SCHEMA_VERSION = "kamn.mainnet-cutover.manifest.v1"
REPORT_SCHEMA_VERSION = "kamn.mainnet-cutover.validation-report.v1"
ALLOWED_ROLES = ("processor", "listener", "approver")
ALLOWED_STATUSES = ("PENDING", "READY", "COMPLETED", "FAILED")


class ValidationError(Exception):
    """Raised when manifest validation fails."""


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ValidationError(message)


def _parse_json(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValidationError(f"manifest is not valid JSON: {exc.msg}") from exc
    _require(isinstance(payload, dict), "manifest root must be an object")
    return payload


def _validate_manifest(payload: dict[str, Any]) -> list[dict[str, Any]]:
    required_fields = (
        "schema_version",
        "manifest_id",
        "release_candidate",
        "quorum_policy",
        "checkpoints",
    )
    for field in required_fields:
        _require(field in payload, f"missing manifest field: {field}")

    _require(
        payload["schema_version"] == MANIFEST_SCHEMA_VERSION,
        f"unexpected schema_version: {payload['schema_version']}",
    )

    manifest_id = payload["manifest_id"]
    _require(isinstance(manifest_id, str) and manifest_id.strip(), "manifest_id must be a non-empty string")

    release_candidate = payload["release_candidate"]
    _require(
        isinstance(release_candidate, str) and release_candidate.strip(),
        "release_candidate must be a non-empty string",
    )

    quorum_policy = payload["quorum_policy"]
    _require(isinstance(quorum_policy, dict), "quorum_policy must be an object")

    role_quorum: dict[str, int] = {}
    for role in ALLOWED_ROLES:
        value = quorum_policy.get(role)
        _require(isinstance(value, int), f"quorum_policy.{role} must be an integer")
        _require(value >= 1, f"quorum_policy.{role} must be >= 1")
        role_quorum[role] = value

    checkpoints = payload["checkpoints"]
    _require(isinstance(checkpoints, list) and checkpoints, "checkpoints must be a non-empty array")

    normalized: list[dict[str, Any]] = []
    checkpoint_ids: set[str] = set()
    checkpoint_orders: set[int] = set()

    for item in checkpoints:
        _require(isinstance(item, dict), "each checkpoint must be an object")
        checkpoint_fields = (
            "id",
            "order",
            "role",
            "status",
            "approvals_required",
            "approvals_received",
            "approved_by",
            "depends_on",
            "rollback_ready",
        )
        for field in checkpoint_fields:
            _require(field in item, f"missing checkpoint field: {field}")

        checkpoint_id = item["id"]
        _require(
            isinstance(checkpoint_id, str) and checkpoint_id.strip(),
            "checkpoint id must be a non-empty string",
        )
        _require(checkpoint_id not in checkpoint_ids, f"duplicate checkpoint id: {checkpoint_id}")

        order = item["order"]
        _require(isinstance(order, int), f"checkpoint '{checkpoint_id}' order must be an integer")
        _require(order >= 1, f"checkpoint '{checkpoint_id}' order must be >= 1")
        _require(order not in checkpoint_orders, f"duplicate checkpoint order: {order}")

        role = item["role"]
        _require(role in ALLOWED_ROLES, f"checkpoint '{checkpoint_id}' has invalid role: {role}")

        status = item["status"]
        _require(status in ALLOWED_STATUSES, f"checkpoint '{checkpoint_id}' has invalid status: {status}")
        _require(
            status in {"READY", "COMPLETED"},
            f"checkpoint '{checkpoint_id}' status must be READY or COMPLETED for launch validation",
        )

        approvals_required = item["approvals_required"]
        approvals_received = item["approvals_received"]
        _require(
            isinstance(approvals_required, int),
            f"checkpoint '{checkpoint_id}' approvals_required must be an integer",
        )
        _require(
            isinstance(approvals_received, int),
            f"checkpoint '{checkpoint_id}' approvals_received must be an integer",
        )
        _require(
            approvals_required >= 1,
            f"checkpoint '{checkpoint_id}' approvals_required must be >= 1",
        )
        _require(
            approvals_required <= role_quorum[role],
            f"checkpoint '{checkpoint_id}' approvals_required exceeds quorum_policy.{role}",
        )
        _require(
            approvals_received >= approvals_required,
            f"insufficient approvals for checkpoint '{checkpoint_id}'",
        )

        approved_by = item["approved_by"]
        _require(
            isinstance(approved_by, list) and approved_by,
            f"checkpoint '{checkpoint_id}' approved_by must be a non-empty array",
        )
        seen_approvers: set[str] = set()
        for approver in approved_by:
            _require(
                isinstance(approver, str) and approver.strip(),
                f"checkpoint '{checkpoint_id}' approver entries must be non-empty strings",
            )
            _require(
                approver not in seen_approvers,
                f"checkpoint '{checkpoint_id}' has duplicate approver: {approver}",
            )
            seen_approvers.add(approver)
        _require(
            len(approved_by) >= approvals_received,
            f"checkpoint '{checkpoint_id}' approvals_received exceeds approved_by evidence",
        )

        depends_on = item["depends_on"]
        _require(
            isinstance(depends_on, list),
            f"checkpoint '{checkpoint_id}' depends_on must be an array",
        )
        seen_dependencies: set[str] = set()
        for dependency in depends_on:
            _require(
                isinstance(dependency, str) and dependency.strip(),
                f"checkpoint '{checkpoint_id}' depends_on entries must be non-empty strings",
            )
            _require(
                dependency != checkpoint_id,
                f"checkpoint '{checkpoint_id}' cannot depend on itself",
            )
            _require(
                dependency not in seen_dependencies,
                f"checkpoint '{checkpoint_id}' has duplicate dependency: {dependency}",
            )
            seen_dependencies.add(dependency)

        rollback_ready = item["rollback_ready"]
        _require(
            isinstance(rollback_ready, bool),
            f"checkpoint '{checkpoint_id}' rollback_ready must be a boolean",
        )
        _require(rollback_ready, f"checkpoint '{checkpoint_id}' rollback_ready must be true")

        checkpoint_ids.add(checkpoint_id)
        checkpoint_orders.add(order)

        normalized.append(
            {
                "id": checkpoint_id,
                "order": order,
                "role": role,
                "status": status,
                "approvals_required": approvals_required,
                "approvals_received": approvals_received,
                "depends_on": depends_on,
                "rollback_ready": rollback_ready,
            }
        )

    expected_orders = list(range(1, len(normalized) + 1))
    actual_orders = sorted(checkpoint_orders)
    _require(
        actual_orders == expected_orders,
        "checkpoint orders must be contiguous and start at 1",
    )

    id_to_order = {checkpoint["id"]: checkpoint["order"] for checkpoint in normalized}
    for checkpoint in normalized:
        for dependency in checkpoint["depends_on"]:
            dependency_order = id_to_order.get(dependency)
            _require(
                dependency_order is not None,
                f"unresolved dependency: checkpoint '{checkpoint['id']}' depends on unknown checkpoint '{dependency}'",
            )
            _require(
                dependency_order < checkpoint["order"],
                f"unresolved dependency: checkpoint '{checkpoint['id']}' depends on checkpoint '{dependency}' with non-prior order",
            )

    return sorted(normalized, key=lambda entry: entry["order"])


def _build_report(
    manifest_file: Path,
    payload: dict[str, Any] | None,
    checkpoints: list[dict[str, Any]] | None,
    decision: str,
    errors: list[str],
) -> dict[str, Any]:
    report: dict[str, Any] = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "manifest_file": str(manifest_file),
        "manifest_id": payload.get("manifest_id") if payload else None,
        "release_candidate": payload.get("release_candidate") if payload else None,
        "checkpoint_count": len(checkpoints or []),
        "failed_count": len(errors),
        "decision": decision,
        "errors": errors,
        "checkpoints": checkpoints or [],
    }
    return report


def _write_report(path: Path, report: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate a mainnet cutover manifest and emit deterministic CI-gating output."
    )
    parser.add_argument("--manifest", required=True, help="Path to the manifest JSON file")
    parser.add_argument("--output-json", required=True, help="Path to write validation report JSON")
    args = parser.parse_args()

    manifest_file = Path(args.manifest).resolve()
    report_file = Path(args.output_json).resolve()

    if not manifest_file.is_file():
        print(f"manifest file not found: {manifest_file}", file=sys.stderr)
        report = _build_report(manifest_file, None, None, "NO-GO", ["manifest file not found"])
        _write_report(report_file, report)
        return 1

    try:
        payload = _parse_json(manifest_file)
        checkpoints = _validate_manifest(payload)
    except ValidationError as exc:
        message = str(exc)
        report = _build_report(manifest_file, None, None, "NO-GO", [message])
        _write_report(report_file, report)
        print(message, file=sys.stderr)
        return 1

    report = _build_report(manifest_file, payload, checkpoints, "GO", [])
    _write_report(report_file, report)

    print("status=valid")
    print(f"manifest_file={manifest_file}")
    print(f"checkpoint_count={len(checkpoints)}")
    print(f"max_checkpoint_order={checkpoints[-1]['order']}")
    print("validation_decision=GO")
    print(f"output_json={report_file}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
