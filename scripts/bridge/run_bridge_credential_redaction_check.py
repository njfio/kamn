#!/usr/bin/env python3
"""Validate bridge credential redaction contracts for staged lanes."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from time import perf_counter


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate redacted bridge credential report and enforce no-secret-leak policy."
    )
    parser.add_argument(
        "--output-json",
        type=Path,
        required=True,
        help="Path to write redaction report JSON.",
    )
    parser.add_argument(
        "--mode",
        choices=("contract", "deep"),
        required=True,
        help="Validation lane mode.",
    )
    parser.add_argument(
        "--telegram-token",
        required=True,
        help="Telegram connector token fixture.",
    )
    parser.add_argument(
        "--discord-token",
        required=True,
        help="Discord connector token fixture.",
    )
    parser.add_argument(
        "--cross-chain-token",
        required=True,
        help="Cross-chain provider token fixture.",
    )
    return parser.parse_args()


def redact(secret: str) -> str:
    if len(secret) <= 6:
        return "<redacted>"
    return f"{secret[:4]}...{secret[-2:]}"


def main() -> int:
    args = parse_args()
    started = perf_counter()

    tokens = {
        "telegram": args.telegram_token,
        "discord": args.discord_token,
        "cross_chain": args.cross_chain_token,
    }

    for connector, secret in tokens.items():
        if not secret.strip():
            print(f"status=fail")
            print(f"reason=empty-token:{connector}")
            return 1

    connectors = [
        {
            "connector": connector,
            "credential_ref": f"env:KAMN_BRIDGE_{connector.upper()}_TOKEN",
            "redacted_credential": redact(secret),
            "raw_length": len(secret),
        }
        for connector, secret in tokens.items()
    ]

    report: dict[str, object] = {
        "status": "pass",
        "mode": args.mode,
        "generated_at": datetime.now(tz=timezone.utc).isoformat(),
        "connectors": connectors,
        "policy_contract": {
            "raw_credential_exposure": "blocked",
            "regression_guard": "Regression: #621",
        },
    }

    if args.mode == "deep":
        report["deep_lane_contract"] = {
            "sample_count": 128,
            "full_bridge_suite": [
                "bridge_adapter",
                "telegram_bridge",
                "discord_bridge",
                "cross_chain_bridge",
            ],
        }

    serialized_report = json.dumps(report, sort_keys=True)
    leaked = [
        connector
        for connector, secret in tokens.items()
        if secret and secret in serialized_report
    ]
    if leaked:
        report["status"] = "fail"
        report["leaked_connectors"] = leaked

    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")

    elapsed_ms = int((perf_counter() - started) * 1000)
    if report["status"] != "pass":
        print("status=fail")
        print("reason=credential-leak-detected")
        print(f"leaked_connectors={','.join(leaked)}")
        return 1

    print("status=pass")
    print(f"mode={args.mode}")
    print(f"output_json={args.output_json}")
    print(f"elapsed_ms={elapsed_ms}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
