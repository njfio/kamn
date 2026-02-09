#!/usr/bin/env python3
"""Governance stake/slash risk evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import re
import sys
from typing import Any, Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.governance.stake-slash-risk.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
SIMULATION_HASH_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
NON_NEGATIVE_INT_PATTERN = re.compile(r"^[0-9]+$")


def _parse_bool(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("boolean fields must be true or false")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def _parse_bps_int(raw_value: str) -> int:
    if not NON_NEGATIVE_INT_PATTERN.fullmatch(raw_value):
        fail("bps field must be an integer")
    return int(raw_value)


def _compute_policy_checks(
    *,
    simulation_hash: str,
    stake_at_risk_bps: int,
    max_stake_at_risk_bps: int,
    slash_probability_bps: int,
    max_slash_probability_bps: int,
    validator_churn_bps: int,
    max_validator_churn_bps: int,
    quorum_safety_margin_bps: int,
    min_quorum_safety_margin_bps: int,
) -> Mapping[str, bool]:
    return {
        "simulation_hash_valid": bool(SIMULATION_HASH_PATTERN.match(simulation_hash)),
        "stake_risk_within_limit": stake_at_risk_bps <= max_stake_at_risk_bps,
        "slash_probability_within_limit": (
            slash_probability_bps <= max_slash_probability_bps
        ),
        "validator_churn_within_limit": validator_churn_bps <= max_validator_churn_bps,
        "quorum_margin_within_limit": (
            quorum_safety_margin_bps >= min_quorum_safety_margin_bps
        ),
    }


def _compute_reason_codes(
    *,
    simulation_hash_valid: bool,
    stake_risk_within_limit: bool,
    slash_probability_within_limit: bool,
    validator_churn_within_limit: bool,
    quorum_margin_within_limit: bool,
    evidence_complete: bool,
    ci_fast_gate: str,
) -> list[str]:
    reason_codes: list[str] = []
    if not simulation_hash_valid:
        reason_codes.append("simulation_hash_invalid")
    if not stake_risk_within_limit:
        reason_codes.append("stake_at_risk_threshold_breach")
    if not slash_probability_within_limit:
        reason_codes.append("slash_probability_threshold_breach")
    if not validator_churn_within_limit:
        reason_codes.append("validator_churn_threshold_breach")
    if not quorum_margin_within_limit:
        reason_codes.append("quorum_safety_margin_breach")
    if not evidence_complete:
        reason_codes.append("evidence_incomplete")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")
    return reason_codes


def generate_bundle(args: argparse.Namespace) -> int:
    stake_at_risk_bps = _parse_bps_int(args.stake_at_risk_bps)
    max_stake_at_risk_bps = _parse_bps_int(args.max_stake_at_risk_bps)
    slash_probability_bps = _parse_bps_int(args.slash_probability_bps)
    max_slash_probability_bps = _parse_bps_int(args.max_slash_probability_bps)
    validator_churn_bps = _parse_bps_int(args.validator_churn_bps)
    max_validator_churn_bps = _parse_bps_int(args.max_validator_churn_bps)
    quorum_safety_margin_bps = _parse_bps_int(args.quorum_safety_margin_bps)
    min_quorum_safety_margin_bps = _parse_bps_int(args.min_quorum_safety_margin_bps)
    evidence_complete = _parse_bool(args.evidence_complete)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)

    policy_checks = _compute_policy_checks(
        simulation_hash=args.simulation_hash,
        stake_at_risk_bps=stake_at_risk_bps,
        max_stake_at_risk_bps=max_stake_at_risk_bps,
        slash_probability_bps=slash_probability_bps,
        max_slash_probability_bps=max_slash_probability_bps,
        validator_churn_bps=validator_churn_bps,
        max_validator_churn_bps=max_validator_churn_bps,
        quorum_safety_margin_bps=quorum_safety_margin_bps,
        min_quorum_safety_margin_bps=min_quorum_safety_margin_bps,
    )
    simulation_hash_valid = policy_checks["simulation_hash_valid"]
    stake_risk_within_limit = policy_checks["stake_risk_within_limit"]
    slash_probability_within_limit = policy_checks["slash_probability_within_limit"]
    validator_churn_within_limit = policy_checks["validator_churn_within_limit"]
    quorum_margin_within_limit = policy_checks["quorum_margin_within_limit"]

    is_go = (
        simulation_hash_valid
        and stake_risk_within_limit
        and slash_probability_within_limit
        and validator_churn_within_limit
        and quorum_margin_within_limit
        and evidence_complete
        and ci_fast_gate == "PASS"
    )
    final_decision = GO_DECISION if is_go else NO_GO_DECISION

    reason_codes = _compute_reason_codes(
        simulation_hash_valid=simulation_hash_valid,
        stake_risk_within_limit=stake_risk_within_limit,
        slash_probability_within_limit=slash_probability_within_limit,
        validator_churn_within_limit=validator_churn_within_limit,
        quorum_margin_within_limit=quorum_margin_within_limit,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "proposal_id": args.proposal_id,
        "simulation_hash": args.simulation_hash,
        "risk_metrics_bps": {
            "stake_at_risk": stake_at_risk_bps,
            "slash_probability": slash_probability_bps,
            "validator_churn": validator_churn_bps,
            "quorum_safety_margin": quorum_safety_margin_bps,
        },
        "risk_thresholds_bps": {
            "max_stake_at_risk": max_stake_at_risk_bps,
            "max_slash_probability": max_slash_probability_bps,
            "max_validator_churn": max_validator_churn_bps,
            "min_quorum_safety_margin": min_quorum_safety_margin_bps,
        },
        "evidence_complete": evidence_complete,
        "ci_fast_gate": ci_fast_gate,
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    return 0


def _require_bool(payload: Mapping[str, Any], field_name: str) -> bool:
    value = payload.get(field_name)
    if not isinstance(value, bool):
        fail(f"{field_name} must be boolean")
    return value


def check_bundle(args: argparse.Namespace) -> int:
    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "proposal_id",
            "simulation_hash",
            "risk_metrics_bps",
            "risk_thresholds_bps",
            "evidence_complete",
            "ci_fast_gate",
            "policy_checks",
            "reason_codes",
            "final_decision",
        ),
    )

    evidence_complete = _require_bool(payload, "evidence_complete")

    ci_fast_gate = payload.get("ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    metrics = payload.get("risk_metrics_bps")
    thresholds = payload.get("risk_thresholds_bps")
    if not isinstance(metrics, dict):
        fail("risk_metrics_bps must be an object")
    if not isinstance(thresholds, dict):
        fail("risk_thresholds_bps must be an object")

    metric_fields = (
        "stake_at_risk",
        "slash_probability",
        "validator_churn",
        "quorum_safety_margin",
    )
    threshold_fields = (
        "max_stake_at_risk",
        "max_slash_probability",
        "max_validator_churn",
        "min_quorum_safety_margin",
    )

    for field_name in metric_fields:
        if field_name not in metrics:
            fail(f"missing risk_metrics_bps field: {field_name}")
        if not isinstance(metrics[field_name], int):
            fail(f"risk_metrics_bps.{field_name} must be an integer")

    for field_name in threshold_fields:
        if field_name not in thresholds:
            fail(f"missing risk_thresholds_bps field: {field_name}")
        if not isinstance(thresholds[field_name], int):
            fail(f"risk_thresholds_bps.{field_name} must be an integer")

    policy_checks = payload.get("policy_checks")
    if not isinstance(policy_checks, dict):
        fail("policy_checks must be an object")

    for field_name in (
        "simulation_hash_valid",
        "stake_risk_within_limit",
        "slash_probability_within_limit",
        "validator_churn_within_limit",
        "quorum_margin_within_limit",
    ):
        if field_name not in policy_checks:
            fail(f"missing policy_checks field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    derived_policy_checks = _compute_policy_checks(
        simulation_hash=str(payload.get("simulation_hash")),
        stake_at_risk_bps=metrics["stake_at_risk"],
        max_stake_at_risk_bps=thresholds["max_stake_at_risk"],
        slash_probability_bps=metrics["slash_probability"],
        max_slash_probability_bps=thresholds["max_slash_probability"],
        validator_churn_bps=metrics["validator_churn"],
        max_validator_churn_bps=thresholds["max_validator_churn"],
        quorum_safety_margin_bps=metrics["quorum_safety_margin"],
        min_quorum_safety_margin_bps=thresholds["min_quorum_safety_margin"],
    )
    simulation_hash_valid = derived_policy_checks["simulation_hash_valid"]
    stake_risk_within_limit = derived_policy_checks["stake_risk_within_limit"]
    slash_probability_within_limit = derived_policy_checks[
        "slash_probability_within_limit"
    ]
    validator_churn_within_limit = derived_policy_checks["validator_churn_within_limit"]
    quorum_margin_within_limit = derived_policy_checks["quorum_margin_within_limit"]

    if policy_checks["simulation_hash_valid"] != simulation_hash_valid:
        fail("policy_checks.simulation_hash_valid does not match derived policy")
    if policy_checks["stake_risk_within_limit"] != stake_risk_within_limit:
        fail("policy_checks.stake_risk_within_limit does not match derived policy")
    if policy_checks["slash_probability_within_limit"] != slash_probability_within_limit:
        fail(
            "policy_checks.slash_probability_within_limit does not match derived policy"
        )
    if policy_checks["validator_churn_within_limit"] != validator_churn_within_limit:
        fail("policy_checks.validator_churn_within_limit does not match derived policy")
    if policy_checks["quorum_margin_within_limit"] != quorum_margin_within_limit:
        fail("policy_checks.quorum_margin_within_limit does not match derived policy")

    expected_go = (
        simulation_hash_valid
        and stake_risk_within_limit
        and slash_probability_within_limit
        and validator_churn_within_limit
        and quorum_margin_within_limit
        and evidence_complete
        and ci_fast_gate == "PASS"
    )
    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION

    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    failed_checks = _compute_reason_codes(
        simulation_hash_valid=simulation_hash_valid,
        stake_risk_within_limit=stake_risk_within_limit,
        slash_probability_within_limit=slash_probability_within_limit,
        validator_churn_within_limit=validator_churn_within_limit,
        quorum_margin_within_limit=quorum_margin_within_limit,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )
    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Governance stake/slash risk evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--proposal-id", required=True)
    generate.add_argument("--simulation-hash", required=True)
    generate.add_argument("--stake-at-risk-bps", required=True)
    generate.add_argument("--max-stake-at-risk-bps", required=True)
    generate.add_argument("--slash-probability-bps", required=True)
    generate.add_argument("--max-slash-probability-bps", required=True)
    generate.add_argument("--validator-churn-bps", required=True)
    generate.add_argument("--max-validator-churn-bps", required=True)
    generate.add_argument("--quorum-safety-margin-bps", required=True)
    generate.add_argument("--min-quorum-safety-margin-bps", required=True)
    generate.add_argument("--evidence-complete", required=True)
    generate.add_argument("--ci-fast-gate", required=True)
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file", required=True)
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
