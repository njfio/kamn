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
    LiveTransportBackendAdapterError,
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

        success_endpoint = "https://live.kamn.testnet/profile-probe-python-adapter"

        class SuccessAdapter:
            def invoke(self, request: dict[str, object]) -> dict[str, object]:
                operation = str(request.get("operation", ""))
                if operation == "register":
                    return {"status": "ok", "value": "kamn:did:agent:backend-1"}
                if operation == "send":
                    return {"status": "ok", "value": "msg_backend_1"}
                if operation == "receive":
                    return {
                        "status": "ok",
                        "value": [
                            {
                                "id": "msg_backend_1",
                                "from": "kamn:did:agent:backend-1",
                                "to": "kamn:did:agent:backend-2",
                                "body": "backend hello",
                            }
                        ],
                    }
                return {"status": "ok", "value": None}

        LiveKAMNClient.register_backend_adapter(success_endpoint, SuccessAdapter())
        try:
            adapter_client = LiveKAMNClient(success_endpoint)
            backend_register_id = adapter_client.register("autonomous", "claude-4", ["text"])
            backend_message_id = adapter_client.send(
                "kamn:did:agent:backend-1",
                "kamn:did:agent:backend-2",
                "backend hello",
            )
            backend_messages = adapter_client.receive("kamn:did:agent:backend-2")
            if len(backend_messages) != 1:
                print("status=error")
                print("error=backend adapter receive returned unexpected message count")
                return 1
            backend_receive_body = str(backend_messages[0].get("body", ""))
        finally:
            LiveKAMNClient.clear_backend_adapters()

        failure_endpoint = "https://live.kamn.testnet/profile-probe-python-adapter-fail"

        class FailureAdapter:
            def invoke(self, request: dict[str, object]) -> dict[str, object]:
                operation = str(request.get("operation", ""))
                if operation == "register":
                    return {"status": "ok", "value": 7}
                if operation == "send":
                    return {"status": "error", "reason": "backend_timeout"}
                return {"status": "error", "reason": "policy_denied"}

        LiveKAMNClient.register_backend_adapter(failure_endpoint, FailureAdapter())
        try:
            failing_client = LiveKAMNClient(failure_endpoint)
            try:
                failing_client.register("autonomous", "claude-4", ["text"])
                print("status=error")
                print("error=failing adapter unexpectedly accepted invalid register payload")
                return 1
            except SDKError as error:
                backend_invalid_response_message = str(error)

            try:
                failing_client.send("kamn:did:agent:x", "kamn:did:agent:y", "hello")
                print("status=error")
                print("error=failing adapter unexpectedly accepted send operation")
                return 1
            except LiveTransportBackendAdapterError as error:
                backend_error_operation = error.operation
                backend_error_reason = error.reason

            try:
                failing_client.receive("kamn:did:agent:y")
                print("status=error")
                print("error=failing adapter unexpectedly accepted receive operation")
                return 1
            except LiveTransportBackendAdapterError as error:
                backend_policy_reason = error.reason
        finally:
            LiveKAMNClient.clear_backend_adapters()

        print("status=ok")
        print(f"default_transport_mode={memory.transport_mode().value}")
        print(f"live_transport_mode={live.transport_mode().value}")
        print(f"memory_mismatch_expected={memory_expected}")
        print(f"memory_mismatch_found={memory_found}")
        print(f"live_mismatch_expected={live_expected}")
        print(f"live_mismatch_found={live_found}")
        print(f"backend_adapter_register_id={backend_register_id}")
        print(f"backend_adapter_message_id={backend_message_id}")
        print(f"backend_adapter_receive_body={backend_receive_body}")
        print(
            "backend_adapter_invalid_response_message="
            f"{sanitize(backend_invalid_response_message)}"
        )
        print(f"backend_adapter_error_operation={backend_error_operation}")
        print(f"backend_adapter_error_reason={backend_error_reason}")
        print(f"backend_adapter_policy_reason={backend_policy_reason}")
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
