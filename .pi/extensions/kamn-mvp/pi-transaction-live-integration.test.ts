import assert from "node:assert/strict";
import { resolve } from "node:path";
import test from "node:test";
import { LiveTaskWorkflow } from "./live-task-workflow.ts";
import { testSetup } from "./live-task-workflow-test-support.ts";
import { verifyPiTransactionActors, writePiTransactionActor } from "./pi-transaction-evidence.ts";

test("live three-role workflow persists and verifies v2 service authority", async () => {
	const setup = await testSetup();
	const workflow = new LiveTaskWorkflow(setup.env, process.cwd());
	await workflow.register("agent_a");
	const provider = await workflow.register("agent_b");
	await workflow.register("agent_c");
	await workflow.createTask("Authority", "Bind service receipts to Pi actors", String(provider.did));
	await workflow.fundEscrow();
	await workflow.acceptTask();
	await workflow.completeTask();
	await workflow.releaseEscrow();
	await workflow.queryParticipantProjection("agent_a");
	await workflow.queryParticipantProjection("agent_b");
	await workflow.queryVerifierProjection();

	const paths = {
		agent_a: resolve(setup.root, "agent-a.json"),
		agent_b: resolve(setup.root, "agent-b.json"),
		agent_c: resolve(setup.root, "agent-c.json"),
	};
	for (const [role, pid] of [["agent_a", 101], ["agent_b", 202], ["agent_c", 303]] as const) {
		const evidence = workflow.actorEvidence(role, pid, `sha256:${"b".repeat(64)}`);
		await writePiTransactionActor(paths[role], evidence);
	}
	const verified = await verifyPiTransactionActors(paths);
	assert.equal(verified.task_id, workflow.currentTaskId());
	assert.match(verified.receipt_chain_commitment, /^sha256:[0-9a-f]{64}$/);
	assert.deepEqual(verified.pi_process_ids, [101, 202, 303]);
	await workflow.shutdown();
});
