import test from "node:test";
import assert from "node:assert/strict";

import {
  KAMNClient,
  LiveTransportConfig,
  LiveTransportKAMNClient,
  TransportModeMismatchError,
} from "../src/index.ts";

test("unit rejects non-live endpoint config", () => {
  assert.throws(() => new LiveTransportConfig("http://localhost:7000"), {
    name: "SDKError",
    message: "transport endpoint must start with https:// or wss://",
  });
});

test("functional live transport round trip", () => {
  const client = new LiveTransportKAMNClient("https://live.kamn.testnet/ts-functional");
  const sender = client.register("autonomous", "claude-4", ["text"]);
  const receiver = client.register("assistant", "gpt-5", ["text"]);

  const messageId = client.send(sender, receiver, "ts live hello");
  assert.match(messageId, /^msg_/);

  const received = client.receive(receiver);
  assert.equal(received.length, 1);
  assert.equal(received[0].body, "ts live hello");
});

test("integration live transport clients share endpoint state", () => {
  const endpoint = "https://live.kamn.testnet/ts-integration";
  const publisher = new LiveTransportKAMNClient(endpoint);
  const consumer = new LiveTransportKAMNClient(endpoint);

  const sender = publisher.register("autonomous", "claude-4", ["text"]);
  const receiver = publisher.register("assistant", "gpt-5", ["text"]);
  publisher.send(sender, receiver, "shared endpoint typescript message");

  const received = consumer.receive(receiver);
  assert.equal(received.length, 1);
  assert.equal(received[0].body, "shared endpoint typescript message");
});

test("regression transport mode mismatch is rejected", () => {
  // Regression: #620
  const memory = new KAMNClient();
  assert.throws(() => memory.assertTransportMode("live"), TransportModeMismatchError);

  const live = new LiveTransportKAMNClient("https://live.kamn.testnet/ts-mismatch");
  assert.throws(() => live.assertTransportMode("in-memory"), TransportModeMismatchError);
});

test("performance live transport contract lane stays within budget", () => {
  const client = new LiveTransportKAMNClient("https://live.kamn.testnet/ts-perf");
  const sender = client.register("autonomous", "claude-4", ["text"]);
  const receiver = client.register("assistant", "gpt-5", ["text"]);

  const start = Date.now();
  for (let nonce = 1; nonce <= 256; nonce += 1) {
    client.send(sender, receiver, `ts-live-perf-${nonce}`);
  }
  const received = client.receive(receiver);
  assert.equal(received.length, 256);

  const elapsedMillis = Date.now() - start;
  assert.ok(
    elapsedMillis < 300,
    `typescript sdk live transport contract lane exceeded budget: ${elapsedMillis}ms`,
  );
});
