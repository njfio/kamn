import assert from "node:assert/strict";
import { chmod, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { LiveTaskWorkflow } from "./live-task-workflow.ts";

const fixture = resolve(".pi/extensions/kamn-mvp/test-fixtures/fake-mcp-server.mjs");

test("two agents register and share one accepted task through independent sessions", async () => {
	const setup = await testSetup();
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());

	const agentA = await workflow.register("agent_a");
	const agentB = await workflow.register("agent_b");
	const created = await workflow.createTask("Review proof", "Validate the local task lifecycle");
	const accepted = await workflow.acceptTask();
	const queryA = await workflow.queryTask("agent_a");
	const queryB = await workflow.queryTask("agent_b");

	assert.notEqual(agentA.did, agentB.did);
	assert.notEqual(agentA.pid, agentB.pid);
	assert.deepEqual([agentA.request_id, created.request_id, queryA.request_id], ["1", "2", "3"]);
	assert.deepEqual([agentB.request_id, accepted.request_id, queryB.request_id], ["1", "2", "3"]);
	assert.equal(created.title, "Review proof");
	assert.equal(created.description, "Validate the local task lifecycle");
	for (const result of [accepted, queryA, queryB]) {
		assert.equal(result.task_id, created.task_id);
		assert.equal(result.state, "accepted");
	}

	await workflow.shutdown();
	await workflow.shutdown();
	assert.equal((await readFile(setup.stopFile, "utf8")).trim().split("\n").length, 2);
});

test("workflow rejects calls that violate registration and task order", async () => {
	const setup = await testSetup();
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());

	await assert.rejects(workflow.createTask("title", "description"), /Register Agent A/);
	await assert.rejects(workflow.acceptTask(), /Register Agent B/);
	await assert.rejects(workflow.queryTask("agent_a"), /Register Agent A/);
	await workflow.register("agent_a");
	await assert.rejects(workflow.createTask(" ", "description"), /title/);
	await assert.rejects(workflow.queryTask("agent_a"), /Create a task/);
	await workflow.shutdown();
});

async function testSetup() {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-live-task-"));
	const agentAKey = resolve(root, "agent-a.key");
	const agentBKey = resolve(root, "agent-b.key");
	const stopFile = resolve(root, "stop");
	await writeFile(agentAKey, "test-agent-a-key\n");
	await writeFile(agentBKey, "test-agent-b-key\n");
	await chmod(fixture, 0o755);
	return {
		stopFile,
		env: {
			KAMN_MVP_LIVE_MCP_BINARY: fixture,
			KAMN_MVP_LIVE_MCP_ENDPOINT: "http://127.0.0.1:18278",
			KAMN_MVP_LIVE_MCP_AGENT_A_NAME: "agent-a",
			KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE: agentAKey,
			KAMN_MVP_LIVE_MCP_AGENT_B_NAME: "agent-b",
			KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE: agentBKey,
			KAMN_MVP_FAKE_MCP_STOP_FILE: stopFile,
		},
	};
}
