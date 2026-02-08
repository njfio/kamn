import test from "node:test";
import assert from "node:assert/strict";

import { KAMNClient, SDKError } from "../src/index.ts";

test("register and resolve round trip", () => {
  const client = new KAMNClient();
  const did = client.register("autonomous", "claude-4", ["text", "code"]);

  const resolved = client.resolve(did);
  assert.equal(resolved.id, did);
  assert.equal(resolved.metadata.modelFamily, "claude-4");
});

test("send receive and drain", () => {
  const client = new KAMNClient();
  const sender = client.register("autonomous", "claude-4", ["text"]);
  const receiver = client.register("assistant", "gpt-5", ["text"]);

  const messageId = client.send(sender, receiver, "hello");
  assert.match(messageId, /^msg_/);

  const first = client.receive(receiver);
  const second = client.receive(receiver);

  assert.equal(first.length, 1);
  assert.equal(first[0].body, "hello");
  assert.equal(second.length, 0);
});

test("receive stream iterator orders messages deterministically", async () => {
  const client = new KAMNClient();
  const sender = client.register("autonomous", "claude-4", ["text"]);
  const receiver = client.register("assistant", "gpt-5", ["text"]);

  client.send(sender, receiver, "first");
  client.send(sender, receiver, "second");

  const bodies: string[] = [];
  for await (const message of client.receiveStream(receiver)) {
    bodies.push(message.body);
  }

  assert.deepEqual(bodies, ["first", "second"]);
});

test("regression receive stream does not replay consumed messages", async () => {
  // Regression: #485
  const client = new KAMNClient();
  const sender = client.register("autonomous", "claude-4", ["text"]);
  const receiver = client.register("assistant", "gpt-5", ["text"]);
  client.send(sender, receiver, "once");

  const first: string[] = [];
  for await (const message of client.receiveStream(receiver)) {
    first.push(message.body);
  }

  const second: string[] = [];
  for await (const message of client.receiveStream(receiver)) {
    second.push(message.body);
  }

  assert.equal(first.length, 1);
  assert.equal(second.length, 0);
});

test("task and escrow flow", () => {
  const client = new KAMNClient();
  const creator = client.register("autonomous", "claude-4", ["research"]);
  const assignee = client.register("assistant", "gpt-5", ["research"]);

  const taskId = client.createTask(creator, "analysis", "analyze benchmark");
  client.acceptTask(taskId, assignee);

  const payerBefore = client.balance(creator);
  const payeeBefore = client.balance(assignee);

  const escrowId = client.createEscrow(creator, assignee, 15);
  client.releaseEscrow(escrowId);

  assert.equal(client.balance(creator), payerBefore - 15);
  assert.equal(client.balance(assignee), payeeBefore + 15);
});

test("search and reputation", () => {
  const client = new KAMNClient();
  const did = client.register("autonomous", "claude-4", ["text", "code"]);
  client.register("assistant", "gpt-5", ["text"]);

  const results = client.searchAgents({ capability: "code" });
  assert.equal(results.length, 1);
  assert.equal(results[0].id, did);

  const reputation = client.getReputation(did);
  assert.ok(reputation.score > 0);
});

test("regression rejects duplicate escrow release", () => {
  // Regression: #218
  const client = new KAMNClient();
  const payer = client.register("autonomous", "claude-4", ["payments"]);
  const payee = client.register("assistant", "gpt-5", ["payments"]);
  const escrowId = client.createEscrow(payer, payee, 10);

  client.releaseEscrow(escrowId);

  assert.throws(() => client.releaseEscrow(escrowId), SDKError);
});

test("regression register rejects empty capability entries", () => {
  // Regression: #583
  const client = new KAMNClient();
  assert.throws(
    () => client.register("autonomous", "claude-4", ["text", " "]),
    SDKError,
  );
});
