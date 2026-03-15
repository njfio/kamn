#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import time
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import URLError
from urllib.request import Request, urlopen

SCHEMA_VERSION = "kamn.solana.devnet.live-proof-report.v1"
COMMITMENTS = ("processed", "confirmed", "finalized")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Capture live Solana devnet JSON-RPC commitment evidence."
    )
    parser.add_argument(
        "--rpc-url",
        default="https://api.devnet.solana.com",
        help="Solana JSON-RPC endpoint (default: https://api.devnet.solana.com)",
    )
    parser.add_argument("--output-json", required=True)
    parser.add_argument("--timeout-seconds", type=int, default=20)
    parser.add_argument("--max-seconds", type=int, default=30)
    return parser.parse_args()


def rpc_call(rpc_url: str, method: str, params: list[object], timeout_seconds: int) -> object:
    payload = json.dumps(
        {"jsonrpc": "2.0", "id": method, "method": method, "params": params}
    ).encode("utf-8")
    request = Request(rpc_url, data=payload, headers={"Content-Type": "application/json"})
    try:
        with urlopen(request, timeout=timeout_seconds) as response:
            body = json.loads(response.read().decode("utf-8"))
    except URLError as error:  # pragma: no cover - external boundary
        raise SystemExit(f"solana rpc request failed for {method}: {error}") from error
    if "error" in body:
        raise SystemExit(f"solana rpc error for {method}: {body['error']}")
    return body.get("result")


def build_receipt_proofs(slots: dict[str, int]) -> list[dict[str, object]]:
    proofs: list[dict[str, object]] = []
    for label in COMMITMENTS:
        slot = slots[label]
        proofs.append(
            {
                "network": "solana",
                "receipt_id": f"solana:{label}:slot:{slot}",
                "block_reference": f"solana:slot:{slot}",
                "finality_label": label,
                "confirmation_count": 0,
                "status": "success",
            }
        )
    return proofs


def main() -> int:
    args = parse_args()
    start = time.time()
    health = rpc_call(args.rpc_url, "getHealth", [], args.timeout_seconds)
    version = rpc_call(args.rpc_url, "getVersion", [], args.timeout_seconds)
    slots = {
        label: rpc_call(
            args.rpc_url,
            "getSlot",
            [{"commitment": label}],
            args.timeout_seconds,
        )
        for label in COMMITMENTS
    }
    elapsed = time.time() - start
    if elapsed > args.max_seconds:
        raise SystemExit(
            f"live Solana devnet proof exceeded runtime budget: {elapsed:.2f}s > {args.max_seconds}s"
        )
    if health != "ok":
        raise SystemExit(f"unexpected Solana getHealth result: {health!r}")
    if not isinstance(version, dict) or not version.get("solana-core"):
        raise SystemExit("solana getVersion result missing solana-core")
    if any(not isinstance(slots[label], int) for label in COMMITMENTS):
        raise SystemExit("solana getSlot results must all be integers")

    report = {
        "schema_version": SCHEMA_VERSION,
        "rpc_url": args.rpc_url,
        "observed_at_utc": datetime.now(timezone.utc).isoformat(),
        "health_status": health,
        "solana_core_version": version["solana-core"],
        "feature_set": version.get("feature-set"),
        "commitment_slots": slots,
        "slot_order_valid": slots["processed"] >= slots["confirmed"] >= slots["finalized"],
        "finality_labels": list(COMMITMENTS),
        "receipt_proofs": build_receipt_proofs(slots),
        "elapsed_seconds": round(elapsed, 3),
    }

    output = Path(args.output_json).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    print("status=ok")
    print(f"rpc_url={args.rpc_url}")
    print(f"health_status={health}")
    for label in COMMITMENTS:
        print(f"{label}_slot={slots[label]}")
    print(f"slot_order_valid={str(report['slot_order_valid']).lower()}")
    print(f"report_file={output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
