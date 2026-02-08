import unittest
import asyncio

from kamn_sdk import (
    KAMNClient,
    LiveKAMNClient,
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


if __name__ == "__main__":
    unittest.main()
