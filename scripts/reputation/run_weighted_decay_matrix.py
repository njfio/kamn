#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List, Tuple


ROOT_DIR = Path(__file__).resolve().parents[2]
TRUST_SCORE_MIN = 0
TRUST_SCORE_MAX = 1000


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fixture",
        default=str(ROOT_DIR / "fixtures/reputation_decay/compact_cases.json"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-cases",
        type=int,
        default=0,
        help="Optional cap for first N cases to evaluate.",
    )
    return parser.parse_args()


def clamp(value: int, lower: int, upper: int) -> int:
    return max(lower, min(upper, value))


def response_component(response_time_avg_ms: int) -> int:
    if response_time_avg_ms <= 1_000:
        return 100
    if response_time_avg_ms <= 5_000:
        return 50
    if response_time_avg_ms <= 30_000:
        return 0
    return -50


def calculate_decay_multiplier_bps(
    last_updated_block: int,
    score_history_blocks: List[int],
    tasks_completed: int,
    tasks_failed: int,
    tasks_delegated: int,
    dispute_rate: float,
) -> int:
    if not score_history_blocks:
        multiplier = 950
    else:
        recent = 0
        mid = 0
        stale = 0
        for block in score_history_blocks:
            age = max(0, last_updated_block - block)
            if age <= 128:
                recent += 1
            elif age <= 512:
                mid += 1
            else:
                stale += 1
        multiplier = 550 + min(recent, 3) * 120 + min(mid, 4) * 40 + min(stale, 8) * 10

    if tasks_completed + tasks_failed + tasks_delegated < 25:
        multiplier -= 50
    if dispute_rate > 0.2:
        multiplier -= 80

    return clamp(multiplier, 500, 1000)


def classify_abuse_penalty(
    tasks_completed: int,
    tasks_failed: int,
    tasks_delegated: int,
    disputes_count: int,
) -> Tuple[str, int]:
    completed = float(tasks_completed)
    delegated = float(tasks_delegated)
    failed = float(tasks_failed)
    disputes = float(disputes_count)

    delegation_ratio = delegated / max(1.0, completed)
    failure_ratio = failed / max(1.0, completed + failed)
    churn_ratio = disputes / max(1.0, completed + failed)

    reciprocity_ring = delegation_ratio >= 0.60
    burst_spam = failure_ratio >= 0.45 and tasks_failed >= 10
    churn_spike = churn_ratio >= 0.20 and disputes_count >= 5

    triggered = sum([reciprocity_ring, burst_spam, churn_spike])
    if triggered > 1:
        return ("Compound", 140)
    if reciprocity_ring:
        return ("ReciprocityRing", 80)
    if burst_spam:
        return ("BurstSpam", 70)
    if churn_spike:
        return ("ChurnSpike", 60)
    return ("None", 0)


def evaluate_case(case: Dict[str, Any]) -> Dict[str, Any]:
    delivery_rate = float(case["delivery_rate"])
    dispute_rate = float(case["dispute_rate"])
    response_time_avg_ms = int(case["response_time_avg_ms"])
    tasks_completed = int(case["tasks_completed"])
    tasks_failed = int(case["tasks_failed"])
    tasks_delegated = int(case["tasks_delegated"])
    endorsements_count = int(case["endorsements_count"])
    disputes_count = int(case["disputes_count"])
    last_updated_block = int(case["last_updated_block"])
    score_history_blocks = [int(value) for value in case.get("score_history_blocks", [])]

    base_score = 500
    delivery = int((delivery_rate - 0.5) * 400.0)
    response = response_component(response_time_avg_ms)
    dispute_penalty = int(dispute_rate * 150.0)
    volume_bonus = int(min(tasks_completed, 1000) * 0.1)
    endorsement_bonus = min(endorsements_count, 50)

    decay_multiplier_bps = calculate_decay_multiplier_bps(
        last_updated_block=last_updated_block,
        score_history_blocks=score_history_blocks,
        tasks_completed=tasks_completed,
        tasks_failed=tasks_failed,
        tasks_delegated=tasks_delegated,
        dispute_rate=dispute_rate,
    )
    decayed_volume_bonus = volume_bonus * decay_multiplier_bps // 1000
    decayed_endorsement_bonus = endorsement_bonus * decay_multiplier_bps // 1000
    abuse_penalty_kind, abuse_penalty_points = classify_abuse_penalty(
        tasks_completed=tasks_completed,
        tasks_failed=tasks_failed,
        tasks_delegated=tasks_delegated,
        disputes_count=disputes_count,
    )

    raw_score = (
        base_score
        + delivery
        + response
        - dispute_penalty
        + decayed_volume_bonus
        + decayed_endorsement_bonus
        - abuse_penalty_points
    )
    final_score = clamp(raw_score, TRUST_SCORE_MIN, TRUST_SCORE_MAX)

    return {
        "delivery_component": delivery,
        "response_component": response,
        "dispute_penalty": dispute_penalty,
        "volume_bonus": volume_bonus,
        "endorsement_bonus": endorsement_bonus,
        "decay_multiplier_bps": decay_multiplier_bps,
        "decayed_volume_bonus": decayed_volume_bonus,
        "decayed_endorsement_bonus": decayed_endorsement_bonus,
        "abuse_penalty_kind": abuse_penalty_kind,
        "abuse_penalty_points": abuse_penalty_points,
        "raw_score": raw_score,
        "final_score": final_score,
    }


