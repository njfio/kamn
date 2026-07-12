import assert from "node:assert/strict";
import { mkdtemp, readFile, utimes, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import {
	verifyIndependentActorReceipts,
	coordinationConfig,
	waitForTaskHandoff,
	writeActorReceipt,
	writeTaskHandoff,
} from "./live-task-coordination.ts";

test("handoff is minimal, valid, and idempotent", async () => {
	const paths = await pathsForTest();
	await writeTaskHandoff(paths.handoff, handoff());
	await writeTaskHandoff(paths.handoff, handoff());

	assert.deepEqual(await waitForTaskHandoff(paths.handoff, options()), handoff());
	assert.deepEqual(Object.keys(JSON.parse(await readFile(paths.handoff, "utf8"))).sort(), [
		"artifact_digest",
		"provider_did",
		"schema_version",
		"task_id",
		"terms_digest",
		"transaction_id",
	]);
	await assert.rejects(writeTaskHandoff(paths.handoff, handoff("task-other")), /conflicts/);
});

test("handoff rejects tampering, stale state, timeout, abort, and secret-like paths", async () => {
	const paths = await pathsForTest();
	await writeTaskHandoff(paths.handoff, handoff());
	const tampered = (await readFile(paths.handoff, "utf8")).replace("task-live-1", "task-tampered");
	await writeFile(paths.handoff, tampered);
	await assert.rejects(waitForTaskHandoff(paths.handoff, options()), /digest mismatch/);

	await writeTaskHandoff(paths.stale, handoff("task-stale"));
	await utimes(paths.stale, new Date(0), new Date(0));
	await assert.rejects(waitForTaskHandoff(paths.stale, options({ maxAgeMs: 1 })), /stale/);
	await assert.rejects(waitForTaskHandoff(paths.missing, options({ timeoutMs: 20 })), /timed out/);
	const controller = new AbortController();
	controller.abort();
	await assert.rejects(waitForTaskHandoff(paths.missing, options(), controller.signal), /aborted/);
	await assert.rejects(writeTaskHandoff("/tmp/kamn-keypair-handoff.json", handoff("task-1")), /secret-like/);
});

test("coordination rejects missing config and unknown artifact fields", async () => {
	assert.throws(() => coordinationConfig({}, process.cwd()), /KAMN_MVP_LIVE_TASK_HANDOFF_FILE/);
	const paths = await pathsForTest();
	await writeTaskHandoff(paths.handoff, handoff());
	const artifact = JSON.parse(await readFile(paths.handoff, "utf8"));
	artifact.unexpected = "field";
	await writeFile(paths.handoff, JSON.stringify(artifact));

	await assert.rejects(waitForTaskHandoff(paths.handoff, options()), /field mismatch/);
});

test("actor receipts verify agreement and distinct Pi processes", async () => {
	const paths = await pathsForTest();
	await writeTaskHandoff(paths.handoff, handoff());
	await writeActorReceipt(paths.agentA, "agent_a", "task-live-1", "accepted", 101);
	await writeActorReceipt(paths.agentB, "agent_b", "task-live-1", "accepted", 202);

	const result = await verifyIndependentActorReceipts(paths.handoff, paths.agentA, paths.agentB);
	assert.deepEqual(result, {
		claim_boundary: "real local-only independent Pi actors",
		task_id: "task-live-1",
		state: "accepted",
		agent_a_pi_process_id: 101,
		agent_b_pi_process_id: 202,
	});
	assert.deepEqual(Object.keys(JSON.parse(await readFile(paths.agentA, "utf8"))).sort(), [
		"actor",
		"artifact_digest",
		"pi_process_id",
		"schema_version",
		"state",
		"task_id",
	]);
	await writeActorReceipt(paths.agentA, "agent_a", "task-live-1", "accepted", 101);
	await assert.rejects(writeActorReceipt(paths.agentA, "agent_a", "task-live-1", "accepted", 303), /conflicts/);
	const tampered = (await readFile(paths.agentB, "utf8")).replace("task-live-1", "task-tampered");
	await writeFile(paths.agentB, tampered);
	await assert.rejects(verifyIndependentActorReceipts(paths.handoff, paths.agentA, paths.agentB), /digest mismatch/);
});

test("receipt verifier rejects same process and task mismatch", async () => {
	const paths = await pathsForTest();
	await writeTaskHandoff(paths.handoff, handoff());
	await writeActorReceipt(paths.agentA, "agent_a", "task-live-1", "accepted", 101);
	await writeActorReceipt(paths.agentB, "agent_b", "task-live-1", "accepted", 101);
	await assert.rejects(verifyIndependentActorReceipts(paths.handoff, paths.agentA, paths.agentB), /distinct Pi processes/);

	const other = await pathsForTest();
	await writeTaskHandoff(other.handoff, handoff());
	await writeActorReceipt(other.agentA, "agent_a", "task-live-1", "accepted", 101);
	await writeActorReceipt(other.agentB, "agent_b", "task-other", "accepted", 202);
	await assert.rejects(verifyIndependentActorReceipts(other.handoff, other.agentA, other.agentB), /task ID mismatch/);
});

function options(overrides: Partial<{ timeoutMs: number; pollMs: number; maxAgeMs: number }> = {}) {
	return { timeoutMs: 100, pollMs: 5, maxAgeMs: 60000, ...overrides };
}

function handoff(taskId = "task-live-1") {
	return {
		task_id: taskId,
		transaction_id: "pi-devnet-1234567890abcdef",
		terms_digest: "a".repeat(64),
		provider_did: "kamn:did:agent-b",
	};
}

async function pathsForTest() {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-task-coordination-"));
	return {
		handoff: resolve(root, "handoff.json"),
		stale: resolve(root, "stale.json"),
		missing: resolve(root, "missing.json"),
		agentA: resolve(root, "agent-a.json"),
		agentB: resolve(root, "agent-b.json"),
	};
}
