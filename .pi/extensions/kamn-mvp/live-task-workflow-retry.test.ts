import assert from "node:assert/strict";
import test from "node:test";
import { LiveTaskWorkflow } from "./live-task-workflow.ts";
import { testSetup } from "./live-task-workflow-test-support.ts";

test("ambiguous release survives multiple observations on the same MCP child", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_MODE: "ambiguous-three-releases" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	const agentB = await workflow.register("agent_b");
	await workflow.createTask("Settle proof", "Reconcile one signature", String(agentB.did));
	await workflow.fundEscrow();
	const released = await workflow.releaseEscrow();
	const provenance = workflow.provenance("agent_a");
	assert.equal(released.state, "release-authorized");
	assert.deepEqual(provenance.transport_response_receipts.slice(-4).map((receipt) => receipt.outcome), ["error", "error", "error", "success"]);
	assert.equal(provenance.transport_response_receipts.slice(-4).every((receipt) => receipt.tool === "release_escrow"), true);
	assert.equal(provenance.last_request_id, 7);
	await workflow.shutdown();
});

test("participant projection waits through interim views", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "pending-three-projections" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	const agentB = await workflow.register("agent_b");
	await workflow.createTask("Settle proof", "Wait for finalized view", String(agentB.did));
	const projection = await workflow.queryParticipantProjection("agent_a");
	assert.equal(projection.settlement_tx_signature, "devnet-signature-1");
	assert.equal(workflow.provenance("agent_a").transport_response_receipts.filter(
		(receipt) => receipt.tool === "query_participant_task_projection",
	).length, 4);
	await workflow.shutdown();
});

test("Agent B waits for a runtime escrow binding", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "missing-first-escrow" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_b");
	workflow.importTask("task-live-1", {
		transaction_id: "transaction-live-1", terms_digest: "a".repeat(64), provider_did: "kamn:did:agent-b",
	});
	const projection = await workflow.waitForEscrowFunding();
	assert.equal(projection.escrow_id, "escrow-live-1");
	assert.equal(workflow.provenance("agent_b").last_request_id, 3);
	await workflow.shutdown();
});

test("Agent A waits for completed task state", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "completed-second-query" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	workflow.importTask("task-live-1");
	const task = await workflow.waitForCompleted("agent_a", { timeoutMs: 100, pollMs: 5 });
	assert.equal(task.state, "completed");
	assert.equal(workflow.provenance("agent_a").last_request_id, 3);
	await workflow.shutdown();
});

test("authenticated calls retain and back off after a typed rate limit", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_MODE: "rate-limit-on-second" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	const created = await workflow.createTask("Rate proof", "Respect server backoff", "kamn:did:provider");
	const receipts = workflow.provenance("agent_a").transport_response_receipts;
	assert.equal(created.task_id, "task-live-1");
	assert.deepEqual(receipts.slice(-2).map((receipt) => receipt.outcome), ["error", "success"]);
	await workflow.shutdown();
});

test("actor evidence rejects missing or mismatched projections", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "projection-authority-mismatch" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	assert.throws(() => workflow.actorEvidence("agent_c", 303, `sha256:${"b".repeat(64)}`), /Register Agent C/);
	await workflow.register("agent_a");
	const agentB = await workflow.register("agent_b");
	await workflow.createTask("Authority", "Reject copied projection receipt", String(agentB.did));
	await workflow.fundEscrow();
	await workflow.releaseEscrow();
	await workflow.queryParticipantProjection("agent_a");
	assert.throws(() => workflow.actorEvidence("agent_a", 101, `sha256:${"b".repeat(64)}`), /PI_SERVICE_AUTHORITY_MISMATCH/);
	await workflow.shutdown();
});
