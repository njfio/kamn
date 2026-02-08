#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[2]
if str(ROOT_DIR) not in sys.path:
    sys.path.insert(0, str(ROOT_DIR))

from kamn_sdk import (  # noqa: E402
    KAMNClient,
    LiveKAMNClient,
    SDKError,
    TransportMode,
    TransportModeMismatchError,
)


def sanitize(value: str) -> str:
    return value.replace("\n", " ")


def main() -> int:
    try:
        memory = KAMNClient()
        live = LiveKAMNClient("https://live.kamn.testnet/profile-probe-python")

        try:
            memory.assert_transport_mode(TransportMode.LIVE)
        except TransportModeMismatchError as error:
            memory_expected = error.expected
            memory_found = error.found
        else:
            print("status=error")
            print("error=memory client unexpectedly accepted live mode assertion")
            return 1

        try:
            live.assert_transport_mode(TransportMode.IN_MEMORY)
        except TransportModeMismatchError as error:
            live_expected = error.expected
            live_found = error.found
        else:
            print("status=error")
            print("error=live client unexpectedly accepted in-memory mode assertion")
            return 1

        print("status=ok")
        print(f"default_transport_mode={memory.transport_mode().value}")
        print(f"live_transport_mode={live.transport_mode().value}")
        print(f"memory_mismatch_expected={memory_expected}")
        print(f"memory_mismatch_found={memory_found}")
        print(f"live_mismatch_expected={live_expected}")
        print(f"live_mismatch_found={live_found}")
        return 0
    except SDKError as error:
        print("status=error")
        print(f"error={sanitize(str(error))}")
        return 1
    except Exception as error:  # pragma: no cover
        print("status=error")
        print(f"error={sanitize(str(error))}")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
