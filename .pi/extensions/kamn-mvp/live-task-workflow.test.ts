import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { LiveTaskWorkflow } from "./live-task-workflow.ts";
import { completionDigest, projectedReceipt, testSetup } from "./live-task-workflow-test-support.ts";

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
	assert.equal(accepted.idempotency_key, `${created.transaction_id}-accept`);
	assert.equal(completed.idempotency_key, `${created.transaction_id}-complete`);
	assert.equal(completed.completion_evidence_digest, completionDigest(String(created.terms_digest)));
	assert.equal(funded.transaction_id, created.transaction_id);
	assert.equal(funded.beneficiary_did, agentB.did);
	assert.equal(released.idempotency_key, `${created.transaction_id}-release`);
	assert.equal(funded.escrow_id, released.escrow_id);
	assert.equal(viewA.public_commitment, viewB.public_commitment);
	assert.equal(viewB.public_commitment, viewC.public_commitment);
	assert.deepEqual(viewA.receipt_chain_receipts, workflow.provenance("agent_a").service_authority_receipts.map(projectedReceipt));
	assert.deepEqual(viewB.receipt_chain_receipts, workflow.provenance("agent_b").service_authority_receipts.map(projectedReceipt));
	assert.equal("receipt_chain_receipts" in viewC, false);
	for (const [role, pid, scope] of actorCases()) {
		const evidence = workflow.actorEvidence(role, pid, `sha256:${"b".repeat(64)}`);
		assert.equal(evidence.pi_process_id, pid);
		assert.equal(evidence.view_scope, scope);
		assert.deepEqual(evidence.service_receipts, workflow.provenance(role).service_authority_receipts);
		assert.equal(evidence.receipt_chain_commitment, viewC.receipt_chain_commitment);
	}
	for (const role of ["agent_a", "agent_b", "agent_c"] as const) {
		const provenance = workflow.provenance(role);
		assert.ok(provenance.child_process_id > 0);
		assert.equal(provenance.transport_response_digests.every((value) => value.startsWith("sha256:")), true);
		assert.match(provenance.service_profile_commitment, /^sha256:[0-9a-f]{64}$/);
	}
	await workflow.shutdown();
});

function actorCases() {
	return [
		["agent_a", 101, "participant-private"],
		["agent_b", 202, "participant-private"],
		["agent_c", 303, "restricted-public"],
	] as const;
}
