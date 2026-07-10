import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { writeActorReceipt, writeTaskHandoff } from "./live-task-coordination.ts";
import {
	writeRestrictedTaskObservation,
	verifyRestrictedTaskObservation,
} from "./restricted-task-observation.ts";

test("Agent C writes a minimal task-bound restricted observation", async () => {
	const paths = await acceptedSources(101, 202);
	const first = await writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.observation, 303);
	const second = await writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.observation, 303);

	assert.deepEqual(second, first);
	assert.deepEqual(Object.keys(JSON.parse(await readFile(paths.observation, "utf8"))).sort(), [
		"agent_c_pi_process_id",
		"artifact_digest",
		"private_field_count",
		"private_payload_redacted",
		"public_commitment",
		"schema_version",
		"source_agent_a_receipt_digest",
		"source_agent_b_receipt_digest",
		"source_handoff_digest",
		"state",
		"task_id",
		"view_scope",
	]);
	assert.deepEqual(await verifyRestrictedTaskObservation(paths.observation), first);
	assert.equal(first.claim_boundary, "real local-only independent Agent C artifact observation");
	assert.equal(first.task_id, "task-live-c");
	assert.equal(first.state, "accepted");
	assert.equal(first.view_scope, "restricted-public");
	assert.equal(first.private_field_count, 0);
	assert.equal(first.private_payload_redacted, true);
});

test("Agent C observation binds all source digests and rejects conflicts", async () => {
	const paths = await acceptedSources(101, 202);
	await assert.rejects(
		writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.handoff, 303),
		/must differ from source artifacts/,
	);
	const result = await writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.observation, 303);
	const artifact = JSON.parse(await readFile(paths.observation, "utf8"));

	assert.equal(result.public_commitment, publicCommitment(artifact));
	await assert.rejects(
		writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.observation, 404),
		/conflicts/,
	);
	artifact.task_id = "task-altered";
	await writeFile(paths.observation, JSON.stringify(artifact));
	await assert.rejects(verifyRestrictedTaskObservation(paths.observation), /digest mismatch/);
});

test("Agent C observation rejects private fields and unrestricted scope", async () => {
	const paths = await acceptedSources(101, 202);
	await writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.observation, 303);
	const privateArtifact = JSON.parse(await readFile(paths.observation, "utf8"));
	privateArtifact.participant_private_view_digest = "forbidden";
	privateArtifact.artifact_digest = digestOf(privateArtifact);
	await writeFile(paths.observation, JSON.stringify(privateArtifact));
	await assert.rejects(verifyRestrictedTaskObservation(paths.observation), /field mismatch/);

	const other = await acceptedSources(101, 202);
	await writeRestrictedTaskObservation(other.handoff, other.agentA, other.agentB, other.observation, 303);
	const scoped = JSON.parse(await readFile(other.observation, "utf8"));
	scoped.view_scope = "participant-private";
	scoped.private_field_count = 1;
	scoped.private_payload_redacted = false;
	scoped.artifact_digest = digestOf(scoped);
	await writeFile(other.observation, JSON.stringify(scoped));
	await assert.rejects(verifyRestrictedTaskObservation(other.observation), /restricted-public/);
});

test("Agent C must be a third distinct Pi process", async () => {
	for (const agentCPid of [101, 202]) {
		const paths = await acceptedSources(101, 202);
		await assert.rejects(
			writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.observation, agentCPid),
			/third distinct Pi process/,
		);
	}
	const paths = await acceptedSources(101, 101);
	await assert.rejects(
		writeRestrictedTaskObservation(paths.handoff, paths.agentA, paths.agentB, paths.observation, 303),
		/distinct Pi processes/,
	);
});

test("Agent C rejects source task disagreement and non-accepted state", async () => {
	const mismatched = await acceptedSources(101, 202, "task-other");
	await assert.rejects(
		writeRestrictedTaskObservation(mismatched.handoff, mismatched.agentA, mismatched.agentB, mismatched.observation, 303),
		/task ID mismatch/,
	);
	const state = await acceptedSources(101, 202);
	const receipt = JSON.parse(await readFile(state.agentB, "utf8"));
	receipt.state = "submitted";
	receipt.artifact_digest = digestOf(receipt);
	await writeFile(state.agentB, JSON.stringify(receipt));
	await assert.rejects(
		writeRestrictedTaskObservation(state.handoff, state.agentA, state.agentB, state.observation, 303),
		/state mismatch/,
	);
});

async function acceptedSources(agentAPid: number, agentBPid: number, agentBTask = "task-live-c") {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-agent-c-observation-"));
	const paths = {
		handoff: resolve(root, "handoff.json"),
		agentA: resolve(root, "agent-a.json"),
		agentB: resolve(root, "agent-b.json"),
		observation: resolve(root, "agent-c.json"),
	};
	await writeTaskHandoff(paths.handoff, "task-live-c");
	await writeActorReceipt(paths.agentA, "agent_a", "task-live-c", "accepted", agentAPid);
	await writeActorReceipt(paths.agentB, "agent_b", agentBTask, "accepted", agentBPid);
	return paths;
}

function digestOf(artifact: Record<string, unknown>): string {
	const unsigned = Object.fromEntries(Object.entries(artifact).filter(([key]) => key !== "artifact_digest"));
	return createHash("sha256").update(JSON.stringify(unsigned)).digest("hex");
}

function publicCommitment(artifact: Record<string, unknown>): string {
	return createHash("sha256")
		.update(JSON.stringify({
			task_id: artifact.task_id,
			state: artifact.state,
			source_handoff_digest: artifact.source_handoff_digest,
			source_agent_a_receipt_digest: artifact.source_agent_a_receipt_digest,
			source_agent_b_receipt_digest: artifact.source_agent_b_receipt_digest,
		}))
		.digest("hex");
}
