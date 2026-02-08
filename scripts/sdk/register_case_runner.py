#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import List

ROOT_DIR = Path(__file__).resolve().parents[2]
if str(ROOT_DIR) not in sys.path:
    sys.path.insert(0, str(ROOT_DIR))

from kamn_sdk import KAMNClient, SDKError


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent-type", required=True)
    parser.add_argument("--model-family", required=True)
    parser.add_argument("--capability", action="append", default=[])
    return parser.parse_args()


def sanitize(value: str) -> str:
    return value.replace("\n", " ")


def main() -> None:
    args = parse_args()
    capabilities: List[str] = list(args.capability)

    client = KAMNClient()
    try:
        did = client.register(args.agent_type, args.model_family, capabilities)
        print("status=ok")
        print(f"did={did}")
    except SDKError as error:
        print("status=error")
        print(f"error={sanitize(str(error))}")


if __name__ == "__main__":
    main()
