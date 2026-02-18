#!/usr/bin/env python3
"""Generate and validate superseded-script inventory + deletion-manifest contracts."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

INVENTORY_SCHEMA_VERSION = "kamn.ci.superseded-script-inventory.v1"
DELETION_MANIFEST_SCHEMA_VERSION = "kamn.ci.superseded-script-deletion-manifest.v1"
CHECK_REPORT_SCHEMA_VERSION = "kamn.ci.superseded-script-deletion-manifest-report.v1"
MIGRATION_GROUPS_SCHEMA_VERSION = "kamn.kolme-manifest-migration-contract-groups.v1"
LANE_OWNERSHIP_SCHEMA_VERSION = "kamn.ci.superseded-script-lane-ownership.v1"
REASON_TAXONOMY_VERSION = (
    "kamn.ci.superseded-script-deletion-manifest-reason-taxonomy.v1"
)
REASON_CODES_CSV = (
    "superseded_deletion_manifest_entry_invalid,"
    "superseded_deletion_manifest_reason_invalid,"
    "superseded_deletion_manifest_references_unknown_script,"
    "superseded_deletion_manifest_schema_invalid,"
    "superseded_inventory_replacement_evidence_missing,"
    "superseded_inventory_schema_invalid"
)
DEFAULT_MIGRATION_GROUPS_FILE = "fixtures/ci/kolme_manifest_migration_contract_groups.json"
DEFAULT_LANE_OWNERSHIP_FILE = "fixtures/ci/superseded_script_lane_ownership.json"
DEFAULT_INVENTORY_FILE = "fixtures/ci/superseded_script_inventory_baseline.json"
DEFAULT_DELETION_MANIFEST_FILE = "fixtures/ci/superseded_script_deletion_manifest.json"
DEFAULT_SUPERSESSION_REASON_CODE = "superseded_by_manifest_lane_runner"
ALLOWED_DELETION_REASON_CODES = {DEFAULT_SUPERSESSION_REASON_CODE}
MANIFEST_RUNNER_MARKER = "scripts/framework/run_manifest_lane.sh"


class CheckerError(RuntimeError):
    """Raised for deterministic checker failures."""


@dataclass(frozen=True)
class LaneOwnershipRule:
    path_prefix: str
    owner: str


def fail(message: str) -> None:
    raise CheckerError(message)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate/check superseded-script inventory and deletion-manifest contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate_parser = subparsers.add_parser(
        "generate",
        help="Generate deterministic superseded-script inventory from migration metadata.",
    )
    generate_parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root used to resolve relative paths.",
    )
    generate_parser.add_argument(
        "--migration-groups-file",
        default=DEFAULT_MIGRATION_GROUPS_FILE,
        help="Path to migration-groups metadata JSON.",
    )
    generate_parser.add_argument(
        "--lane-ownership-file",
        default=DEFAULT_LANE_OWNERSHIP_FILE,
        help="Path to lane-ownership mapping JSON.",
    )
    generate_parser.add_argument(
        "--output-json",
        required=True,
        help="Path to write generated superseded-script inventory JSON.",
    )

    check_parser = subparsers.add_parser(
        "check",
        help="Validate deletion manifest against superseded-script inventory.",
    )
    check_parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root used to resolve relative paths.",
    )
    check_parser.add_argument(
        "--inventory-file",
        default=DEFAULT_INVENTORY_FILE,
        help="Path to superseded-script inventory JSON.",
    )
    check_parser.add_argument(
        "--deletion-manifest-file",
        default=DEFAULT_DELETION_MANIFEST_FILE,
        help="Path to superseded-script deletion manifest JSON.",
    )
    check_parser.add_argument(
        "--output-json",
        required=True,
        help="Path to write deletion-manifest check report JSON.",
    )

    return parser.parse_args(argv)


def resolve_optional_path(*, repo_root: Path, value: str) -> Path:
    path = Path(value)
    if not path.is_absolute():
        path = (repo_root / path).resolve()
    return path


def to_repo_relative(path: Path, repo_root: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def load_json_object(path: Path, *, label: str) -> dict[str, Any]:
    if not path.is_file():
        fail(f"{label} not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{label} must be valid JSON object: {path}: {exc}")
    if not isinstance(payload, dict):
        fail(f"{label} must be a JSON object: {path}")
    return payload


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def require_non_empty_string(
    value: Any,
    *,
    label: str,
) -> str:
    if not isinstance(value, str) or not value.strip():
        fail(f"expected non-empty string for {label}")
    return value.strip()


def parse_migration_group_lanes(payload: dict[str, Any]) -> list[dict[str, str]]:
    if payload.get("schema_version") != MIGRATION_GROUPS_SCHEMA_VERSION:
        fail(
            "unexpected migration groups schema version; "
            f"expected {MIGRATION_GROUPS_SCHEMA_VERSION}"
        )

    groups = payload.get("groups")
    if not isinstance(groups, dict) or not groups:
        fail("migration groups payload must include a non-empty groups object")

    parsed: list[dict[str, str]] = []
    for group_key in sorted(groups.keys()):
        group_value = groups[group_key]
        if not isinstance(group_value, dict):
            fail(f"migration group {group_key} must be an object")
        lanes = group_value.get("lanes")
        if not isinstance(lanes, list) or not lanes:
            fail(f"migration group {group_key} must contain a non-empty lanes array")

        for lane_index, lane in enumerate(lanes):
            if not isinstance(lane, dict):
                fail(f"migration group {group_key} lanes[{lane_index}] must be an object")
            lane_script = require_non_empty_string(
                lane.get("lane_script"),
                label=f"migration group {group_key} lanes[{lane_index}].lane_script",
            )
            manifest_file = require_non_empty_string(
                lane.get("manifest_file"),
                label=f"migration group {group_key} lanes[{lane_index}].manifest_file",
            )
            lane_id = require_non_empty_string(
                lane.get("lane_id"),
                label=f"migration group {group_key} lanes[{lane_index}].lane_id",
            )
            contract_script = require_non_empty_string(
                lane.get("contract_script"),
                label=f"migration group {group_key} lanes[{lane_index}].contract_script",
            )
            parsed.append(
                {
                    "group_key": group_key,
                    "lane_script": lane_script,
                    "manifest_file": manifest_file,
                    "lane_id": lane_id,
                    "contract_script": contract_script,
                }
            )

    parsed.sort(key=lambda entry: (entry["lane_script"], entry["lane_id"], entry["group_key"]))
    return parsed


def parse_lane_ownership_rules(payload: dict[str, Any]) -> list[LaneOwnershipRule]:
    if payload.get("schema_version") != LANE_OWNERSHIP_SCHEMA_VERSION:
        fail(
            "unexpected lane ownership mapping schema version; "
            f"expected {LANE_OWNERSHIP_SCHEMA_VERSION}"
        )

    raw_rules = payload.get("ownership_rules")
    if not isinstance(raw_rules, list) or not raw_rules:
        fail("lane ownership mapping must contain a non-empty ownership_rules array")

    parsed_rules: list[LaneOwnershipRule] = []
    for index, raw_rule in enumerate(raw_rules):
        if not isinstance(raw_rule, dict):
            fail(f"lane ownership mapping ownership_rules[{index}] must be an object")
        path_prefix = require_non_empty_string(
            raw_rule.get("path_prefix"),
            label=f"lane ownership mapping ownership_rules[{index}].path_prefix",
        )
        owner = require_non_empty_string(
            raw_rule.get("owner"),
            label=f"lane ownership mapping ownership_rules[{index}].owner",
        )
        parsed_rules.append(
            LaneOwnershipRule(
                path_prefix=path_prefix,
                owner=owner,
            )
        )

    parsed_rules.sort(key=lambda rule: (-len(rule.path_prefix), rule.path_prefix))
    return parsed_rules


def resolve_owner_for_lane(
    lane_script: str,
    rules: list[LaneOwnershipRule],
) -> tuple[str, str]:
    matches = [rule for rule in rules if lane_script.startswith(rule.path_prefix)]
    if not matches:
        fail(f"no ownership mapping rule matched superseded script path: {lane_script}")
    best = matches[0]
    if len(matches) > 1 and len(matches[0].path_prefix) == len(matches[1].path_prefix):
        fail(
            "ambiguous ownership mapping rules with identical prefix length for "
            f"superseded script path: {lane_script}"
        )
    return best.owner, best.path_prefix


def build_inventory_payload(
    *,
    repo_root: Path,
    migration_groups_path: Path,
    lane_ownership_path: Path,
) -> dict[str, Any]:
    migration_payload = load_json_object(migration_groups_path, label="migration groups file")
    lanes = parse_migration_group_lanes(migration_payload)

    ownership_payload = load_json_object(lane_ownership_path, label="lane ownership file")
    ownership_rules = parse_lane_ownership_rules(ownership_payload)

    entries: list[dict[str, Any]] = []
    for lane in lanes:
        manifest_file_path = resolve_optional_path(repo_root=repo_root, value=lane["manifest_file"])
        contract_script_path = resolve_optional_path(
            repo_root=repo_root,
            value=lane["contract_script"],
        )
        if not manifest_file_path.is_file():
            fail(
                "manifest file missing for superseded inventory generation: "
                f"{manifest_file_path}"
            )
        if not contract_script_path.is_file():
            fail(
                "contract script missing for superseded inventory generation: "
                f"{contract_script_path}"
            )

        owner, owner_rule_prefix = resolve_owner_for_lane(lane["lane_script"], ownership_rules)
        entries.append(
            {
                "script_path": lane["lane_script"],
                "supersession_reason_code": DEFAULT_SUPERSESSION_REASON_CODE,
                "replacement_evidence": {
                    "contract_script": lane["contract_script"],
                    "lane_id": lane["lane_id"],
                    "manifest_file": lane["manifest_file"],
                    "migration_group": lane["group_key"],
                    "ownership_rule_prefix": owner_rule_prefix,
                    "owner": owner,
                    "replacement_runner": MANIFEST_RUNNER_MARKER,
                },
            }
        )

    entries.sort(key=lambda entry: entry["script_path"])
    return {
        "schema_version": INVENTORY_SCHEMA_VERSION,
        "migration_groups_schema_version": MIGRATION_GROUPS_SCHEMA_VERSION,
        "lane_ownership_schema_version": LANE_OWNERSHIP_SCHEMA_VERSION,
        "generated_from": {
            "migration_groups_file": to_repo_relative(migration_groups_path, repo_root),
            "lane_ownership_file": to_repo_relative(lane_ownership_path, repo_root),
        },
        "superseded_script_count": len(entries),
        "superseded_scripts": entries,
    }


def validate_inventory_payload(payload: dict[str, Any]) -> tuple[dict[str, dict[str, Any]], list[str]]:
    reasons: list[str] = []
    if payload.get("schema_version") != INVENTORY_SCHEMA_VERSION:
        return {}, ["superseded_inventory_schema_invalid"]

    superseded_scripts = payload.get("superseded_scripts")
    superseded_script_count = payload.get("superseded_script_count")
    if not isinstance(superseded_scripts, list) or not isinstance(superseded_script_count, int):
        return {}, ["superseded_inventory_schema_invalid"]

    seen_paths: set[str] = set()
    inventory_by_path: dict[str, dict[str, Any]] = {}
    for index, entry in enumerate(superseded_scripts):
        if not isinstance(entry, dict):
            reasons.append("superseded_inventory_schema_invalid")
            continue
        script_path = entry.get("script_path")
        reason_code = entry.get("supersession_reason_code")
        replacement_evidence = entry.get("replacement_evidence")
        if not isinstance(script_path, str) or not script_path.strip():
            reasons.append("superseded_inventory_schema_invalid")
            continue
        script_path = script_path.strip()
        if script_path in seen_paths:
            reasons.append("superseded_inventory_schema_invalid")
            continue
        seen_paths.add(script_path)
        if not isinstance(reason_code, str) or not reason_code.strip():
            reasons.append("superseded_inventory_schema_invalid")
            continue
        if not isinstance(replacement_evidence, dict):
            reasons.append("superseded_inventory_replacement_evidence_missing")
            continue

        required_evidence_keys = (
            "contract_script",
            "lane_id",
            "manifest_file",
            "migration_group",
            "owner",
            "ownership_rule_prefix",
            "replacement_runner",
        )
        missing_evidence = False
        for key in required_evidence_keys:
            value = replacement_evidence.get(key)
            if not isinstance(value, str) or not value.strip():
                missing_evidence = True
                break
        if missing_evidence:
            reasons.append("superseded_inventory_replacement_evidence_missing")
            continue

        inventory_by_path[script_path] = {
            "index": index,
            "reason_code": reason_code.strip(),
            "replacement_evidence": replacement_evidence,
        }

    if len(superseded_scripts) != superseded_script_count:
        reasons.append("superseded_inventory_schema_invalid")
    return inventory_by_path, sorted(set(reasons))


def validate_deletion_manifest_payload(
    payload: dict[str, Any],
) -> tuple[list[dict[str, str]], list[str]]:
    reasons: list[str] = []
    if payload.get("schema_version") != DELETION_MANIFEST_SCHEMA_VERSION:
        return [], ["superseded_deletion_manifest_schema_invalid"]

    deletions = payload.get("deletions")
    if not isinstance(deletions, list):
        return [], ["superseded_deletion_manifest_schema_invalid"]

    normalized_deletions: list[dict[str, str]] = []
    seen_paths: set[str] = set()
    for entry in deletions:
        if not isinstance(entry, dict):
            reasons.append("superseded_deletion_manifest_entry_invalid")
            continue
        script_path = entry.get("script_path")
        reason_code = entry.get("reason_code")
        if not isinstance(script_path, str) or not script_path.strip():
            reasons.append("superseded_deletion_manifest_entry_invalid")
            continue
        if not isinstance(reason_code, str) or not reason_code.strip():
            reasons.append("superseded_deletion_manifest_entry_invalid")
            continue
        script_path = script_path.strip()
        reason_code = reason_code.strip()
        if script_path in seen_paths:
            reasons.append("superseded_deletion_manifest_entry_invalid")
            continue
        seen_paths.add(script_path)
        if reason_code not in ALLOWED_DELETION_REASON_CODES:
            reasons.append("superseded_deletion_manifest_reason_invalid")
        normalized_deletions.append(
            {
                "script_path": script_path,
                "reason_code": reason_code,
            }
        )
    return normalized_deletions, sorted(set(reasons))


def run_generate(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    if not repo_root.is_dir():
        fail(f"repo root is not a directory: {repo_root}")

    migration_groups_path = resolve_optional_path(
        repo_root=repo_root,
        value=args.migration_groups_file,
    )
    lane_ownership_path = resolve_optional_path(
        repo_root=repo_root,
        value=args.lane_ownership_file,
    )
    output_json_path = resolve_optional_path(repo_root=repo_root, value=args.output_json)

    payload = build_inventory_payload(
        repo_root=repo_root,
        migration_groups_path=migration_groups_path,
        lane_ownership_path=lane_ownership_path,
    )
    write_json(output_json_path, payload)

    print("status=generated")
    print(f"inventory_entry_count={payload['superseded_script_count']}")
    print("reason_codes=none")
    print(f"output_json={output_json_path}")
    return 0


def run_check(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    if not repo_root.is_dir():
        fail(f"repo root is not a directory: {repo_root}")

    inventory_path = resolve_optional_path(repo_root=repo_root, value=args.inventory_file)
    deletion_manifest_path = resolve_optional_path(
        repo_root=repo_root,
        value=args.deletion_manifest_file,
    )
    output_json_path = resolve_optional_path(repo_root=repo_root, value=args.output_json)

    inventory_payload = load_json_object(inventory_path, label="superseded inventory file")
    inventory_by_path, inventory_reasons = validate_inventory_payload(inventory_payload)

    deletion_manifest_payload = load_json_object(
        deletion_manifest_path,
        label="superseded deletion manifest file",
    )
    deletion_entries, deletion_manifest_reasons = validate_deletion_manifest_payload(
        deletion_manifest_payload
    )

    reason_codes: list[str] = [*inventory_reasons, *deletion_manifest_reasons]
    unknown_manifest_entries: list[str] = []
    reason_mismatch_entries: list[str] = []
    for deletion in deletion_entries:
        inventory_entry = inventory_by_path.get(deletion["script_path"])
        if inventory_entry is None:
            unknown_manifest_entries.append(deletion["script_path"])
            continue
        if inventory_entry["reason_code"] != deletion["reason_code"]:
            reason_mismatch_entries.append(deletion["script_path"])

    if unknown_manifest_entries:
        reason_codes.append("superseded_deletion_manifest_references_unknown_script")
    if reason_mismatch_entries:
        reason_codes.append("superseded_deletion_manifest_reason_invalid")

    reason_codes = sorted(set(reason_codes))
    status = "ok" if not reason_codes else "fail"
    final_decision = "GO" if status == "ok" else "NO-GO"
    reason_codes_value = "none" if not reason_codes else ",".join(reason_codes)
    report_payload = {
        "schema_version": CHECK_REPORT_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": reason_codes_value,
        "metrics": {
            "inventory_entry_count": len(inventory_by_path),
            "deletion_entry_count": len(deletion_entries),
            "unknown_manifest_entry_count": len(unknown_manifest_entries),
            "reason_mismatch_entry_count": len(reason_mismatch_entries),
            "invalid_inventory_entry_count": len(inventory_reasons),
        },
        "unknown_manifest_entries": unknown_manifest_entries,
        "reason_mismatch_entries": reason_mismatch_entries,
    }
    write_json(output_json_path, report_payload)

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes={reason_codes_value}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"inventory_entry_count={len(inventory_by_path)}")
    print(f"deletion_entry_count={len(deletion_entries)}")
    print(f"unknown_manifest_entry_count={len(unknown_manifest_entries)}")
    print(f"reason_mismatch_entry_count={len(reason_mismatch_entries)}")
    print(f"invalid_inventory_entry_count={len(inventory_reasons)}")
    print(f"output_json={output_json_path}")

    if status == "fail":
        for script_path in unknown_manifest_entries:
            print(f"unknown_manifest_script={script_path}", file=sys.stderr)
        for script_path in reason_mismatch_entries:
            print(f"reason_mismatch_script={script_path}", file=sys.stderr)
        return 1
    return 0


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        if args.command == "generate":
            return run_generate(args)
        if args.command == "check":
            return run_check(args)
    except CheckerError as error:
        if args.command == "check":
            print("status=fail")
            print("final_decision=NO-GO")
            print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
            print("reason_codes=checker_configuration_invalid")
            print(f"reason_codes_csv={REASON_CODES_CSV}")
            print("inventory_entry_count=0")
            print("deletion_entry_count=0")
            print("unknown_manifest_entry_count=0")
            print("reason_mismatch_entry_count=0")
            print("invalid_inventory_entry_count=0")
            print(f"error={error}")
            return 1
        print("status=fail")
        print("reason_codes=checker_configuration_invalid")
        print(f"error={error}")
        return 1

    raise AssertionError(f"unhandled command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
