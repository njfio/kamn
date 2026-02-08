import test from "node:test";
import assert from "node:assert/strict";

import { LiveTransportKAMNClient } from "../src/index.ts";

test("performance live transport multi-client deep lane", () => {
  const endpoint = "https://live.kamn.testnet/ts-deep";
  const publisher = new LiveTransportKAMNClient(endpoint);
  const consumer = new LiveTransportKAMNClient(endpoint);

  const sender = publisher.register("autonomous", "claude-4", ["text"]);
  const receiver = publisher.register("assistant", "gpt-5", ["text"]);

  for (let nonce = 1; nonce <= 5000; nonce += 1) {
    publisher.send(sender, receiver, `ts-live-deep-${nonce}`);
  }

  const received = consumer.receive(receiver);
  assert.equal(received.length, 5000);
});
