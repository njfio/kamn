#!/usr/bin/env python3
"""DR evidence bundle generator and SLO gate policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    parse_int,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.release.dr-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_pass_fail(field_name: str, raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail(f"{field_name} must be PASS or FAIL")


def _parse_bool(field_name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"{field_name} must be true or false")


def _compute_decision_reasons(
    *,
    recovery_rto_seconds: int,
    recovery_rpo_seconds: int,
    max_rto_seconds: int,
    max_rpo_seconds: int,
    rollback_restored: bool,
    evidence_complete: bool,
    ci_fast_gate: str,
) -> list[str]:
    decision_reasons: list[str] = []
    if recovery_rto_seconds > max_rto_seconds:
        decision_reasons.append("rto threshold exceeded")
    if recovery_rpo_seconds > max_rpo_seconds:
        decision_reasons.append("rpo threshold exceeded")
    if not rollback_restored:
        decision_reasons.append("rollback not restored")
    if not evidence_complete:
        decision_reasons.append("incomplete drill evidence")
    if ci_fast_gate != "PASS":
        decision_reasons.append("ci-fast-gate-failed")
    return decision_reasons


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.drill_id,
        args.recovery_rto_seconds,
        args.recovery_rpo_seconds,
        args.max_rto_seconds,
        args.max_rpo_seconds,
        args.rollback_restored,
        args.evidence_complete,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all DR evidence bundle arguments are required")

    recovery_rto_seconds = parse_int("recovery-rto-seconds", args.recovery_rto_seconds)
    recovery_rpo_seconds = parse_int("recovery-rpo-seconds", args.recovery_rpo_seconds)
    max_rto_seconds = parse_int("max-rto-seconds", args.max_rto_seconds)
    max_rpo_seconds = parse_int("max-rpo-seconds", args.max_rpo_seconds)
    if max_rto_seconds < 1:
        fail("max-rto-seconds must be >= 1")
    if max_rpo_seconds < 1:
        fail("max-rpo-seconds must be >= 1")

    ci_fast_gate = _parse_pass_fail("ci-fast-gate", args.ci_fast_gate)
    rollback_restored = _parse_bool("rollback-restored", args.rollback_restored)
    evidence_complete = _parse_bool("evidence-complete", args.evidence_complete)

    decision_reasons = _compute_decision_reasons(
        recovery_rto_seconds=recovery_rto_seconds,
        recovery_rpo_seconds=recovery_rpo_seconds,
        max_rto_seconds=max_rto_seconds,
        max_rpo_seconds=max_rpo_seconds,
        rollback_restored=rollback_restored,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )
    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    if not decision_reasons:
        decision_reasons.append("all dr evidence gates satisfied")

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "drill_id": args.drill_id,
        "dr_evidence": {
            "recovery_rto_seconds": recovery_rto_seconds,
            "recovery_rpo_seconds": recovery_rpo_seconds,
            "max_rto_seconds": max_rto_seconds,
            "max_rpo_seconds": max_rpo_seconds,
            "rollback_restored": rollback_restored,
            "evidence_complete": evidence_complete,
            "ci_fast_gate": ci_fast_gate,
        },
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    return 0


def _require_dr_field(dr_evidence: Any, field_name: str) -> Any:
    if field_name not in dr_evidence:
        fail(f"missing dr_evidence field: {field_name}")
    return dr_evidence[field_name]


def _require_dr_int(dr_evidence: Any, field_name: str) -> int:
    value = _require_dr_field(dr_evidence, field_name)
    if not isinstance(value, int):
        fail(f"dr_evidence.{field_name} must be an integer")
    return value


def check_bundle(args: argparse.Namespace) -> int:
    if not args.bundle_file:
        fail("--bundle-file is required")

    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "drill_id",
            "dr_evidence",
            "decision_reasons",
            "final_decision",
        ),
    )

    dr_evidence = payload["dr_evidence"]
    if not isinstance(dr_evidence, dict):
        fail("bundle field 'dr_evidence' must be an object")

    recovery_rto_seconds = _require_dr_int(dr_evidence, "recovery_rto_seconds")
    recovery_rpo_seconds = _require_dr_int(dr_evidence, "recovery_rpo_seconds")
    max_rto_seconds = _require_dr_int(dr_evidence, "max_rto_seconds")
    max_rpo_seconds = _require_dr_int(dr_evidence, "max_rpo_seconds")

    if max_rto_seconds < 1:
        fail("dr_evidence.max_rto_seconds must be >= 1")
    if max_rpo_seconds < 1:
        fail("dr_evidence.max_rpo_seconds must be >= 1")

    rollback_restored = _require_dr_field(dr_evidence, "rollback_restored")
    if not isinstance(rollback_restored, bool):
        fail("dr_evidence.rollback_restored must be a boolean")

    evidence_complete = _require_dr_field(dr_evidence, "evidence_complete")
    if not isinstance(evidence_complete, bool):
        fail("dr_evidence.evidence_complete must be a boolean")

    ci_fast_gate = _require_dr_field(dr_evidence, "ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("dr_evidence.ci_fast_gate must be PASS or FAIL")

    decision_reasons = _compute_decision_reasons(
        recovery_rto_seconds=recovery_rto_seconds,
        recovery_rpo_seconds=recovery_rpo_seconds,
        max_rto_seconds=max_rto_seconds,
        max_rpo_seconds=max_rpo_seconds,
        rollback_restored=rollback_restored,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )

    expected_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        reasons = ", ".join(decision_reasons) if decision_reasons else "all dr evidence gates satisfied"
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}; reasons={reasons}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"recovery_rto_seconds={recovery_rto_seconds}")
    print(f"recovery_rpo_seconds={recovery_rpo_seconds}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="DR evidence contract utilities (generate/check)."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--drill-id")
    generate.add_argument("--recovery-rto-seconds")
    generate.add_argument("--recovery-rpo-seconds")
    generate.add_argument("--max-rto-seconds")
    generate.add_argument("--max-rpo-seconds")
    generate.add_argument("--rollback-restored")
    generate.add_argument("--evidence-complete")
    generate.add_argument("--ci-fast-gate")
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file")
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
