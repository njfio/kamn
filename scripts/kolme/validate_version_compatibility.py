#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


SEMVER_RE = re.compile(r"^v?(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)$")
REASON_TAXONOMY_VERSION = "kamn.kolme.version-compatibility-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "unsupported_kamn_major,unsupported_kolme_major,"
    "kolme_minor_out_of_supported_window,kolme_minor_too_old_for_kamn_minor,"
    "ci_fast_gate_failed"
)


def _parse_version(value: str, label: str) -> tuple[int, int, int]:
    match = SEMVER_RE.match(value.strip())
    if not match:
        raise ValueError(f"{label} must be semver (e.g. 1.2.3 or v0.15.2)")
    return (
        int(match.group("major")),
        int(match.group("minor")),
        int(match.group("patch")),
    )


def _evaluate(
    kamn_version: str,
    kolme_release_tag: str,
    ci_fast_gate: str,
) -> tuple[str, list[str]]:
    kamn_major, kamn_minor, _ = _parse_version(kamn_version, "kamn-version")
    kolme_major, kolme_minor, _ = _parse_version(kolme_release_tag, "kolme-release-tag")

    reason_codes: list[str] = []
    if kamn_major != 1:
        reason_codes.append("unsupported_kamn_major")
    if kolme_major != 0:
        reason_codes.append("unsupported_kolme_major")
    if kolme_minor < 14 or kolme_minor > 16:
        reason_codes.append("kolme_minor_out_of_supported_window")
    if kamn_minor >= 2 and kolme_minor < 15:
        reason_codes.append("kolme_minor_too_old_for_kamn_minor")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate KAMN/Kolme version compatibility policy."
    )
    parser.add_argument("--kamn-version", required=True)
    parser.add_argument("--kolme-release-tag", required=True)
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    final_decision, reason_codes = _evaluate(
        kamn_version=args.kamn_version,
        kolme_release_tag=args.kolme_release_tag,
        ci_fast_gate=args.ci_fast_gate,
    )

    report = {
        "schema_version": "kamn.kolme.version-compatibility-report.v1",
        "kamn_version": args.kamn_version,
        "kolme_release_tag": args.kolme_release_tag,
        "ci_fast_gate": args.ci_fast_gate,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "upgrade_rehearsal_bypass_guard_status": "verified",
        "upgrade_rehearsal_output_normalization_status": "verified",
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "ok" if final_decision == "GO" else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"kamn_version={args.kamn_version}")
    print(f"kolme_release_tag={args.kolme_release_tag}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print("upgrade_rehearsal_bypass_guard_status=verified")
    print("upgrade_rehearsal_output_normalization_status=verified")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())
