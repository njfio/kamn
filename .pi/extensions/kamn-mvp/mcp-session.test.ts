import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { McpSession, readLiveMcpConfig } from "./mcp-session.ts";

const fixture = resolve(".pi/extensions/kamn-mvp/test-fixtures/fake-mcp-server.mjs");

test("session starts lazily and reuses one child for ordered calls", async () => {
	const paths = await testPaths();
	const session = sessionFor(paths, "success");
	await assert.rejects(readFile(paths.startFile), { code: "ENOENT" });

	const registered = await session.call("register");
	const queried = await session.call("query_agent_profile", { did: registered.did });

	assert.equal(registered.pid, queried.pid);
	assert.equal(registered.request_id, "1");
	assert.equal(queried.request_id, "2");
	assert.equal(queried.did, registered.did);
	await session.shutdown();
	await session.shutdown();
	assert.equal((await readFile(paths.stopFile, "utf8")).trim(), String(registered.pid));
});

test("session provenance binds child process, request range, and runtime response digests", async () => {
	const paths = await testPaths();
	const session = sessionFor(paths, "success");
	const registered = await session.call("register");
	const queried = await session.call("query_agent_profile", { did: registered.did });

	assert.deepEqual(session.provenance(), {
		child_process_id: registered.pid,
		first_request_id: 1,
		last_request_id: 2,
		runtime_response_digests: [responseDigest(registered), responseDigest(queried)],
	});
	await session.shutdown();
});

test("session provenance keeps handled error responses in the contiguous request stream", async () => {
	const paths = await testPaths();
	const session = sessionFor(paths, "error-on-second");
	const registered = await session.call("register");
	await assert.rejects(session.call("release_escrow"), /forced backend failure/);
	const queried = await session.call("query_agent_profile", { did: registered.did });

	assert.deepEqual(session.provenance(), {
		child_process_id: registered.pid,
		first_request_id: 1,
		last_request_id: 3,
		runtime_response_digests: [
			responseDigest(registered),
			responseDigest({ kind: "backend_error", message: "forced backend failure" }),
			responseDigest(queried),
		],
	});
	await session.shutdown();
});

test("configuration rejects missing values and key files", () => {
	assert.throws(() => readLiveMcpConfig("AGENT_A", {}, process.cwd()), /KAMN_MVP_LIVE_MCP_BINARY/);
	assert.throws(
		() => readLiveMcpConfig("AGENT_A", configEnv("/missing/key", {}), process.cwd()),
		/key file does not exist/,
	);
});

test("configuration does not forward unrelated Pi credentials", async () => {
	const paths = await testPaths();
	const config = readLiveMcpConfig("AGENT_A", configEnv(paths.keyFile, { OPENAI_API_KEY: "must-not-leak" }), process.cwd());

	assert.equal(config.env.OPENAI_API_KEY, undefined);
	assert.equal(config.env.KAMN_MVP_LIVE_MCP_AGENT_A_NAME, "agent-a");
});

test("configuration selects independent Agent B identity inputs", async () => {
	const paths = await testPaths();
	const env = configEnv(paths.keyFile, {
		KAMN_MVP_LIVE_MCP_AGENT_B_NAME: "agent-b",
		KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE: paths.keyFile,
	});
	const config = readLiveMcpConfig("AGENT_B", env, process.cwd());

	assert.equal(config.agentName, "agent-b");
	assert.equal(config.keyFile, paths.keyFile);
});

test("configuration selects independent Agent C verifier identity inputs", async () => {
	const paths = await testPaths();
	const env = configEnv(paths.keyFile, {
		KAMN_MVP_LIVE_MCP_AGENT_C_NAME: "agent-c-verifier",
		KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE: paths.keyFile,
	});
	const config = readLiveMcpConfig("AGENT_C", env, process.cwd());

	assert.equal(config.agentName, "agent-c-verifier");
	assert.equal(config.keyFile, paths.keyFile);
});

for (const [mode, expected] of [
	["error", /backend_error.*forced backend failure/],
	["malformed", /invalid JSON/],
	["mismatch", /response ID mismatch/],
	["exit", /exited.*7/],
	["hang", /timed out/],
] as const) {
	test(`session fails loudly for ${mode} child behavior`, async () => {
		const paths = await testPaths();
		const session = sessionFor(paths, mode, mode === "hang" ? 50 : 1000);
		await assert.rejects(session.call("register"), expected);
		await session.shutdown();
	});
}

test("an already aborted call rejects without starting the child", async () => {
	const paths = await testPaths();
	const session = sessionFor(paths, "success");
	const controller = new AbortController();
	controller.abort();

	await assert.rejects(session.call("register", {}, controller.signal), /aborted/);
	await assert.rejects(readFile(paths.startFile), { code: "ENOENT" });
});

async function testPaths() {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-mcp-session-"));
	const keyFile = resolve(root, "agent.key");
	await writeFile(keyFile, "test-only-key\n");
	await chmod(fixture, 0o755);
	return { keyFile, startFile: resolve(root, "start"), stopFile: resolve(root, "stop") };
}

function sessionFor(paths: Awaited<ReturnType<typeof testPaths>>, mode: string, timeoutMs = 1000) {
	return new McpSession(
		readLiveMcpConfig("AGENT_A", configEnv(paths.keyFile, {
			KAMN_MVP_FAKE_MCP_MODE: mode,
			KAMN_MVP_FAKE_MCP_START_FILE: paths.startFile,
			KAMN_MVP_FAKE_MCP_STOP_FILE: paths.stopFile,
		}), process.cwd()),
		{ timeoutMs },
	);
}

function configEnv(keyFile: string, extra: Record<string, string>) {
	return {
		KAMN_MVP_LIVE_MCP_BINARY: fixture,
		KAMN_MVP_LIVE_MCP_ENDPOINT: "http://127.0.0.1:18278",
		KAMN_MVP_LIVE_MCP_AGENT_A_NAME: "agent-a",
		KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE: keyFile,
		...extra,
	};
}

function responseDigest(response: Record<string, unknown>): string {
	return `sha256:${createHash("sha256").update(JSON.stringify(response)).digest("hex")}`;
}
