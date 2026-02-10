import test from "node:test";
import assert from "node:assert/strict";

import {
  KAMNClient,
  LiveTransportBackendAdapterError,
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

test("functional live transport backend adapter mode normalizes success payloads", () => {
  const endpoint = "https://live.kamn.testnet/ts-backend-adapter";
  const requests: string[] = [];

  LiveTransportKAMNClient.registerBackendAdapter(endpoint, {
    invoke(request) {
      requests.push(request.operation);
      if (request.operation === "register") {
        return { status: "ok", value: "kamn:did:agent:backend-1" };
      }
      if (request.operation === "send") {
        return { status: "ok", value: "msg_backend_1" };
      }
      if (request.operation === "receive") {
        return {
          status: "ok",
          value: [
            {
              id: "msg_backend_1",
              from: "kamn:did:agent:backend-1",
              to: "kamn:did:agent:backend-2",
              body: "backend hello",
            },
          ],
        };
      }
      return { status: "ok", value: null };
    },
  });

  try {
    const client = new LiveTransportKAMNClient(endpoint);
    const sender = client.register("autonomous", "claude-4", ["text"]);
    assert.equal(sender, "kamn:did:agent:backend-1");

    const messageId = client.send(
      "kamn:did:agent:backend-1",
      "kamn:did:agent:backend-2",
      "backend hello",
    );
    assert.equal(messageId, "msg_backend_1");

    const received = client.receive("kamn:did:agent:backend-2");
    assert.equal(received.length, 1);
    assert.equal(received[0].body, "backend hello");
    assert.deepEqual(requests, ["register", "send", "receive"]);
  } finally {
    LiveTransportKAMNClient.clearBackendAdapters();
  }
});

test("regression backend adapter errors and invalid payloads fail closed", () => {
  // Regression: #1414
  const endpoint = "https://live.kamn.testnet/ts-backend-adapter-fail";

  LiveTransportKAMNClient.registerBackendAdapter(endpoint, {
    invoke(request) {
      if (request.operation === "register") {
        return { status: "ok", value: 99 };
      }
      return { status: "error", reason: "backend_timeout" };
    },
  });

  try {
    const client = new LiveTransportKAMNClient(endpoint);
    assert.throws(() => client.register("autonomous", "claude-4", ["text"]), {
      name: "SDKError",
      message: "backend adapter invalid response for operation register: expected string value",
    });

    let sendError: unknown;
    try {
      client.send("kamn:did:agent:x", "kamn:did:agent:y", "hello");
      assert.fail("expected backend adapter send failure");
    } catch (error: unknown) {
      sendError = error;
    }
    assert.ok(sendError instanceof LiveTransportBackendAdapterError);
    assert.equal(sendError.operation, "send");
    assert.equal(sendError.reason, "backend_timeout");
  } finally {
    LiveTransportKAMNClient.clearBackendAdapters();
  }
});
