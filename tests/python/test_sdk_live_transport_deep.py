import unittest

from kamn_sdk import LiveKAMNClient


class PythonLiveTransportDeepTests(unittest.TestCase):
    def test_performance_live_transport_multi_client_deep_lane(self) -> None:
        endpoint = "https://live.kamn.testnet/python-deep"
        publisher = LiveKAMNClient(endpoint)
        consumer = LiveKAMNClient(endpoint)

        sender = publisher.register("autonomous", "claude-4", ["text"])
        receiver = publisher.register("assistant", "gpt-5", ["text"])

        for nonce in range(1, 5001):
            publisher.send(sender, receiver, f"python-live-deep-{nonce}")

        received = consumer.receive(receiver)
        self.assertEqual(len(received), 5000)


if __name__ == "__main__":
    unittest.main()
