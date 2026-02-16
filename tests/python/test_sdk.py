import unittest
import asyncio

from kamn_sdk import (
    KAMNClient,
    LiveKAMNClient,
    LiveTransportBackendAdapterError,
    SDKError,
    TransportMode,
    TransportModeMismatchError,
)


class PythonSDKTests(unittest.TestCase):
    def setUp(self) -> None:
        self.client = KAMNClient()

    def test_register_and_resolve_round_trip(self) -> None:
        did = self.client.register(
            agent_type="autonomous",
            model_family="claude-4",
            capabilities=["text", "code"],
        )
        resolved = self.client.resolve(did)
        self.assertEqual(resolved["id"], did)
        self.assertEqual(resolved["metadata"]["model_family"], "claude-4")

    def test_send_receive_and_drain(self) -> None:
        sender = self.client.register("autonomous", "claude-4", ["text"])
        receiver = self.client.register("assistant", "gpt-5", ["text"])
        message_id = self.client.send(sender, receiver, "hello")
        self.assertTrue(message_id.startswith("msg_"))

        first = self.client.receive(receiver)
        second = self.client.receive(receiver)
        self.assertEqual(len(first), 1)
        self.assertEqual(first[0]["body"], "hello")
        self.assertEqual(second, [])

    def test_receive_stream_orders_messages_deterministically(self) -> None:
        sender = self.client.register("autonomous", "claude-4", ["text"])
        receiver = self.client.register("assistant", "gpt-5", ["text"])
        self.client.send(sender, receiver, "first")
        self.client.send(sender, receiver, "second")

        async def collect() -> list[str]:
            bodies: list[str] = []
            async for message in self.client.receive_stream(receiver):
                bodies.append(str(message["body"]))
            return bodies

        bodies = asyncio.run(collect())
        self.assertEqual(bodies, ["first", "second"])

    def test_receive_stream_does_not_replay_consumed_messages(self) -> None:
        # Regression: #483
        sender = self.client.register("autonomous", "claude-4", ["text"])
        receiver = self.client.register("assistant", "gpt-5", ["text"])
        self.client.send(sender, receiver, "once")

        async def drain_counts() -> tuple[int, int]:
            first = [message async for message in self.client.receive_stream(receiver)]
            second = [message async for message in self.client.receive_stream(receiver)]
            return len(first), len(second)

        first_len, second_len = asyncio.run(drain_counts())
        self.assertEqual(first_len, 1)
        self.assertEqual(second_len, 0)

    def test_task_and_escrow_flow(self) -> None:
        creator = self.client.register("autonomous", "claude-4", ["research"])
        assignee = self.client.register("assistant", "gpt-5", ["research"])

        task_id = self.client.create_task(creator, "analysis", "analyze benchmark")
        self.client.accept_task(task_id, assignee)

        payer_before = self.client.balance(creator)
        payee_before = self.client.balance(assignee)
        escrow_id = self.client.create_escrow(creator, assignee, 15)
        self.client.release_escrow(escrow_id)
        self.assertEqual(self.client.balance(creator), payer_before - 15)
        self.assertEqual(self.client.balance(assignee), payee_before + 15)

    def test_search_and_reputation(self) -> None:
        did = self.client.register("autonomous", "claude-4", ["text", "code"])
        self.client.register("assistant", "gpt-5", ["text"])

        results = self.client.search_agents(capability="code")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0]["id"], did)

        reputation = self.client.get_reputation(did)
        self.assertGreater(reputation["score"], 0)

    def test_rejects_unknown_did(self) -> None:
        with self.assertRaises(SDKError):
            self.client.resolve("kamn:did:agent:unknown")

    def test_regression_register_rejects_empty_capability_entries(self) -> None:
        # Regression: #583
        with self.assertRaises(SDKError):
            self.client.register(
                agent_type="autonomous",
                model_family="claude-4",
                capabilities=["text", " "],
            )


