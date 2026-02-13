#!/usr/bin/env python3
"""Staging rehearsal evidence generator and policy checker."""

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
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.release.staging-rehearsal.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
MAX_BPS = 10_000


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


def _parse_non_negative_int(field_name: str, raw_value: str) -> int:
    try:
        parsed = int(raw_value)
    except (TypeError, ValueError):
        fail(f"{field_name} must be an integer")
    if parsed < 0:
        fail(f"{field_name} must be >= 0")
    return parsed


def _parse_positive_int(field_name: str, raw_value: str) -> int:
    parsed = _parse_non_negative_int(field_name, raw_value)
    if parsed == 0:
        fail(f"{field_name} must be > 0")
    return parsed


def _parse_bps(field_name: str, raw_value: str) -> int:
    parsed = _parse_non_negative_int(field_name, raw_value)
    if parsed > MAX_BPS:
        fail(f"{field_name} must be <= {MAX_BPS}")
    return parsed


def _compute_decision_reasons(
    *,
    deploy_status: str,
    rollback_status: str,
    rollback_hash_match: bool,
    mttr_within_bound: bool,
    runtime_submit_success_rate_within_bound: bool,
    runtime_finality_timeouts_within_bound: bool,
    signer_profile_drift_within_bound: bool,
    evidence_complete: bool,
    ci_fast_gate: str,
) -> list[str]:
    decision_reasons: list[str] = []
    if deploy_status != "PASS":
        decision_reasons.append("deploy-failed")
    if rollback_status != "PASS":
        decision_reasons.append("rollback-failed")
    if not rollback_hash_match:
        decision_reasons.append("rollback target hash mismatch")
    if not mttr_within_bound:
        decision_reasons.append("mttr-threshold-exceeded")
    if not runtime_submit_success_rate_within_bound:
        decision_reasons.append("runtime_submit_success_rate_below_threshold")
    if not runtime_finality_timeouts_within_bound:
        decision_reasons.append("runtime_finality_timeout_threshold_exceeded")
    if not signer_profile_drift_within_bound:
        decision_reasons.append("signer_profile_drift_threshold_exceeded")
    if not evidence_complete:
        decision_reasons.append("incomplete evidence")
    if ci_fast_gate != "PASS":
        decision_reasons.append("ci-fast-gate-failed")
    return decision_reasons


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.release_candidate,
        args.deploy_status,
        args.rollback_status,
        args.rollback_target_hash,
        args.post_rollback_hash,
        args.recovery_time_seconds,
        args.max_allowed_recovery_time_seconds,
        args.evidence_complete,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all rehearsal bundle arguments are required")

    deploy_status = _parse_pass_fail("deploy-status", args.deploy_status)
    rollback_status = _parse_pass_fail("rollback-status", args.rollback_status)
    ci_fast_gate = _parse_pass_fail("ci-fast-gate", args.ci_fast_gate)
    recovery_time_seconds = _parse_non_negative_int(
        "recovery-time-seconds", args.recovery_time_seconds
    )
    max_allowed_recovery_time_seconds = _parse_positive_int(
        "max-allowed-recovery-time-seconds", args.max_allowed_recovery_time_seconds
    )
    runtime_submit_success_rate_bps = _parse_bps(
        "runtime-submit-success-rate-bps", args.runtime_submit_success_rate_bps
    )
    min_runtime_submit_success_rate_bps = _parse_bps(
        "min-runtime-submit-success-rate-bps", args.min_runtime_submit_success_rate_bps
    )
    runtime_finality_timeout_count = _parse_non_negative_int(
        "runtime-finality-timeout-count", args.runtime_finality_timeout_count
    )
    max_runtime_finality_timeout_count = _parse_non_negative_int(
        "max-runtime-finality-timeout-count", args.max_runtime_finality_timeout_count
    )
    signer_profile_drift_events = _parse_non_negative_int(
        "signer-profile-drift-events", args.signer_profile_drift_events
    )
    max_signer_profile_drift_events = _parse_non_negative_int(
        "max-signer-profile-drift-events", args.max_signer_profile_drift_events
    )
    evidence_complete = _parse_bool("evidence-complete", args.evidence_complete)
    rollback_hash_match = args.rollback_target_hash == args.post_rollback_hash
    mttr_within_bound = recovery_time_seconds <= max_allowed_recovery_time_seconds
    runtime_submit_success_rate_within_bound = (
        runtime_submit_success_rate_bps >= min_runtime_submit_success_rate_bps
    )
    runtime_finality_timeouts_within_bound = (
        runtime_finality_timeout_count <= max_runtime_finality_timeout_count
    )
    signer_profile_drift_within_bound = (
        signer_profile_drift_events <= max_signer_profile_drift_events
    )

    decision_reasons = _compute_decision_reasons(
        deploy_status=deploy_status,
        rollback_status=rollback_status,
        rollback_hash_match=rollback_hash_match,
        mttr_within_bound=mttr_within_bound,
        runtime_submit_success_rate_within_bound=runtime_submit_success_rate_within_bound,
        runtime_finality_timeouts_within_bound=runtime_finality_timeouts_within_bound,
        signer_profile_drift_within_bound=signer_profile_drift_within_bound,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )

    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    if not decision_reasons:
        decision_reasons.append("all rehearsal gates satisfied")

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "release_candidate": args.release_candidate,
        "rehearsal": {
            "deploy_status": deploy_status,
            "rollback_status": rollback_status,
            "rollback_target_hash": args.rollback_target_hash,
            "post_rollback_hash": args.post_rollback_hash,
            "rollback_hash_match": rollback_hash_match,
            "recovery_time_seconds": recovery_time_seconds,
            "max_allowed_recovery_time_seconds": max_allowed_recovery_time_seconds,
            "mttr_within_bound": mttr_within_bound,
            "runtime_submit_success_rate_bps": runtime_submit_success_rate_bps,
            "min_runtime_submit_success_rate_bps": min_runtime_submit_success_rate_bps,
            "runtime_submit_success_rate_within_bound": runtime_submit_success_rate_within_bound,
            "runtime_finality_timeout_count": runtime_finality_timeout_count,
            "max_runtime_finality_timeout_count": max_runtime_finality_timeout_count,
            "runtime_finality_timeouts_within_bound": runtime_finality_timeouts_within_bound,
            "signer_profile_drift_events": signer_profile_drift_events,
            "max_signer_profile_drift_events": max_signer_profile_drift_events,
            "signer_profile_drift_within_bound": signer_profile_drift_within_bound,
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


def _require_rehearsal_field(rehearsal: Any, field_name: str) -> Any:
    if field_name not in rehearsal:
        fail(f"missing rehearsal field: {field_name}")
    return rehearsal[field_name]


def _require_rehearsal_string(rehearsal: Any, field_name: str) -> str:
    value = _require_rehearsal_field(rehearsal, field_name)
    if not isinstance(value, str):
        fail(f"rehearsal.{field_name} must be a string")
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
            "release_candidate",
            "rehearsal",
            "decision_reasons",
            "final_decision",
        ),
    )

    rehearsal = payload["rehearsal"]
    if not isinstance(rehearsal, dict):
        fail("bundle field 'rehearsal' must be an object")

    deploy_status = _require_rehearsal_field(rehearsal, "deploy_status")
    rollback_status = _require_rehearsal_field(rehearsal, "rollback_status")
    ci_fast_gate = _require_rehearsal_field(rehearsal, "ci_fast_gate")
    if deploy_status not in {"PASS", "FAIL"}:
        fail("rehearsal.deploy_status must be PASS or FAIL")
    if rollback_status not in {"PASS", "FAIL"}:
        fail("rehearsal.rollback_status must be PASS or FAIL")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("rehearsal.ci_fast_gate must be PASS or FAIL")

    rollback_hash_match = _require_rehearsal_field(rehearsal, "rollback_hash_match")
    if not isinstance(rollback_hash_match, bool):
        fail("rehearsal.rollback_hash_match must be a boolean")

    recovery_time_seconds = _require_rehearsal_field(rehearsal, "recovery_time_seconds")
    if not isinstance(recovery_time_seconds, int) or isinstance(recovery_time_seconds, bool):
        fail("rehearsal.recovery_time_seconds must be an integer")
    if recovery_time_seconds < 0:
        fail("rehearsal.recovery_time_seconds must be >= 0")

    max_allowed_recovery_time_seconds = _require_rehearsal_field(
        rehearsal, "max_allowed_recovery_time_seconds"
    )
    if not isinstance(max_allowed_recovery_time_seconds, int) or isinstance(
        max_allowed_recovery_time_seconds, bool
    ):
        fail("rehearsal.max_allowed_recovery_time_seconds must be an integer")
    if max_allowed_recovery_time_seconds <= 0:
        fail("rehearsal.max_allowed_recovery_time_seconds must be > 0")

    mttr_within_bound = _require_rehearsal_field(rehearsal, "mttr_within_bound")
    if not isinstance(mttr_within_bound, bool):
        fail("rehearsal.mttr_within_bound must be a boolean")

    runtime_submit_success_rate_bps = _require_rehearsal_field(
        rehearsal, "runtime_submit_success_rate_bps"
    )
    if not isinstance(runtime_submit_success_rate_bps, int) or isinstance(
        runtime_submit_success_rate_bps, bool
    ):
        fail("rehearsal.runtime_submit_success_rate_bps must be an integer")
    if runtime_submit_success_rate_bps < 0 or runtime_submit_success_rate_bps > MAX_BPS:
        fail(f"rehearsal.runtime_submit_success_rate_bps must be within [0,{MAX_BPS}]")

    min_runtime_submit_success_rate_bps = _require_rehearsal_field(
        rehearsal, "min_runtime_submit_success_rate_bps"
    )
    if not isinstance(min_runtime_submit_success_rate_bps, int) or isinstance(
        min_runtime_submit_success_rate_bps, bool
    ):
        fail("rehearsal.min_runtime_submit_success_rate_bps must be an integer")
    if (
        min_runtime_submit_success_rate_bps < 0
        or min_runtime_submit_success_rate_bps > MAX_BPS
    ):
        fail(
            f"rehearsal.min_runtime_submit_success_rate_bps must be within [0,{MAX_BPS}]"
        )

    runtime_submit_success_rate_within_bound = _require_rehearsal_field(
        rehearsal, "runtime_submit_success_rate_within_bound"
    )
    if not isinstance(runtime_submit_success_rate_within_bound, bool):
        fail("rehearsal.runtime_submit_success_rate_within_bound must be a boolean")

    runtime_finality_timeout_count = _require_rehearsal_field(
        rehearsal, "runtime_finality_timeout_count"
    )
    if not isinstance(runtime_finality_timeout_count, int) or isinstance(
        runtime_finality_timeout_count, bool
    ):
        fail("rehearsal.runtime_finality_timeout_count must be an integer")
    if runtime_finality_timeout_count < 0:
        fail("rehearsal.runtime_finality_timeout_count must be >= 0")

    max_runtime_finality_timeout_count = _require_rehearsal_field(
        rehearsal, "max_runtime_finality_timeout_count"
    )
    if not isinstance(max_runtime_finality_timeout_count, int) or isinstance(
        max_runtime_finality_timeout_count, bool
    ):
        fail("rehearsal.max_runtime_finality_timeout_count must be an integer")
    if max_runtime_finality_timeout_count < 0:
        fail("rehearsal.max_runtime_finality_timeout_count must be >= 0")

    runtime_finality_timeouts_within_bound = _require_rehearsal_field(
        rehearsal, "runtime_finality_timeouts_within_bound"
    )
    if not isinstance(runtime_finality_timeouts_within_bound, bool):
        fail("rehearsal.runtime_finality_timeouts_within_bound must be a boolean")

    signer_profile_drift_events = _require_rehearsal_field(
        rehearsal, "signer_profile_drift_events"
    )
    if not isinstance(signer_profile_drift_events, int) or isinstance(
        signer_profile_drift_events, bool
    ):
        fail("rehearsal.signer_profile_drift_events must be an integer")
    if signer_profile_drift_events < 0:
        fail("rehearsal.signer_profile_drift_events must be >= 0")

    max_signer_profile_drift_events = _require_rehearsal_field(
        rehearsal, "max_signer_profile_drift_events"
    )
    if not isinstance(max_signer_profile_drift_events, int) or isinstance(
        max_signer_profile_drift_events, bool
    ):
        fail("rehearsal.max_signer_profile_drift_events must be an integer")
    if max_signer_profile_drift_events < 0:
        fail("rehearsal.max_signer_profile_drift_events must be >= 0")

    signer_profile_drift_within_bound = _require_rehearsal_field(
        rehearsal, "signer_profile_drift_within_bound"
    )
    if not isinstance(signer_profile_drift_within_bound, bool):
        fail("rehearsal.signer_profile_drift_within_bound must be a boolean")

    evidence_complete = _require_rehearsal_field(rehearsal, "evidence_complete")
    if not isinstance(evidence_complete, bool):
        fail("rehearsal.evidence_complete must be a boolean")

    rollback_target_hash = _require_rehearsal_string(rehearsal, "rollback_target_hash")
    post_rollback_hash = _require_rehearsal_string(rehearsal, "post_rollback_hash")

    derived_hash_match = rollback_target_hash == post_rollback_hash
    if rollback_hash_match != derived_hash_match:
        fail(
            "rollback target hash mismatch: "
            f"declared rollback_hash_match={rollback_hash_match} "
            f"but hashes compare as {derived_hash_match}"
        )

    derived_mttr_within_bound = recovery_time_seconds <= max_allowed_recovery_time_seconds
    if mttr_within_bound != derived_mttr_within_bound:
        fail(
            "mttr bound mismatch: "
            f"declared mttr_within_bound={mttr_within_bound} "
            f"but recovery_time_seconds={recovery_time_seconds} "
            f"and max_allowed_recovery_time_seconds={max_allowed_recovery_time_seconds} "
            f"compare as {derived_mttr_within_bound}"
        )

    derived_runtime_submit_success_rate_within_bound = (
        runtime_submit_success_rate_bps >= min_runtime_submit_success_rate_bps
    )
    if (
        runtime_submit_success_rate_within_bound
        != derived_runtime_submit_success_rate_within_bound
    ):
        fail(
            "runtime submit success-rate threshold mismatch: "
            f"declared runtime_submit_success_rate_within_bound={runtime_submit_success_rate_within_bound} "
            f"but runtime_submit_success_rate_bps={runtime_submit_success_rate_bps} "
            f"and min_runtime_submit_success_rate_bps={min_runtime_submit_success_rate_bps} "
            f"compare as {derived_runtime_submit_success_rate_within_bound}"
        )

    derived_runtime_finality_timeouts_within_bound = (
        runtime_finality_timeout_count <= max_runtime_finality_timeout_count
    )
    if (
        runtime_finality_timeouts_within_bound
        != derived_runtime_finality_timeouts_within_bound
    ):
        fail(
            "runtime finality timeout threshold mismatch: "
            f"declared runtime_finality_timeouts_within_bound={runtime_finality_timeouts_within_bound} "
            f"but runtime_finality_timeout_count={runtime_finality_timeout_count} "
            f"and max_runtime_finality_timeout_count={max_runtime_finality_timeout_count} "
            f"compare as {derived_runtime_finality_timeouts_within_bound}"
        )

    derived_signer_profile_drift_within_bound = (
        signer_profile_drift_events <= max_signer_profile_drift_events
    )
    if signer_profile_drift_within_bound != derived_signer_profile_drift_within_bound:
        fail(
            "signer profile drift threshold mismatch: "
            f"declared signer_profile_drift_within_bound={signer_profile_drift_within_bound} "
            f"but signer_profile_drift_events={signer_profile_drift_events} "
            f"and max_signer_profile_drift_events={max_signer_profile_drift_events} "
            f"compare as {derived_signer_profile_drift_within_bound}"
        )

    decision_reasons = _compute_decision_reasons(
        deploy_status=deploy_status,
        rollback_status=rollback_status,
        rollback_hash_match=rollback_hash_match,
        mttr_within_bound=mttr_within_bound,
        runtime_submit_success_rate_within_bound=runtime_submit_success_rate_within_bound,
        runtime_finality_timeouts_within_bound=runtime_finality_timeouts_within_bound,
        signer_profile_drift_within_bound=signer_profile_drift_within_bound,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )
    expected_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    expected_decision_reasons = (
        decision_reasons if decision_reasons else ["all rehearsal gates satisfied"]
    )

    actual_decision_reasons = payload.get("decision_reasons")
    if not isinstance(actual_decision_reasons, list) or not all(
        isinstance(reason, str) for reason in actual_decision_reasons
    ):
        fail("decision_reasons must be a list of strings")
    if actual_decision_reasons != expected_decision_reasons:
        fail(
            "decision_reasons mismatch: "
            f"expected {expected_decision_reasons}, found {actual_decision_reasons}"
        )

    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        reasons = ", ".join(decision_reasons) if decision_reasons else "all rehearsal gates satisfied"
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}; reasons={reasons}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"rollback_hash_match={str(rollback_hash_match).lower()}")
    print(f"recovery_time_seconds={recovery_time_seconds}")
    print(f"max_allowed_recovery_time_seconds={max_allowed_recovery_time_seconds}")
    print(f"mttr_within_bound={str(mttr_within_bound).lower()}")
    print(f"runtime_submit_success_rate_bps={runtime_submit_success_rate_bps}")
    print(
        f"min_runtime_submit_success_rate_bps={min_runtime_submit_success_rate_bps}"
    )
    print(
        "runtime_submit_success_rate_within_bound="
        f"{str(runtime_submit_success_rate_within_bound).lower()}"
    )
    print(f"runtime_finality_timeout_count={runtime_finality_timeout_count}")
    print(
        f"max_runtime_finality_timeout_count={max_runtime_finality_timeout_count}"
    )
    print(
        "runtime_finality_timeouts_within_bound="
        f"{str(runtime_finality_timeouts_within_bound).lower()}"
    )
    print(f"signer_profile_drift_events={signer_profile_drift_events}")
    print(
        f"max_signer_profile_drift_events={max_signer_profile_drift_events}"
    )
    print(
        "signer_profile_drift_within_bound="
        f"{str(signer_profile_drift_within_bound).lower()}"
    )
    print(f"reason_codes={','.join(expected_decision_reasons)}")
    print(f"evidence_complete={str(evidence_complete).lower()}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Staging rehearsal evidence contract utilities (generate/check)."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--release-candidate")
    generate.add_argument("--deploy-status")
    generate.add_argument("--rollback-status")
    generate.add_argument("--rollback-target-hash")
    generate.add_argument("--post-rollback-hash")
    generate.add_argument("--recovery-time-seconds")
    generate.add_argument("--max-allowed-recovery-time-seconds")
    generate.add_argument("--evidence-complete")
    generate.add_argument("--ci-fast-gate")
    generate.add_argument("--runtime-submit-success-rate-bps", default="10000")
    generate.add_argument("--min-runtime-submit-success-rate-bps", default="9900")
    generate.add_argument("--runtime-finality-timeout-count", default="0")
    generate.add_argument("--max-runtime-finality-timeout-count", default="1")
    generate.add_argument("--signer-profile-drift-events", default="0")
    generate.add_argument("--max-signer-profile-drift-events", default="0")
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
