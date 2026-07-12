import assert from "node:assert/strict";
import { createHash } from "node:crypto";
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
	const created = await workflow.createTask("Review proof", "Validate the local task lifecycle", String(agentB.did));
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

test("three agents drive one completed escrow transaction through independent MCP sessions", async () => {
	const setup = await testSetup();
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	const agentA = await workflow.register("agent_a");
	const agentB = await workflow.register("agent_b");
	const agentC = await workflow.register("agent_c");
	const created = await workflow.createTask("Settle proof", "Bind three runtime views", String(agentB.did));
	const funded = await workflow.fundEscrow();
	const accepted = await workflow.acceptTask();
	const completed = await workflow.completeTask();
	const released = await workflow.releaseEscrow();
	const viewA = await workflow.queryParticipantProjection("agent_a");
	const viewB = await workflow.queryParticipantProjection("agent_b");
	const viewC = await workflow.queryVerifierProjection();

	assert.equal(new Set([agentA.did, agentB.did, agentC.did]).size, 3);
	assert.equal(new Set([agentA.pid, agentB.pid, agentC.pid]).size, 3);
	assert.equal(created.provider_did, agentB.did);
	assert.match(String(created.transaction_id), /^pi-devnet-[a-f0-9]{16}$/);
	assert.match(String(created.terms_digest), /^[a-f0-9]{64}$/);
	assert.match(String(created.idempotency_key), /-create$/);
	assert.equal(accepted.idempotency_key, `${created.transaction_id}-accept`);
	assert.equal(completed.idempotency_key, `${created.transaction_id}-complete`);
	assert.equal(completed.completion_evidence_digest, completionDigest(String(created.terms_digest)));
	assert.equal(funded.transaction_id, created.transaction_id);
	assert.equal(funded.terms_digest, created.terms_digest);
	assert.equal(funded.beneficiary_did, agentB.did);
	assert.equal(funded.amount_lamports, 1000000);
	assert.equal(released.idempotency_key, `${created.transaction_id}-release`);
	assert.equal(funded.escrow_id, released.escrow_id);
	assert.equal(viewA.task_id, created.task_id);
	assert.equal(viewB.task_id, created.task_id);
	assert.equal(viewC.task_id, created.task_id);
	assert.equal(viewA.public_commitment, viewB.public_commitment);
	assert.equal(viewB.public_commitment, viewC.public_commitment);
	assert.equal(viewA.private_receipt_digest, "sha256:participant-a");
	assert.equal(viewB.private_receipt_digest, "sha256:participant-b");
	assert.equal("private_receipt_digest" in viewC, false);
	for (const [role, pid, scope] of [
		["agent_a", 101, "participant-private"],
		["agent_b", 202, "participant-private"],
		["agent_c", 303, "restricted-public"],
	] as const) {
		const evidence = workflow.actorEvidence(role, pid, `sha256:${"b".repeat(64)}`);
		assert.equal(evidence.pi_process_id, pid);
		assert.equal(evidence.did, `kamn:did:${role.replace("agent_", "agent-")}`);
		assert.equal(evidence.view_scope, scope);
		if (role !== "agent_c") assert.equal(evidence.participant_role, role === "agent_a" ? "creator" : "provider");
		assert.equal(evidence.runtime_projection_digest, workflow.provenance(role).runtime_response_digests.at(-1));
		assert.equal(evidence.handoff_authorized, false);
	}
	for (const role of ["agent_a", "agent_b", "agent_c"] as const) {
		const provenance = workflow.provenance(role);
		assert.ok(provenance.child_process_id > 0);
		assert.ok(provenance.last_request_id >= provenance.first_request_id);
		assert.equal(provenance.runtime_response_digests.every((value) => value.startsWith("sha256:")), true);
	}
	await workflow.shutdown();
});

test("ambiguous release reconciles through the same MCP child and idempotency key", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_MODE: "ambiguous-first-release" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	const agentB = await workflow.register("agent_b");
	await workflow.createTask("Settle proof", "Reconcile one signature", String(agentB.did));
	await workflow.fundEscrow();
	const released = await workflow.releaseEscrow();
	const provenance = workflow.provenance("agent_a");

	assert.equal(released.state, "released");
	assert.deepEqual(provenance.runtime_response_receipts.slice(-2).map((receipt) => receipt.status), ["error", "success"]);
	assert.deepEqual(provenance.runtime_response_receipts.slice(-2).map((receipt) => receipt.tool), ["release_escrow", "release_escrow"]);
	assert.equal(provenance.runtime_response_receipts.at(-2)?.response_digest.startsWith("sha256:"), true);
	assert.equal(provenance.last_request_id, 4);
	await workflow.shutdown();
});

