import assert from "node:assert/strict";
import test from "node:test";
import { LiveTaskWorkflow } from "./live-task-workflow.ts";
import { testSetup } from "./live-task-workflow-test-support.ts";

test("workflow rejects calls that violate registration and task order", async () => {
	const setup = await testSetup();
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await assert.rejects(workflow.createTask("title", "description", "kamn:did:provider"), /Register Agent A/);
	await assert.rejects(workflow.acceptTask(), /Register Agent B/);
	await workflow.register("agent_a");
	await assert.rejects(workflow.createTask(" ", "description", "kamn:did:provider"), /title/);
	await assert.rejects(workflow.queryTask("agent_a"), /Create a task/);
	await workflow.shutdown();
});

test("workflow rejects duplicate participant DIDs", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "same-did" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	await assert.rejects(workflow.register("agent_b"), /must be distinct/);
	await workflow.shutdown();
});

for (const [mode, expected] of [
	["missing-task-id", /MCP_AUTHORITY_RECEIPT_MISSING/],
	["wrong-create-state", /MCP_AUTHORITY_RECEIPT_INVALID/],
] as const) {
	test(`workflow rejects ${mode} creation result`, async () => {
		const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: mode });
		const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
		await workflow.register("agent_a");
		await assert.rejects(workflow.createTask("title", "description", "kamn:did:provider"), expected);
		await workflow.shutdown();
	});
}

test("workflow rejects mismatched acceptance and task projections", async () => {
	const mismatch = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "wrong-accept-id" });
	const workflow = new LiveTaskWorkflow(mismatch.env, process.cwd());
	await workflow.register("agent_a");
	await workflow.register("agent_b");
	await workflow.createTask("title", "description", "kamn:did:provider");
	await assert.rejects(workflow.acceptTask(), /different task ID/);
	await workflow.shutdown();

	const wrongState = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "wrong-query-state" });
	const queried = new LiveTaskWorkflow(wrongState.env, process.cwd());
	await queried.register("agent_a");
	await queried.register("agent_b");
	await queried.createTask("title", "description", "kamn:did:provider");
	await queried.acceptTask();
	await assert.rejects(queried.queryTask("agent_a"), /expected accepted/);
	await queried.shutdown();
});

test("independent workflows hand off one external task", async () => {
	const setupA = await testSetup();
	const setupB = await testSetup();
	const agentA = new LiveTaskWorkflow(setupA.env, process.cwd());
	const agentB = new LiveTaskWorkflow(setupB.env, process.cwd());
	await agentA.register("agent_a");
	await agentB.register("agent_b");
	const created = await agentA.createTask("title", "description", "kamn:did:agent-b");
	agentB.importTask(String(created.task_id), {
		transaction_id: String(created.transaction_id), terms_digest: String(created.terms_digest), provider_did: String(created.provider_did),
	});
	const accepted = await agentB.acceptTask();
	assert.equal(accepted.idempotency_key, `${created.transaction_id}-accept`);
	await agentB.queryTask("agent_b");
	const observed = await agentA.waitForAccepted("agent_a", { timeoutMs: 100, pollMs: 5 });
	assert.deepEqual(agentA.acceptedObservation("agent_a"), observed);
	await agentA.shutdown();
	await agentB.shutdown();
});

test("external task import and acceptance polling fail closed", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "wrong-query-state" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	assert.throws(() => workflow.importTask("bad task id"), /invalid/);
	workflow.importTask("task-live-1");
	assert.throws(() => workflow.importTask("task-other"), /conflicts/);
	await workflow.register("agent_a");
	await assert.rejects(workflow.waitForAccepted("agent_a", { timeoutMs: 20, pollMs: 5 }), /timed out/);
	const controller = new AbortController();
	controller.abort();
	await assert.rejects(workflow.waitForAccepted("agent_a", { timeoutMs: 20, pollMs: 5 }, controller.signal), /aborted/);
	await workflow.shutdown();
});
