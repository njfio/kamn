import unittest

from kamn_sdk import KAMNClient, SDKError


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


if __name__ == "__main__":
    unittest.main()
