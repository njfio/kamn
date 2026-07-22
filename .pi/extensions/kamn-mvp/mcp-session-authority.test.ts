import assert from "node:assert/strict";
import { chmod, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { McpSession, readLiveMcpConfig } from "./mcp-session.ts";

const fixture = resolve(".pi/extensions/kamn-mvp/test-fixtures/fake-mcp-server.mjs");

test("session separates validated service authority from transport provenance", async (context) => {
	const setup = await testSetup();
	const session = sessionFor(setup, "success");
	context.after(() => session.shutdown());
	const registered = await session.call("register");
	await session.call("create_task", { payload: taskPayload() });
	const provenance = session.provenance() as Record<string, unknown>;

	assert.equal(provenance.service_profile_commitment, digest("f"));
	assert.deepEqual(provenance.service_authority_receipts, [{
		actor_did: registered.did,
		action: "task:create",
		resource_id: "task-live-1",
		resulting_state: "submitted",
		service_receipt_id: "task-transition-receipt-1",
		service_receipt_digest: digest("1"),
		tool: "create_task",
	}]);
	assert.equal(Array.isArray(provenance.transport_response_digests), true);
	assert.equal("runtime_response_digests" in provenance, false);
	assert.equal("runtime_response_receipts" in provenance, false);
});

for (const [mode, expected] of [
	["missing-authority", "MCP_AUTHORITY_RECEIPT_MISSING"],
	["malformed-authority", "MCP_AUTHORITY_RECEIPT_INVALID"],
	["mixed-authority-version", "MCP_AUTHORITY_RECEIPT_INVALID"],
] as const) {
	test(`${mode} is fatal and cannot open a replacement nonce stream`, async (context) => {
		const setup = await testSetup();
		const session = sessionFor(setup, mode);
		context.after(() => session.shutdown());

		await assert.rejects(session.call("register"), new RegExp(expected));
		await assert.rejects(session.call("register"), new RegExp(expected));
		assert.equal((await readFile(setup.startFile, "utf8")).trim().split("\n").length, 1);
	});
}

test("session rejects copied cross-role mutation authority", async (context) => {
	const setup = await testSetup();
	const session = sessionFor(setup, "cross-role-authority");
	context.after(() => session.shutdown());
	await session.call("register");

	await assert.rejects(
		session.call("create_task", { payload: taskPayload() }),
		/MCP_AUTHORITY_RECEIPT_INVALID/,
	);
	await assert.rejects(session.call("query_agent_profile"), /MCP_AUTHORITY_RECEIPT_INVALID/);
});

async function testSetup() {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-mcp-authority-"));
	const keyFile = resolve(root, "agent.key");
	await writeFile(keyFile, "test-only-key\n");
	await chmod(fixture, 0o755);
	return { keyFile, startFile: resolve(root, "start"), stopFile: resolve(root, "stop") };
}

function sessionFor(setup: Awaited<ReturnType<typeof testSetup>>, resultMode: string) {
	const env = {
		KAMN_MVP_LIVE_MCP_BINARY: fixture,
		KAMN_MVP_LIVE_MCP_ENDPOINT: "http://127.0.0.1:18278",
		KAMN_MVP_LIVE_MCP_AGENT_A_NAME: "agent-a",
		KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE: setup.keyFile,
		KAMN_MVP_FAKE_MCP_MODE: "success",
		KAMN_MVP_FAKE_MCP_RESULT_MODE: resultMode,
		KAMN_MVP_FAKE_MCP_START_FILE: setup.startFile,
		KAMN_MVP_FAKE_MCP_STOP_FILE: setup.stopFile,
	};
	return new McpSession(readLiveMcpConfig("AGENT_A", env, process.cwd()));
}

function taskPayload() {
	return JSON.stringify({
		title: "Authority proof",
		description: "Bind service receipt",
		provider_did: "kamn:did:agent-b",
		transaction_id: "transaction-live-1",
		terms_digest: "a".repeat(64),
		idempotency_key: "transaction-live-1-create",
	});
}

function digest(character: string): string {
	return `sha256:${character.repeat(64)}`;
}