def main() -> int:
    args = parse_args()
    fixture_path = Path(args.fixture)
    if not fixture_path.exists():
        print(f"status=fail; reason=fixture-not-found; fixture={fixture_path}")
        return 2

    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
    cases = fixture.get("cases")
    if not isinstance(cases, list):
        print("status=fail; reason=invalid-fixture-cases")
        return 2

    selected_cases = cases[: args.max_cases] if args.max_cases > 0 else cases
    failed_case_ids: List[str] = []
    report_cases: List[Dict[str, Any]] = []

    for index, case in enumerate(selected_cases):
        if not isinstance(case, dict):
            case_id = f"invalid-case-{index}"
            failed_case_ids.append(case_id)
            report_cases.append(
                {
                    "case_id": case_id,
                    "passed": False,
                    "error": "case payload is not an object",
                }
            )
            continue

        case_id = str(case.get("case_id", f"case-{index}"))
        try:
            actual = evaluate_case(case)
        except (TypeError, ValueError, KeyError) as exc:
            failed_case_ids.append(case_id)
            report_cases.append(
                {
                    "case_id": case_id,
                    "passed": False,
                    "error": str(exc),
                }
            )
            continue

        expected_final_score = int(case.get("expected_final_score", 0))
        expected_decay_multiplier_bps = int(case.get("expected_decay_multiplier_bps", 0))
        expected_penalty_kind = str(case.get("expected_abuse_penalty_kind", ""))
        expected_penalty_points = int(case.get("expected_abuse_penalty_points", 0))

        passed = (
            actual["final_score"] == expected_final_score
            and actual["decay_multiplier_bps"] == expected_decay_multiplier_bps
            and actual["abuse_penalty_kind"] == expected_penalty_kind
            and actual["abuse_penalty_points"] == expected_penalty_points
        )
        if not passed:
            failed_case_ids.append(case_id)

        report_cases.append(
            {
                "case_id": case_id,
                "expected_final_score": expected_final_score,
                "actual_final_score": actual["final_score"],
                "expected_decay_multiplier_bps": expected_decay_multiplier_bps,
                "actual_decay_multiplier_bps": actual["decay_multiplier_bps"],
                "expected_abuse_penalty_kind": expected_penalty_kind,
                "actual_abuse_penalty_kind": actual["abuse_penalty_kind"],
                "expected_abuse_penalty_points": expected_penalty_points,
                "actual_abuse_penalty_points": actual["abuse_penalty_points"],
                "passed": passed,
            }
        )

    status = "pass" if not failed_case_ids else "fail"
    report = {
        "schema_version": "kamn.reputation.weighted-decay.matrix.v1",
        "status": status,
        "fixture": str(fixture_path),
        "case_count": len(selected_cases),
        "failed_count": len(failed_case_ids),
        "failed_case_ids": failed_case_ids,
        "cases": report_cases,
    }

    if args.output_json:
        Path(args.output_json).write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )

    if status == "pass":
        print(f"status=pass; cases={len(selected_cases)}; failed=0")
        return 0

    print(
        "status=fail; "
        f"cases={len(selected_cases)}; failed={len(failed_case_ids)}; "
        f"failed_ids={','.join(failed_case_ids)}"
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
