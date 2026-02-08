import test from "node:test";
import assert from "node:assert/strict";

import { KAMNClient, OpenClawConnector, SDKError } from "../src/index.ts";

test("connector registers OpenClaw agent with required capability", () => {
  const client = new KAMNClient();
  const connector = new OpenClawConnector(client);

  const openClawDid = connector.registerOpenClawAgent("gpt-5");
  const resolved = client.resolve(openClawDid);

  assert.ok(resolved.metadata.capabilities.includes("openclaw"));
  assert.ok(resolved.metadata.capabilities.includes("code"));
});

test("reference workflow executes request to settlement", () => {
  const client = new KAMNClient();
  const connector = new OpenClawConnector(client);

  const requesterDid = client.register("autonomous", "claude-4", ["text", "payments"]);
  const openClawDid = connector.registerOpenClawAgent("gpt-5");

  const requesterBefore = client.balance(requesterDid);
  const openClawBefore = client.balance(openClawDid);

  const result = connector.runReferenceWorkflow({
    requesterDid,
    openClawDid,
    prompt: "Analyze benchmark variance and summarize remediation steps",
    compensation: 20,
  });

  assert.match(result.messageId, /^msg_/);
  assert.match(result.taskId, /^task_/);
  assert.match(result.escrowId, /^escrow_/);
  assert.equal(result.workflowStatus, "settled");

  assert.equal(client.balance(requesterDid), requesterBefore - 20);
  assert.equal(client.balance(openClawDid), openClawBefore + 20);
});

test("integration rejects agent without openclaw capability", () => {
  const client = new KAMNClient();
  const connector = new OpenClawConnector(client);

  const requesterDid = client.register("autonomous", "claude-4", ["text", "payments"]);
  const genericDid = client.register("assistant", "gpt-5", ["text", "code"]);

  assert.throws(
    () =>
      connector.runReferenceWorkflow({
        requesterDid,
        openClawDid: genericDid,
        prompt: "hello",
        compensation: 5,
      }),
    SDKError,
  );
});

test("regression rejects empty prompt", () => {
  // Regression: #190
  const client = new KAMNClient();
  const connector = new OpenClawConnector(client);

  const requesterDid = client.register("autonomous", "claude-4", ["text", "payments"]);
  const openClawDid = connector.registerOpenClawAgent("gpt-5");

  assert.throws(
    () =>
      connector.runReferenceWorkflow({
        requesterDid,
        openClawDid,
        prompt: "   ",
        compensation: 5,
      }),
    SDKError,
  );
});