class PythonLiveTransportSDKTests(unittest.TestCase):
    def test_unit_live_transport_config_rejects_non_live_endpoint(self) -> None:
        with self.assertRaises(SDKError):
            LiveKAMNClient("http://localhost:7000")

    def test_functional_live_transport_round_trip(self) -> None:
        client = LiveKAMNClient("https://live.kamn.testnet/python-functional")
        sender = client.register("autonomous", "claude-4", ["text"])
        receiver = client.register("assistant", "gpt-5", ["text"])

        message_id = client.send(sender, receiver, "python live hello")
        self.assertTrue(message_id.startswith("msg_"))

        received = client.receive(receiver)
        self.assertEqual(len(received), 1)
        self.assertEqual(received[0]["body"], "python live hello")

    def test_integration_live_clients_share_endpoint_state(self) -> None:
        endpoint = "https://live.kamn.testnet/python-integration"
        publisher = LiveKAMNClient(endpoint)
        consumer = LiveKAMNClient(endpoint)

        sender = publisher.register("autonomous", "claude-4", ["text"])
        receiver = publisher.register("assistant", "gpt-5", ["text"])
        publisher.send(sender, receiver, "shared endpoint python message")

        received = consumer.receive(receiver)
        self.assertEqual(len(received), 1)
        self.assertEqual(received[0]["body"], "shared endpoint python message")

    def test_regression_transport_mode_mismatch_is_rejected(self) -> None:
        # Regression: #620
        memory = KAMNClient()
        with self.assertRaises(TransportModeMismatchError) as memory_error:
            memory.assert_transport_mode(TransportMode.LIVE)
        self.assertEqual(memory_error.exception.expected, "live")
        self.assertEqual(memory_error.exception.found, "in-memory")

        live = LiveKAMNClient("https://live.kamn.testnet/python-mismatch")
        with self.assertRaises(TransportModeMismatchError) as live_error:
            live.assert_transport_mode(TransportMode.IN_MEMORY)
        self.assertEqual(live_error.exception.expected, "in-memory")
        self.assertEqual(live_error.exception.found, "live")

    def test_performance_live_transport_contract_lane_budget(self) -> None:
        client = LiveKAMNClient("https://live.kamn.testnet/python-perf")
        sender = client.register("autonomous", "claude-4", ["text"])
        receiver = client.register("assistant", "gpt-5", ["text"])

        for nonce in range(256):
            client.send(sender, receiver, f"python-live-perf-{nonce}")
        received = client.receive(receiver)
        self.assertEqual(len(received), 256)

    def test_functional_live_transport_backend_adapter_mode_normalizes_success_payloads(
        self,
    ) -> None:
        endpoint = "https://live.kamn.testnet/python-backend-adapter"
        operations: list[str] = []

        class Adapter:
            def invoke(self, request: dict[str, object]) -> dict[str, object]:
                operation = str(request["operation"])
                operations.append(operation)
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

        LiveKAMNClient.register_backend_adapter(endpoint, Adapter())
        try:
            client = LiveKAMNClient(endpoint)
            sender = client.register("autonomous", "claude-4", ["text"])
            self.assertEqual(sender, "kamn:did:agent:backend-1")

            message_id = client.send(
                "kamn:did:agent:backend-1",
                "kamn:did:agent:backend-2",
                "backend hello",
            )
            self.assertEqual(message_id, "msg_backend_1")

            received = client.receive("kamn:did:agent:backend-2")
            self.assertEqual(len(received), 1)
            self.assertEqual(received[0]["body"], "backend hello")
            self.assertEqual(operations, ["register", "send", "receive"])
        finally:
            LiveKAMNClient.clear_backend_adapters()

    def test_regression_backend_adapter_errors_and_invalid_payloads_fail_closed(
        self,
    ) -> None:
        # Regression: #1415
        endpoint = "https://live.kamn.testnet/python-backend-adapter-fail"

        class Adapter:
            def invoke(self, request: dict[str, object]) -> dict[str, object]:
                operation = str(request["operation"])
                if operation == "register":
                    return {"status": "ok", "value": 42}
                if operation == "send":
                    return {
                        "status": "error",
                        "reason_code": "backend_timeout",
                        "message": "secure signer backend timed out",
                    }
                return {"status": "error", "reason": "policy_denied"}

        LiveKAMNClient.register_backend_adapter(endpoint, Adapter())
        try:
            client = LiveKAMNClient(endpoint)
            with self.assertRaises(SDKError) as invalid_response:
                client.register("autonomous", "claude-4", ["text"])
            self.assertEqual(
                str(invalid_response.exception),
                "backend adapter invalid response for operation register: expected string value",
            )

            with self.assertRaises(LiveTransportBackendAdapterError) as timeout_error:
                client.send("kamn:did:agent:x", "kamn:did:agent:y", "hello")
            self.assertEqual(timeout_error.exception.operation, "send")
            self.assertEqual(timeout_error.exception.reason, "backend_timeout")
            self.assertEqual(timeout_error.exception.reason_code, "backend_timeout")
            self.assertEqual(
                timeout_error.exception.message, "secure signer backend timed out"
            )

            with self.assertRaises(LiveTransportBackendAdapterError) as policy_error:
                client.receive("kamn:did:agent:y")
            self.assertEqual(policy_error.exception.operation, "receive")
            self.assertEqual(policy_error.exception.reason, "policy_denied")
            self.assertEqual(policy_error.exception.reason_code, "policy_denied")
            self.assertEqual(policy_error.exception.message, "policy_denied")
        finally:
            LiveKAMNClient.clear_backend_adapters()

    def test_regression_backend_adapter_legacy_reason_normalization_is_deterministic(
        self,
    ) -> None:
        # Regression: #4436
        endpoint = "https://live.kamn.testnet/python-backend-adapter-normalization"
        reasons = [
            "Policy Denied",
            "retry-timeout",
            "###",
        ]

        class Adapter:
            def __init__(self) -> None:
                self.index = 0

            def invoke(self, request: dict[str, object]) -> dict[str, object]:
                operation = str(request["operation"])
                if operation == "receive":
                    reason = reasons[self.index]
                    self.index += 1
                    return {"status": "error", "reason": reason}
                return {"status": "ok", "value": "kamn:did:agent:backend-z"}

        LiveKAMNClient.register_backend_adapter(endpoint, Adapter())
        try:
            client = LiveKAMNClient(endpoint)

            with self.assertRaises(LiveTransportBackendAdapterError) as first:
                client.receive("kamn:did:agent:x")
            self.assertEqual(first.exception.reason, "policy_denied")
            self.assertEqual(first.exception.reason_code, "policy_denied")

            with self.assertRaises(LiveTransportBackendAdapterError) as second:
                client.receive("kamn:did:agent:x")
            self.assertEqual(second.exception.reason, "retry_timeout")
            self.assertEqual(second.exception.reason_code, "retry_timeout")

            with self.assertRaises(LiveTransportBackendAdapterError) as third:
                client.receive("kamn:did:agent:x")
            self.assertEqual(
                third.exception.reason,
                "backend_adapter_error_legacy_unknown",
            )
            self.assertEqual(
                third.exception.reason_code,
                "backend_adapter_error_legacy_unknown",
            )
        finally:
            LiveKAMNClient.clear_backend_adapters()


if __name__ == "__main__":
    unittest.main()