test("actor evidence requires a registered identity and final runtime projection", async () => {
	const setup = await testSetup();
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	assert.throws(() => workflow.actorEvidence("agent_c", 303, `sha256:${"b".repeat(64)}`), /Register Agent C/);
	await workflow.register("agent_c");
	workflow.importTask("task-live-1");
	assert.throws(() => workflow.actorEvidence("agent_c", 303, `sha256:${"b".repeat(64)}`), /projection/);
	await workflow.shutdown();
});

test("workflow rejects calls that violate registration and task order", async () => {
	const setup = await testSetup();
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());

	await assert.rejects(workflow.createTask("title", "description", "kamn:did:provider"), /Register Agent A/);
	await assert.rejects(workflow.acceptTask(), /Register Agent B/);
	await assert.rejects(workflow.queryTask("agent_a"), /Register Agent A/);
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
	["missing-task-id", /omitted task_id/],
	["wrong-create-state", /expected submitted/],
] as const) {
	test(`workflow rejects ${mode} creation result`, async () => {
		const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: mode });
		const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
		await workflow.register("agent_a");

		await assert.rejects(workflow.createTask("title", "description", "kamn:did:provider"), expected);
		await workflow.shutdown();
	});
}

function completionDigest(termsDigest: string): string {
	return createHash("sha256").update(`completed:${termsDigest}`).digest("hex");
}

test("workflow rejects a mismatched acceptance task ID", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "wrong-accept-id" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	await workflow.register("agent_b");
	await workflow.createTask("title", "description", "kamn:did:provider");

	await assert.rejects(workflow.acceptTask(), /different task ID/);
	await workflow.shutdown();
});

test("workflow rejects a non-accepted query projection", async () => {
	const setup = await testSetup({ KAMN_MVP_FAKE_MCP_RESULT_MODE: "wrong-query-state" });
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	await workflow.register("agent_b");
	await workflow.createTask("title", "description", "kamn:did:provider");
	await workflow.acceptTask();

	await assert.rejects(workflow.queryTask("agent_a"), /expected accepted/);
	await workflow.shutdown();
});

test("Agent B imports an external task while Agent A polls accepted state", async () => {
	const setupA = await testSetup();
	const setupB = await testSetup();
	const agentA = new LiveTaskWorkflow(setupA.env, process.cwd());
	const agentB = new LiveTaskWorkflow(setupB.env, process.cwd());
	await agentA.register("agent_a");
	await agentB.register("agent_b");
	const created = await agentA.createTask("title", "description", "kamn:did:agent-b");
	agentB.importTask(String(created.task_id), {
		transaction_id: String(created.transaction_id),
		terms_digest: String(created.terms_digest),
		provider_did: String(created.provider_did),
	});

	const accepted = await agentB.acceptTask();
	assert.equal(accepted.idempotency_key, `${created.transaction_id}-accept`);
	await agentB.queryTask("agent_b");
	const observed = await agentA.waitForAccepted("agent_a", { timeoutMs: 100, pollMs: 5 });
	assert.deepEqual(agentA.acceptedObservation("agent_a"), observed);
	assert.equal(agentB.acceptedObservation("agent_b").task_id, created.task_id);
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

async function testSetup(extra: Record<string, string> = {}) {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-live-task-"));
	const agentAKey = resolve(root, "agent-a.key");
	const agentBKey = resolve(root, "agent-b.key");
	const agentCKey = resolve(root, "agent-c.key");
	const stopFile = resolve(root, "stop");
	await writeFile(agentAKey, "test-agent-a-key\n");
	await writeFile(agentBKey, "test-agent-b-key\n");
	await writeFile(agentCKey, "test-agent-c-key\n");
	await chmod(fixture, 0o755);
	return {
		stopFile,
			env: {
				KAMN_MVP_PI_RUN_ID: "test-run",
				KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS: "1000000",
			KAMN_MVP_LIVE_MCP_BINARY: fixture,
			KAMN_MVP_LIVE_MCP_ENDPOINT: "http://127.0.0.1:18278",
			KAMN_MVP_LIVE_MCP_AGENT_A_NAME: "agent-a",
			KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE: agentAKey,
			KAMN_MVP_LIVE_MCP_AGENT_B_NAME: "agent-b",
			KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE: agentBKey,
			KAMN_MVP_LIVE_MCP_AGENT_C_NAME: "agent-c",
			KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE: agentCKey,
			KAMN_MVP_FAKE_MCP_STOP_FILE: stopFile,
			...extra,
		},
	};
}
